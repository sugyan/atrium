use crate::{
    constants::FALLBACK_ALG,
    error::{Error, Result},
    keyset::Keyset,
    oauth_session::OAuthSession,
    resolver::{OAuthResolver, OAuthResolverConfig},
    server_agent::{OAuthRequest, OAuthServerAgent},
    store::{
        session::{Session, SessionStore},
        session_getter::{SessionGetter, SessionHandle},
        state::{InternalStateData, StateStore},
    },
    types::{
        AuthorizationCodeChallengeMethod, AuthorizationResponseType, AuthorizeOptions,
        CallbackParams, OAuthAuthorizationServerMetadata, OAuthClientMetadata,
        OAuthPusehedAuthorizationRequestResponse, PushedAuthorizationRequestParameters,
        TryIntoOAuthClientMetadata,
    },
    utils::{compare_algos, generate_key, generate_nonce},
};
use atrium_api::types::string::Did;
use atrium_common::resolver::Resolver;
use atrium_identity::{did::DidResolver, handle::HandleResolver};
use atrium_xrpc::HttpClient;
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use jose_jwk::{Jwk, JwkSet, Key};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::sync::Arc;

#[cfg(feature = "default-client")]
pub struct OAuthClientConfig<S0, S1, M, D, H>
where
    M: TryIntoOAuthClientMetadata,
{
    // Config
    pub client_metadata: M,
    pub keys: Option<Vec<Jwk>>,
    // Stores
    pub state_store: S0,
    pub session_store: S1,
    // Services
    pub resolver: OAuthResolverConfig<D, H>,
}

#[cfg(not(feature = "default-client"))]
pub struct OAuthClientConfig<S0, S1, T, M, D, H>
where
    M: TryIntoOAuthClientMetadata,
{
    // Config
    pub client_metadata: M,
    pub keys: Option<Vec<Jwk>>,
    // Stores
    pub state_store: S0,
    pub session_store: S1,
    // Services
    pub resolver: OAuthResolverConfig<D, H>,
    // Others
    pub http_client: T,
}

#[cfg(feature = "default-client")]
pub struct OAuthClient<S0, S1, D, H, T = crate::http_client::default::DefaultHttpClient>
where
    T: HttpClient + Send + Sync + 'static,
{
    pub client_metadata: OAuthClientMetadata,
    keyset: Option<Keyset>,
    resolver: Arc<OAuthResolver<T, D, H>>,
    state_store: S0,
    session_getter: SessionGetter<S1>,
    http_client: Arc<T>,
}

#[cfg(not(feature = "default-client"))]
pub struct OAuthClient<S0, S1, D, H, T>
where
    T: HttpClient + Send + Sync + 'static,
{
    pub client_metadata: OAuthClientMetadata,
    keyset: Option<Keyset>,
    resolver: Arc<OAuthResolver<T, D, H>>,
    state_store: S0,
    session_getter: SessionGetter<S1>,
    http_client: Arc<T>,
}

#[cfg(feature = "default-client")]
impl<S0, S1, D, H> OAuthClient<S0, S1, D, H, crate::http_client::default::DefaultHttpClient> {
    pub fn new<M>(config: OAuthClientConfig<S0, S1, M, D, H>) -> Result<Self>
    where
        M: TryIntoOAuthClientMetadata<Error = crate::atproto::Error>,
    {
        let keyset = if let Some(keys) = config.keys { Some(keys.try_into()?) } else { None };
        let client_metadata = config.client_metadata.try_into_client_metadata(&keyset)?;
        let http_client = Arc::new(crate::http_client::default::DefaultHttpClient::default());
        Ok(Self {
            client_metadata,
            keyset,
            resolver: Arc::new(OAuthResolver::new(config.resolver, http_client.clone())),
            state_store: config.state_store,
            session_getter: SessionGetter::new(config.session_store),
            http_client,
        })
    }
}

#[cfg(not(feature = "default-client"))]
impl<S0, S1, D, H, T> OAuthClient<S0, S1, D, H, T>
where
    T: HttpClient + Send + Sync + 'static,
{
    pub fn new<M>(config: OAuthClientConfig<S0, S1, T, M, D, H>) -> Result<Self>
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
            session_getter: SessionGetter::new(config.session_store),
            http_client,
        })
    }
}

