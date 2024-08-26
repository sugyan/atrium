use crate::constants::FALLBACK_ALG;
use crate::error::{Error, Result};
use crate::jose_key::generate;
use crate::resolver::{OAuthResolver, OAuthResolverConfig};
use crate::server_agent::{OAuthRequest, OAuthServerAgent};
use crate::store::state::{InternalStateData, StateStore};
use crate::types::{
    AuthorizationCodeChallengeMethod, AuthorizationResponseType, CallbackParams,
    OAuthClientMetadata, OAuthPusehedAuthorizationRequestResponse,
    PushedAuthorizationRequestParameters, TokenSet,
};
use crate::utils::get_random_values;
use atrium_xrpc::HttpClient;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use elliptic_curve::JwkEcKey;
use rand::rngs::ThreadRng;
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::sync::Arc;

#[cfg(feature = "default-client")]
pub struct OAuthClientConfig<S, M>
where
    M: TryInto<OAuthClientMetadata>,
{
    // Config
    pub client_metadata: M,
    // Stores
    pub state_store: S,
    // Services
    pub resolver: OAuthResolverConfig,
}

#[cfg(not(feature = "default-client"))]
pub struct OAuthClientConfig<S, T, M>
where
    M: TryInto<OAuthClientMetadata>,
{
    // Config
    pub client_metadata: M,
    // Stores
    pub state_store: S,
    // Services
    pub resolver: OAuthResolverConfig,
    // Others
    pub http_client: T,
}

#[cfg(feature = "default-client")]
pub struct OAuthClient<S, T = crate::http_client::default::DefaultHttpClient>
where
    S: StateStore,
    T: HttpClient + Send + Sync + 'static,
{
    pub client_metadata: OAuthClientMetadata,
    resolver: Arc<OAuthResolver<T>>,
    state_store: S,
    http_client: Arc<T>,
}

#[cfg(not(feature = "default-client"))]
pub struct OAuthClient<S, T>
where
    S: StateStore,
    T: HttpClient + Send + Sync + 'static,
{
    pub client_metadata: OAuthClientMetadata,
    resolver: Arc<OAuthResolver<T>>,
    state_store: S,
    http_client: Arc<T>,
}

#[cfg(feature = "default-client")]
impl<S> OAuthClient<S, crate::http_client::default::DefaultHttpClient>
where
    S: StateStore,
{
    pub fn new<M>(config: OAuthClientConfig<S, M>) -> Result<Self>
    where
        M: TryInto<OAuthClientMetadata, Error = crate::atproto::Error>,
    {
        // let client_metadata = config.client_metadata.validate()?;
        let client_metadata = config.client_metadata.try_into()?;
        let http_client = Arc::new(crate::http_client::default::DefaultHttpClient::default());
        Ok(Self {
            client_metadata,
            resolver: Arc::new(OAuthResolver::new(config.resolver, http_client.clone())?),
            state_store: config.state_store,
            http_client,
        })
    }
}

#[cfg(not(feature = "default-client"))]
impl<S, T> OAuthClient<S, T>
where
    S: StateStore,
    T: HttpClient + Send + Sync + 'static,
{
    pub fn new<M>(config: OAuthClientConfig<S, T, M>) -> Result<Self>
    where
        M: TryInto<OAuthClientMetadata, Error = crate::atproto::Error>,
    {
        let client_metadata = config.client_metadata.try_into()?;
        let http_client = Arc::new(config.http_client);
        Ok(Self {
            client_metadata,
            resolver: Arc::new(OAuthResolver::new(config.resolver, http_client.clone())?),
            state_store: config.state_store,
            http_client,
        })
    }
}

