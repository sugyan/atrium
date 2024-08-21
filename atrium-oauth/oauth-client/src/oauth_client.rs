use crate::atproto::ClientMetadata;
use crate::constants::FALLBACK_ALG;
use crate::error::{Error, Result};
use crate::jose_key::generate;
use crate::resolver::*;
use crate::server_agent::{OAuthEndpointName, OAuthServerAgent};
use crate::store::state::{InternalStateData, StateStore};
use crate::types::{OAuthClientMetadata, OAuthParResponse};
use crate::utils::get_random_values;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use elliptic_curve::JwkEcKey;
use rand::rngs::ThreadRng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

pub struct OAuthClientConfig<S>
where
    S: StateStore,
{
    // Config
    pub client_metadata: ClientMetadata,
    // Stores
    pub state_store: S,
    // Services
    pub handle_resolver: HandleResolverConfig,
    pub plc_directory_url: Option<String>,
}

pub struct OAuthClient<S>
where
    S: StateStore,
{
    resolver: OAuthResolver,
    client_metadata: OAuthClientMetadata,
    state_store: S,
}

impl<S> OAuthClient<S>
where
    S: StateStore,
{
    pub fn new(config: OAuthClientConfig<S>) -> Result<Self> {
        // TODO: validate client metadata
        let client_metadata = config.client_metadata.validate()?;
        Ok(Self {
            resolver: OAuthResolver::new(IdentityResolver::new(
                Arc::new(
                    CommonResolver::new(CommonResolverConfig {
                        plc_directory_url: config.plc_directory_url,
                    })
                    .map_err(|e| Error::Resolver(crate::resolver::Error::DidResolver(e)))?,
                ),
                Self::handle_resolver(config.handle_resolver),
            )),
            client_metadata,
            state_store: config.state_store,
        })
    }
    pub async fn authorize(&mut self, input: impl AsRef<str>) -> Result<String> {
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
        let nonce = Self::generate_nonce();
        let state = Self::generate_nonce();
        let state_data = InternalStateData {
            iss: metadata.issuer.clone(),
            dpop_key: dpop_key.clone(),
        };
        self.state_store
            .set(state.clone(), state_data)
            .await
            .map_err(|e| Error::StateStore(Box::new(e)))?;

        // TODO: schema?
        let mut payload = HashMap::from_iter([
            (
                String::from("client_id"),
                self.client_metadata.client_id.clone(),
            ),
            (String::from("redirect_uri"), redirect_uri),
            (String::from("code_challenge"), String::from("dummy")),
            (String::from("code_challenge_method"), String::from("S256")),
            (String::from("response_type"), String::from("code")),
            (String::from("response_mode"), String::from("query")),
            // (String::from("nonce"), nonce),
            (String::from("state"), state),
        ]);
        if identity.is_some() {
            payload.insert(String::from("login_hint"), input.as_ref().into());
        }

        if metadata.pushed_authorization_request_endpoint.is_some() {
            let server =
                OAuthServerAgent::new(dpop_key, metadata.clone(), self.client_metadata.clone())?;
            let par_response = server
                .request::<HashMap<String, String>, OAuthParResponse>(
                    OAuthEndpointName::PushedAuthorizationRequest,
                    payload,
                )
                .await?;

            #[derive(Serialize)]
            struct AuthorizationParams {
                client_id: String,
                request_uri: String,
            }
            Ok(metadata.authorization_endpoint
                + "?"
                + &serde_html_form::to_string(AuthorizationParams {
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
    pub async fn callback(&self, input: impl AsRef<str>) -> Result<()> {
        #[derive(Debug, Deserialize)]
        struct CallbackParams {
            code: String,
            state: Option<String>,
            iss: Option<String>,
        }

        let params = serde_html_form::from_str::<CallbackParams>(input.as_ref())
            .map_err(|e| Error::Callback(e.to_string()))?;

        println!("params: {:?}", &params);
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
        )?;
        println!("{:?}", server.exchange_code(&params.code).await?);

        Ok(())
    }
    fn handle_resolver(handle_resolver_config: HandleResolverConfig) -> Arc<dyn HandleResolver> {
        match handle_resolver_config {
            HandleResolverConfig::AppView(uri) => Arc::new(AppViewResolver::new(uri)),
            HandleResolverConfig::Service(service) => service,
        }
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
    fn generate_nonce() -> String {
        URL_SAFE_NO_PAD.encode(get_random_values(&mut ThreadRng::default()))
    }
}