impl<S0, S1, D, H, T> OAuthClient<S0, S1, D, H, T>
where
    S0: StateStore + Send + Sync + 'static,
    S1: SessionStore + Send + Sync + 'static,
    D: DidResolver + Send + Sync + 'static,
    H: HandleResolver + Send + Sync + 'static,
    T: HttpClient + Send + Sync + 'static,
    S0::Error: std::error::Error + Send + Sync + 'static,
    S1::Error: std::error::Error + Send + Sync + 'static,
{
    pub fn jwks(&self) -> JwkSet {
        self.keyset.as_ref().map(|keyset| keyset.public_jwks()).unwrap_or_default()
    }
    /// Start the authorization process.
    ///
    /// This method will return a URL that the user should be redirected to.
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
            app_state: options.state,
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
            scope: Some(options.scopes.iter().map(AsRef::as_ref).collect::<Vec<_>>().join(" ")),
            response_mode: None,
            code_challenge,
            code_challenge_method: AuthorizationCodeChallengeMethod::S256,
            login_hint,
            prompt: options.prompt.map(String::from),
        };
        if metadata.pushed_authorization_request_endpoint.is_some() {
            let server = self.create_server_agent(dpop_key, metadata.clone())?;
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
    /// Handle the callback from the authorization server.
    ///
    /// This method will exchange the authorization code for an access token and store the session,
    /// and return the [`OAuthSession`] and the application state.
    pub async fn callback(
        &self,
        params: CallbackParams,
    ) -> Result<(OAuthSession<T, D, H, S1>, Option<String>)> {
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
        let server = self.create_server_agent(state.dpop_key.clone(), metadata.clone())?;
        match server.exchange_code(&params.code, &state.verifier).await {
            Ok(token_set) => {
                let sub = token_set.sub.clone();
                let session_handle = self
                    .session_getter
                    .set(sub.clone(), Session { dpop_key: state.dpop_key.clone(), token_set })
                    .await
                    .map_err(|e| Error::SessionStore(Box::new(e)))?;
                Ok((self.create_session(server, session_handle).await?, state.app_state))
            }
            Err(_) => {
                todo!()
            }
        }
    }
    /// Load a stored session by giving the subject DID.
    ///
    /// This method will return the [`OAuthSession`] if it exists.
    pub async fn restore(&self, sub: &Did) -> Result<OAuthSession<T, D, H, S1>> {
        let session_handle = self
            .session_getter
            .get(sub)
            .await
            .map_err(|e| Error::SessionStore(Box::new(e)))?
            .ok_or_else(|| Error::SessionNotFound)?;
        let session = session_handle.read().await;
        self.create_session(
            self.create_server_agent(
                session.dpop_key,
                self.resolver.get_authorization_server_metadata(&session.token_set.iss).await?,
            )?,
            session_handle,
        )
        .await
    }
    async fn create_session(
        &self,
        server: OAuthServerAgent<T, D, H>,
        session_handle: SessionHandle<S1>,
    ) -> Result<OAuthSession<T, D, H, S1>> {
        Ok(OAuthSession::new(server, Arc::clone(&self.http_client), session_handle).await?)
    }
    fn create_server_agent(
        &self,
        dpop_key: Key,
        server_metadata: OAuthAuthorizationServerMetadata,
    ) -> Result<OAuthServerAgent<T, D, H>> {
        Ok(OAuthServerAgent::new(
            dpop_key,
            server_metadata,
            self.client_metadata.clone(),
            self.resolver.clone(),
            self.http_client.clone(),
            self.keyset.clone(),
        )?)
    }
    fn generate_dpop_key(metadata: &OAuthAuthorizationServerMetadata) -> Option<Key> {
        let mut algs =
            metadata.dpop_signing_alg_values_supported.clone().unwrap_or(vec![FALLBACK_ALG.into()]);
        algs.sort_by(compare_algos);
        generate_key(&algs)
    }
    fn generate_pkce() -> (String, String) {
        // https://datatracker.ietf.org/doc/html/rfc7636#section-4.1
        let verifier = [generate_nonce(), generate_nonce()].join("");
        (URL_SAFE_NO_PAD.encode(Sha256::digest(&verifier)), verifier)
    }
}
