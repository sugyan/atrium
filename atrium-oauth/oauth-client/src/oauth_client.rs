use crate::constants::FALLBACK_ALG;
use crate::error::{Error, Result};
use crate::keyset::Keyset;
use crate::oauth_session::OAuthSession;
use crate::resolver::{OAuthResolver, OAuthResolverConfig};
use crate::server_agent::{OAuthRequest, OAuthServerAgent, OAuthServerFactory};
use crate::store::cached::Cached;
use crate::store::session::{Session, SessionStore};
use crate::store::state::{InternalStateData, StateStore};
use crate::types::{
    AuthorizationCodeChallengeMethod, AuthorizationResponseType, AuthorizeOptions, CallbackParams,
    OAuthAuthorizationServerMetadata, OAuthClientMetadata,
    OAuthPusehedAuthorizationRequestResponse, PushedAuthorizationRequestParameters, TokenSet,
    TryIntoOAuthClientMetadata,
};
use crate::utils::{compare_algos, generate_key, generate_nonce, get_random_values};
use atrium_api::types::string::Did;
use atrium_identity::{did::DidResolver, handle::HandleResolver, Resolver};
use atrium_xrpc::HttpClient;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use jose_jwk::{Jwk, JwkSet, Key};
use rand::rngs::ThreadRng;
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::sync::Arc;

#[cfg(feature = "default-client")]
pub struct OAuthClientConfig<S, N, M, D, H>
where
    M: TryIntoOAuthClientMetadata,
{
    // Config
    pub client_metadata: M,
    pub keys: Option<Vec<Jwk>>,
    // Stores
    pub state_store: S,
    pub session_store: N,
    // Services
    pub resolver: OAuthResolverConfig<D, H>,
}

#[cfg(not(feature = "default-client"))]
pub struct OAuthClientConfig<S, N, T, M, D, H>
where
    M: TryIntoOAuthClientMetadata,
{
    // Config
    pub client_metadata: M,
    pub keys: Option<Vec<Jwk>>,
    // Stores
    pub state_store: S,
    pub session_store: N,
    // Services
    pub resolver: OAuthResolverConfig<D, H>,
    // Others
    pub http_client: T,
}

#[cfg(feature = "default-client")]
pub struct OAuthClient<S, N, D, H, T = crate::http_client::default::DefaultHttpClient>
where
    S: StateStore,
    N: SessionStore,
    T: HttpClient + Send + Sync + 'static,
{
    pub client_metadata: OAuthClientMetadata,
    keyset: Option<Keyset>,
    resolver: Arc<OAuthResolver<T, D, H>>,
    server_factory: OAuthServerFactory<D, H, T>,
    state_store: S,
    session_store: N,
    _http_client: Arc<T>,
}

#[cfg(not(feature = "default-client"))]
pub struct OAuthClient<S, N, D, H, T>
where
    S: StateStore,
    N: SessionStore,
    T: HttpClient + Send + Sync + 'static,
{
    pub client_metadata: OAuthClientMetadata,
    keyset: Option<Keyset>,
    resolver: Arc<OAuthResolver<T, D, H>>,
    server_factory: OAuthServerFactory<D, H, T>,
    state_store: S,
    session_store: N,
    http_client: Arc<T>,
}

#[cfg(feature = "default-client")]
impl<S, N, D, H> OAuthClient<S, N, D, H, crate::http_client::default::DefaultHttpClient>
where
    S: StateStore,
    N: SessionStore,
{
    pub fn new<M>(config: OAuthClientConfig<S, N, M, D, H>) -> Result<Self>
    where
        M: TryIntoOAuthClientMetadata<Error = crate::atproto::Error>,
    {
        let keyset = if let Some(keys) = config.keys { Some(keys.try_into()?) } else { None };
        let client_metadata = config.client_metadata.try_into_client_metadata(&keyset)?;
        let http_client = Arc::new(crate::http_client::default::DefaultHttpClient::default());
        let resolver = Arc::new(OAuthResolver::new(config.resolver, http_client.clone()));
        Ok(Self {
            client_metadata: client_metadata.clone(),
            keyset: keyset.clone(),
            resolver: resolver.clone(),
            server_factory: OAuthServerFactory::new(resolver, client_metadata, keyset),
            state_store: config.state_store,
            session_store: config.session_store,
            _http_client: http_client,
        })
    }
}

#[cfg(not(feature = "default-client"))]
impl<S, N, D, H, T> OAuthClient<S, N, D, H, T>
where
    S: StateStore,
    N: SessionStore,
    T: HttpClient + Send + Sync + 'static,
{
    pub fn new<M>(config: OAuthClientConfig<S, N, T, M, D, H>) -> Result<Self>
    where
        M: TryIntoOAuthClientMetadata<Error = crate::atproto::Error>,
    {
        let keyset = if let Some(keys) = config.keys { Some(keys.try_into()?) } else { None };
        let client_metadata = config.client_metadata.try_into_client_metadata(&keyset)?;
        let http_client = Arc::new(config.http_client);
        Ok(Self {
            client_metadata,
            keyset,
            resolver: Arc::new(OAuthResolver::new(config.resolver, http_client.clone())),
            state_store: config.state_store,
            session_store: config.session_store,
            http_client,
        })
    }
}