impl<S, T> OAuthClient<S, T>
where
    S: StateStore,
    T: HttpClient + Send + Sync + 'static,
{
    pub async fn authorize(&self, input: impl AsRef<str>) -> Result<String> {
        let redirect_uri = {
            // TODO: use options.redirect_uri
            self.client_metadata.redirect_uris[0].clone()
        };
        let (metadata, identity) = self.resolver.resolve(input.as_ref()).await?;
        Self::generate_key(
            ["PS384", "RS256", "ES256K", "RS512", "ES384", "ES256"]
                .into_iter()
                .map(String::from)
                .collect(),
        );
        let Some(dpop_key) = Self::generate_key(
            metadata
                .dpop_signing_alg_values_supported
                .clone()
                .unwrap_or(vec![FALLBACK_ALG.into()]),
        ) else {
            return Err(Error::Authorize("none of the algorithms worked".into()));
        };
        let (code_challenge, verifier) = Self::generate_pkce();
        let state = Self::generate_nonce();
        let state_data = InternalStateData {
            iss: metadata.issuer.clone(),
            dpop_key: dpop_key.clone(),
            verifier,
        };
        self.state_store
            .set(state.clone(), state_data)
            .await
            .map_err(|e| Error::StateStore(Box::new(e)))?;
        let login_hint = if identity.is_some() {
            Some(input.as_ref().into())
        } else {
            None
        };
        let parameters = PushedAuthorizationRequestParameters {
            response_type: AuthorizationResponseType::Code,
            redirect_uri,
            state,
            response_mode: None,
            code_challenge,
            code_challenge_method: AuthorizationCodeChallengeMethod::S256,
            login_hint,
        };
        if metadata.pushed_authorization_request_endpoint.is_some() {
            let server = OAuthServerAgent::new(
                dpop_key,
                metadata.clone(),
                self.client_metadata.clone(),
                self.resolver.clone(),
                self.http_client.clone(),
            )?;
            let par_response = server
                .request::<OAuthPusehedAuthorizationRequestResponse>(
                    OAuthRequest::PushedAuthorizationRequest(parameters),
                )
                .await?;

            #[derive(Serialize)]
            struct Parameters {
                client_id: String,
                request_uri: String,
            }
            Ok(metadata.authorization_endpoint
                + "?"
                + &serde_html_form::to_string(Parameters {
                    client_id: self.client_metadata.client_id.clone(),
                    request_uri: par_response.request_uri,
                })
                .unwrap())
        } else if metadata.require_pushed_authorization_requests == Some(true) {
            Err(Error::Authorize(
                "server requires PAR but no endpoint is available".into(),
            ))
        } else {
            // now "the use of PAR is *mandatory* for all clients"
            // https://github.com/bluesky-social/proposals/tree/main/0004-oauth#framework
            todo!()
        }
    }
    pub async fn callback(&self, params: CallbackParams) -> Result<TokenSet> {
        let Some(state) = params.state else {
            return Err(Error::Callback("missing `state` parameter".into()));
        };
        let Some(state) = self
            .state_store
            .get(&state)
            .await
            .map_err(|e| Error::StateStore(Box::new(e)))?
        else {
            return Err(Error::Callback(format!(
                "unknown authorization state: {state}"
            )));
        };
        println!("state: {:?}", &state);

        let metadata = self
            .resolver
            .get_authorization_server_metadata(&state.iss)
            .await?;
        // https://datatracker.ietf.org/doc/html/rfc9207#section-2.4
        if let Some(iss) = params.iss {
            if iss != metadata.issuer {
                return Err(Error::Callback(format!(
                    "issuer mismatch: expected {}, got {iss}",
                    metadata.issuer
                )));
            }
        } else if metadata.authorization_response_iss_parameter_supported == Some(true) {
            return Err(Error::Callback("missing `iss` parameter".into()));
        }
        let server = OAuthServerAgent::new(
            state.dpop_key.clone(),
            metadata.clone(),
            self.client_metadata.clone(),
            self.resolver.clone(),
            self.http_client.clone(),
        )?;
        let token_set = server.exchange_code(&params.code, &state.verifier).await?;
        // TODO: verify id_token?

        Ok(token_set)
    }
    fn generate_key(mut algs: Vec<String>) -> Option<JwkEcKey> {
        // 256K > ES (256 > 384 > 512) > PS (256 > 384 > 512) > RS (256 > 384 > 512) > other (in original order)
        fn compare_algos(a: &String, b: &String) -> std::cmp::Ordering {
            if a == "ES256K" {
                return std::cmp::Ordering::Less;
            }
            if b == "ES256K" {
                return std::cmp::Ordering::Greater;
            }
            for prefix in ["ES", "PS", "RS"] {
                if let Some(stripped_a) = a.strip_prefix(prefix) {
                    if let Some(stripped_b) = b.strip_prefix(prefix) {
                        if let (Ok(len_a), Ok(len_b)) =
                            (stripped_a.parse::<u32>(), stripped_b.parse::<u32>())
                        {
                            return len_a.cmp(&len_b);
                        }
                    } else {
                        return std::cmp::Ordering::Less;
                    }
                } else if b.starts_with(prefix) {
                    return std::cmp::Ordering::Greater;
                }
            }
            std::cmp::Ordering::Equal
        }
        algs.sort_by(compare_algos);
        generate(&algs)
    }
    fn generate_pkce() -> (String, String) {
        // https://datatracker.ietf.org/doc/html/rfc7636#section-4.1
        let verifier =
            URL_SAFE_NO_PAD.encode(get_random_values::<_, 32>(&mut ThreadRng::default()));
        let mut hasher = Sha256::new();
        hasher.update(verifier.as_bytes());
        (
            URL_SAFE_NO_PAD.encode(Sha256::digest(verifier.as_bytes())),
            verifier,
        )
    }
    fn generate_nonce() -> String {
        URL_SAFE_NO_PAD.encode(get_random_values::<_, 16>(&mut ThreadRng::default()))
    }
}
