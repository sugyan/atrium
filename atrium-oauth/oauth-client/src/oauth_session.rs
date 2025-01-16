mod inner;
mod store;

use crate::{http_client::dpop, server_agent::OAuthServerAgent, DpopClient, TokenSet};
use atrium_api::{
    agent::{CloneWithProxy, Configure, InnerStore, SessionManager},
    types::string::Did,
};
use atrium_common::store::{memory::MemoryStore, Store};
use atrium_xrpc::{
    http::{Request, Response},
    HttpClient, OutputDataOrBytes, XrpcClient, XrpcRequest,
};
use jose_jwk::Key;
use serde::{de::DeserializeOwned, Serialize};
use std::{fmt::Debug, sync::Arc};
use store::MemorySessionStore;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Dpop(#[from] dpop::Error),
    #[error(transparent)]
    Store(#[from] atrium_common::store::memory::Error),
}

pub struct OAuthSession<T, D, H, S = MemoryStore<String, String>>
where
    T: HttpClient + Send + Sync + 'static,
    S: Store<String, String>,
{
    server_agent: OAuthServerAgent<T, D, H>,
    store: Arc<InnerStore<store::MemorySessionStore, String>>,
    inner: inner::Client<store::MemorySessionStore, DpopClient<T, S>>,
    token_set: TokenSet, // TODO: replace with a session store?
}

impl<T, D, H> OAuthSession<T, D, H>
where
    T: HttpClient + Send + Sync,
{
    pub(crate) async fn new(
        server_agent: OAuthServerAgent<T, D, H>,
        dpop_key: Key,
        http_client: Arc<T>,
        token_set: TokenSet,
    ) -> Result<Self, Error> {
        let store = Arc::new(InnerStore::new(MemorySessionStore::default(), token_set.aud.clone()));
        store.set(token_set.access_token.clone()).await?;
        let inner = inner::Client::new(
            Arc::clone(&store),
            DpopClient::new(
                dpop_key,
                http_client.clone(),
                false,
                &server_agent.server_metadata.token_endpoint_auth_signing_alg_values_supported,
            )?,
        );
        Ok(Self { server_agent, store, inner, token_set })
    }
}

impl<T, D, H, S> HttpClient for OAuthSession<T, D, H, S>
where
    T: HttpClient + Send + Sync,
    D: Send + Sync,
    H: Send + Sync,
    S: Store<String, String> + Send + Sync + 'static,
    S::Error: std::error::Error + Send + Sync,
{
    async fn send_http(
        &self,
        request: Request<Vec<u8>>,
    ) -> Result<Response<Vec<u8>>, Box<dyn std::error::Error + Send + Sync + 'static>> {
        self.inner.send_http(request).await
    }
}

impl<T, D, H, S> XrpcClient for OAuthSession<T, D, H, S>
where
    T: HttpClient + Send + Sync,
    D: Send + Sync,
    H: Send + Sync,
    S: Store<String, String> + Send + Sync + 'static,
    S::Error: std::error::Error + Send + Sync,
{
    fn base_uri(&self) -> String {
        self.inner.base_uri()
    }
    async fn send_xrpc<P, I, O, E>(
        &self,
        request: &XrpcRequest<P, I>,
    ) -> Result<OutputDataOrBytes<O>, atrium_xrpc::Error<E>>
    where
        P: Serialize + Send + Sync,
        I: Serialize + Send + Sync,
        O: DeserializeOwned + Send + Sync,
        E: DeserializeOwned + Send + Sync + Debug,
    {
        self.inner.send_xrpc(request).await
    }
}

impl<T, D, H, S> SessionManager for OAuthSession<T, D, H, S>
where
    T: HttpClient + Send + Sync,
    D: Send + Sync,
    H: Send + Sync,
    S: Store<String, String> + Send + Sync + 'static,
    S::Error: std::error::Error + Send + Sync,
{
    async fn did(&self) -> Option<Did> {
        Some(self.token_set.sub.clone())
    }
}