impl<S, N, D, H, T> OAuthClient<S, N, D, H, T>
where
    S: StateStore,
    N: SessionStore,
    D: DidResolver + Send + Sync + 'static,
    H: HandleResolver + Send + Sync + 'static,
    T: HttpClient + Send + Sync + 'static,
{
    pub fn jwks(&self) -> JwkSet {
        self.keyset.as_ref().map(|keyset| keyset.public_jwks()).unwrap_or_default()
    }
    pub async fn authorize(
        &self,
        input: impl AsRef<str>,
        options: AuthorizeOptions,
    ) -> Result<String> {
        let redirect_uri = if let Some(uri) = options.redirect_uri {
            if !self.client_metadata.redirect_uris.contains(&uri) {
                return Err(Error::Authorize("invalid redirect_uri".into()));
            }
            uri
        } else {
            self.client_metadata.redirect_uris[0].clone()
        };
        let (metadata, identity) = self.resolver.resolve(input.as_ref()).await?;
        let Some(dpop_key) = Self::generate_dpop_key(&metadata) else {
            return Err(Error::Authorize("none of the algorithms worked".into()));
        };
        let (code_challenge, verifier) = Self::generate_pkce();
        let state = generate_nonce();
        let state_data = InternalStateData {
            iss: metadata.issuer.clone(),
            dpop_key: dpop_key.clone(),
            verifier,
        };
        self.state_store
            .set(state.clone(), state_data)
            .await
            .map_err(|e| Error::StateStore(Box::new(e)))?;
        let login_hint = if identity.is_some() { Some(input.as_ref().into()) } else { None };
        let parameters = PushedAuthorizationRequestParameters {
            response_type: AuthorizationResponseType::Code,
            redirect_uri,
            state,
            scope: options.scopes.map(|v| v.join(" ")),
            response_mode: None,
            code_challenge,
            code_challenge_method: AuthorizationCodeChallengeMethod::S256,
            login_hint,
            prompt: options.prompt.map(String::from),
        };
        if metadata.pushed_authorization_request_endpoint.is_some() {
            let server =
                self.server_factory.from_issuer(&metadata.issuer, dpop_key.clone()).await?;
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
            Err(Error::Authorize("server requires PAR but no endpoint is available".into()))
        } else {
            // now "the use of PAR is *mandatory* for all clients"
            // https://github.com/bluesky-social/proposals/tree/main/0004-oauth#framework
            todo!()
        }
    }
    pub async fn callback(&self, params: CallbackParams) -> Result<TokenSet> {
        let Some(state_key) = params.state else {
            return Err(Error::Callback("missing `state` parameter".into()));
        };

        let Some(state) =
            self.state_store.get(&state_key).await.map_err(|e| Error::StateStore(Box::new(e)))?
        else {
            return Err(Error::Callback(format!("unknown authorization state: {state_key}")));
        };
        // Prevent any kind of replay
        self.state_store.del(&state_key).await.map_err(|e| Error::StateStore(Box::new(e)))?;

        let metadata = self.resolver.get_authorization_server_metadata(&state.iss).await?;
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
        let server =
            self.server_factory.from_issuer(&metadata.issuer, state.dpop_key.clone()).await?;
        let token_set = server.exchange_code(&params.code, &state.verifier).await?;

        let sub: Did = token_set.sub.parse().unwrap();

        if let Err(_error) = self
            .session_store
            .set(sub.clone(), Cached::new(Session::new(state.dpop_key.clone(), token_set.clone())))
            .await
        {
            let _ = server
                .revoke(
                    token_set.refresh_token.as_deref().unwrap_or_else(|| &token_set.access_token),
                )
                .await;

            todo!();
            // return Err(error);
        }
        let _session = self.create_session(server, sub);

        Ok(token_set)
    }
    fn generate_dpop_key(metadata: &OAuthAuthorizationServerMetadata) -> Option<Key> {
        let mut algs =
            metadata.dpop_signing_alg_values_supported.clone().unwrap_or(vec![FALLBACK_ALG.into()]);
        algs.sort_by(compare_algos);
        generate_key(&algs)
    }
    fn generate_pkce() -> (String, String) {
        // https://datatracker.ietf.org/doc/html/rfc7636#section-4.1
        let verifier =
            URL_SAFE_NO_PAD.encode(get_random_values::<_, 32>(&mut ThreadRng::default()));
        let mut hasher = Sha256::new();
        hasher.update(verifier.as_bytes());
        (URL_SAFE_NO_PAD.encode(Sha256::digest(verifier.as_bytes())), verifier)
    }
    fn create_session(
        &self,
        server: OAuthServerAgent<T, D, H>,
        sub: Did,
    ) -> OAuthSession<N, T, D, H> {
        OAuthSession::new(server, sub, self.session_store.clone())
    }
}