impl<T, D, H, S> Configure for OAuthSession<T, D, H, S>
where
    T: HttpClient + Send + Sync,
    S: Store<String, String> + Send + Sync + 'static,
{
    fn configure_endpoint(&self, endpoint: String) {
        self.inner.configure_endpoint(endpoint);
    }
    fn configure_labelers_header(&self, labeler_dids: Option<Vec<(Did, bool)>>) {
        self.inner.configure_labelers_header(labeler_dids);
    }
    fn configure_proxy_header(&self, did: Did, service_type: impl AsRef<str>) {
        self.inner.configure_proxy_header(did, service_type);
    }
}

impl<T, D, H, S> CloneWithProxy for OAuthSession<T, D, H, S>
where
    T: HttpClient + Send + Sync,
    S: Store<String, String> + Send + Sync + 'static,
{
    fn clone_with_proxy(&self, did: Did, service_type: impl AsRef<str>) -> Self {
        Self {
            server_agent: self.server_agent.clone(),
            store: self.store.clone(),
            inner: self.inner.clone_with_proxy(did, service_type),
            token_set: self.token_set.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        jose::jwt::Claims, resolver::OAuthResolver, types::OAuthTokenType, OAuthResolverConfig,
    };
    use atrium_api::{
        agent::{Agent, AtprotoServiceType},
        client::Service,
        did_doc::DidDocument,
        types::string::Handle,
        xrpc::http::{header::CONTENT_TYPE, HeaderMap, HeaderName, HeaderValue, StatusCode},
    };
    use atrium_common::resolver::Resolver;
    use atrium_identity::{did::DidResolver, handle::HandleResolver};
    use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
    use tokio::sync::Mutex;

    #[derive(Default)]
    struct RecordData {
        host: Option<String>,
        headers: HeaderMap<HeaderValue>,
    }

    struct MockHttpClient {
        data: Arc<Mutex<Option<RecordData>>>,
    }

    impl HttpClient for MockHttpClient {
        async fn send_http(
            &self,
            request: Request<Vec<u8>>,
        ) -> Result<Response<Vec<u8>>, Box<dyn std::error::Error + Send + Sync + 'static>> {
            let mut headers = request.headers().clone();
            let Some(authorization) = headers
                .remove("authorization")
                .and_then(|value| value.to_str().map(String::from).ok())
            else {
                return Ok(Response::builder().status(StatusCode::UNAUTHORIZED).body(Vec::new())?);
            };
            let Some(_token) = authorization.strip_prefix("DPoP ") else {
                panic!("authorization header should start with DPoP");
            };
            // TODO: verify token

            let dpop_jwt = headers.remove("dpop").expect("dpop header should be present");
            let payload = dpop_jwt
                .to_str()
                .expect("dpop header should be valid")
                .split('.')
                .nth(1)
                .expect("dpop header should have 2 parts");
            let claims = URL_SAFE_NO_PAD
                .decode(payload)
                .ok()
                .and_then(|value| serde_json::from_slice::<Claims>(&value).ok())
                .expect("dpop payload should be valid");
            assert!(claims.registered.iat.is_some());
            assert!(claims.registered.jti.is_some());
            assert_eq!(claims.public.htm, Some(request.method().to_string()));
            assert_eq!(claims.public.htu, Some(request.uri().to_string()));

            self.data
                .lock()
                .await
                .replace(RecordData { host: request.uri().host().map(String::from), headers });
            let output = atrium_api::com::atproto::server::get_service_auth::OutputData {
                token: String::from("fake_token"),
            };
            Response::builder()
                .header(CONTENT_TYPE, "application/json")
                .body(serde_json::to_vec(&output)?)
                .map_err(|e| e.into())
        }
    }

    struct NoopDidResolver;

    impl Resolver for NoopDidResolver {
        type Input = Did;
        type Output = DidDocument;
        type Error = atrium_identity::Error;
        async fn resolve(&self, _: &Self::Input) -> Result<Self::Output, Self::Error> {
            unimplemented!()
        }
    }

    impl DidResolver for NoopDidResolver {}

    struct NoopHandleResolver;

    impl Resolver for NoopHandleResolver {
        type Input = Handle;
        type Output = Did;
        type Error = atrium_identity::Error;
        async fn resolve(&self, _: &Self::Input) -> Result<Self::Output, Self::Error> {
            unimplemented!()
        }
    }

    impl HandleResolver for NoopHandleResolver {}

    async fn oauth_session(
        data: Arc<Mutex<Option<RecordData>>>,
    ) -> OAuthSession<
        MockHttpClient,
        NoopDidResolver,
        NoopHandleResolver,
        MemoryStore<String, String>,
    > {
        let dpop_key = serde_json::from_str::<Key>(
            r#"{
                "kty": "EC",
                "crv": "P-256",
                "x": "NIRNgPVAwnVNzN5g2Ik2IMghWcjnBOGo9B-lKXSSXFs",
                "y": "iWF-Of43XoSTZxcadO9KWdPTjiCoviSztYw7aMtZZMc",
                "d": "9MuCYfKK4hf95p_VRj6cxKJwORTgvEU3vynfmSgFH2M"
            }"#,
        )
        .expect("key should be valid");
        let http_client = Arc::new(MockHttpClient { data });
        let resolver = Arc::new(OAuthResolver::new(
            OAuthResolverConfig {
                did_resolver: NoopDidResolver,
                handle_resolver: NoopHandleResolver,
                authorization_server_metadata: Default::default(),
                protected_resource_metadata: Default::default(),
            },
            Arc::clone(&http_client),
        ));
        let keyset = None;
        let server_agent = OAuthServerAgent::new(
            dpop_key.clone(),
            Default::default(),
            Default::default(),
            resolver,
            Arc::clone(&http_client),
            keyset,
        )
        .expect("failed to create server agent");
        let token_set = TokenSet {
            iss: String::from("https://iss.example.com"),
            sub: Did::new(String::from("did:fake:sub.test")).expect("did should be valid"),
            aud: String::from("https://aud.example.com"),
            scope: None,
            refresh_token: None,
            access_token: String::from("access_token"),
            token_type: OAuthTokenType::DPoP,
            expires_at: None,
        };
        OAuthSession::new(server_agent, dpop_key, http_client, token_set)
            .await
            .expect("failed to create oauth session")
    }

    async fn oauth_agent(
        data: Arc<Mutex<Option<RecordData>>>,
    ) -> Agent<impl SessionManager + Configure + CloneWithProxy> {
        Agent::new(oauth_session(data).await)
    }

    async fn call_service(
        service: &Service<impl SessionManager + Send + Sync>,
    ) -> Result<(), atrium_xrpc::Error<atrium_api::com::atproto::server::get_service_auth::Error>>
    {
        let output = service
            .com
            .atproto
            .server
            .get_service_auth(
                atrium_api::com::atproto::server::get_service_auth::ParametersData {
                    aud: Did::new(String::from("did:fake:handle.test"))
                        .expect("did should be valid"),
                    exp: None,
                    lxm: None,
                }
                .into(),
            )
            .await?;
        assert_eq!(output.token, "fake_token");
        Ok(())
    }

    #[tokio::test]
    async fn test_new() -> Result<(), Box<dyn std::error::Error>> {
        let agent = oauth_agent(Arc::new(Mutex::new(Default::default()))).await;
        assert_eq!(agent.did().await.as_deref(), Some("did:fake:sub.test"));
        Ok(())
    }

    #[tokio::test]
    async fn test_configure_endpoint() -> Result<(), Box<dyn std::error::Error>> {
        let data = Arc::new(Mutex::new(Default::default()));
        let agent = oauth_agent(Arc::clone(&data)).await;
        call_service(&agent.api).await?;
        assert_eq!(
            data.lock().await.as_ref().expect("data should be recorded").host.as_deref(),
            Some("aud.example.com")
        );
        agent.configure_endpoint(String::from("https://pds.example.com"));
        call_service(&agent.api).await?;
        assert_eq!(
            data.lock().await.as_ref().expect("data should be recorded").host.as_deref(),
            Some("pds.example.com")
        );
        Ok(())
    }

    #[tokio::test]
    async fn test_configure_labelers_header() -> Result<(), Box<dyn std::error::Error>> {
        let data = Arc::new(Mutex::new(Default::default()));
        let agent = oauth_agent(Arc::clone(&data)).await;
        // not configured
        {
            call_service(&agent.api).await?;
            assert_eq!(
                data.lock().await.as_ref().expect("data should be recorded").headers,
                HeaderMap::new()
            );
        }
        // configured 1
        {
            agent.configure_labelers_header(Some(vec![(
                Did::new(String::from("did:fake:labeler.test"))?,
                false,
            )]));
            call_service(&agent.api).await?;
            assert_eq!(
                data.lock().await.as_ref().expect("data should be recorded").headers,
                HeaderMap::from_iter([(
                    HeaderName::from_static("atproto-accept-labelers"),
                    HeaderValue::from_static("did:fake:labeler.test"),
                )])
            );
        }
        // configured 2
        {
            agent.configure_labelers_header(Some(vec![
                (Did::new(String::from("did:fake:labeler.test_redact"))?, true),
                (Did::new(String::from("did:fake:labeler.test"))?, false),
            ]));
            call_service(&agent.api).await?;
            assert_eq!(
                data.lock().await.as_ref().expect("data should be recorded").headers,
                HeaderMap::from_iter([(
                    HeaderName::from_static("atproto-accept-labelers"),
                    HeaderValue::from_static(
                        "did:fake:labeler.test_redact;redact, did:fake:labeler.test"
                    ),
                )])
            );
        }
        Ok(())
    }

    #[tokio::test]
    async fn test_configure_proxy_header() -> Result<(), Box<dyn std::error::Error>> {
        let data = Arc::new(Mutex::new(Default::default()));
        let agent = oauth_agent(data.clone()).await;
        // not configured
        {
            call_service(&agent.api).await?;
            assert_eq!(
                data.lock().await.as_ref().expect("data should be recorded").headers,
                HeaderMap::new()
            );
        }
        // labeler service
        {
            agent.configure_proxy_header(
                Did::new(String::from("did:fake:service.test"))?,
                AtprotoServiceType::AtprotoLabeler,
            );
            call_service(&agent.api).await?;
            assert_eq!(
                data.lock().await.as_ref().expect("data should be recorded").headers,
                HeaderMap::from_iter([(
                    HeaderName::from_static("atproto-proxy"),
                    HeaderValue::from_static("did:fake:service.test#atproto_labeler"),
                )])
            );
        }
        // custom service
        {
            agent.configure_proxy_header(
                Did::new(String::from("did:fake:service.test"))?,
                "custom_service",
            );
            call_service(&agent.api).await?;
            assert_eq!(
                data.lock().await.as_ref().expect("data should be recorded").headers,
                HeaderMap::from_iter([(
                    HeaderName::from_static("atproto-proxy"),
                    HeaderValue::from_static("did:fake:service.test#custom_service"),
                )])
            );
        }
        // api_with_proxy
        {
            call_service(
                &agent.api_with_proxy(
                    Did::new(String::from("did:fake:service.test"))?,
                    "temp_service",
                ),
            )
            .await?;
            assert_eq!(
                data.lock().await.as_ref().expect("data should be recorded").headers,
                HeaderMap::from_iter([(
                    HeaderName::from_static("atproto-proxy"),
                    HeaderValue::from_static("did:fake:service.test#temp_service"),
                )])
            );
            call_service(&agent.api).await?;
            assert_eq!(
                data.lock().await.as_ref().expect("data should be recorded").headers,
                HeaderMap::from_iter([(
                    HeaderName::from_static("atproto-proxy"),
                    HeaderValue::from_static("did:fake:service.test#custom_service"),
                )])
            );
        }
        Ok(())
    }

    #[tokio::test]
    async fn test_xrpc_without_token() -> Result<(), Box<dyn std::error::Error>> {
        let oauth_session = oauth_session(Arc::new(Mutex::new(Default::default()))).await;
        oauth_session.store.clear().await?;
        let agent = Agent::new(oauth_session);
        let result = agent
            .api
            .com
            .atproto
            .server
            .get_service_auth(
                atrium_api::com::atproto::server::get_service_auth::ParametersData {
                    aud: Did::new(String::from("did:fake:handle.test"))
                        .expect("did should be valid"),
                    exp: None,
                    lxm: None,
                }
                .into(),
            )
            .await;
        match result.expect_err("should fail without token") {
            atrium_xrpc::Error::XrpcResponse(err) => {
                assert_eq!(err.status, StatusCode::UNAUTHORIZED);
            }
            _ => panic!("unexpected error"),
        }
        Ok(())
    }
}
