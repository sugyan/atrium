mod inner;
mod store;

use self::store::MemorySessionStore;
use crate::{
    http_client::dpop::{self, DpopClient},
    server_agent::OAuthServerAgent,
    store::{session::SessionStore, session_getter::SessionHandle},
};
use atrium_api::{
    agent::{utils::SessionWithEndpointStore, CloneWithProxy, Configure, SessionManager},
    types::string::Did,
};
use atrium_identity::{did::DidResolver, handle::HandleResolver};
use atrium_xrpc::{
    http::{Request, Response},
    HttpClient, OutputDataOrBytes, XrpcClient, XrpcRequest,
};
use serde::{de::DeserializeOwned, Serialize};
use std::{fmt::Debug, sync::Arc};
use thiserror::Error;
use tokio::sync::RwLock;

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Dpop(#[from] dpop::Error),
    #[error(transparent)]
    Store(#[from] atrium_common::store::memory::Error),
}

pub struct OAuthSession<T, D, H, S>
where
    T: HttpClient + Send + Sync + 'static,
{
    store: Arc<SessionWithEndpointStore<store::MemorySessionStore, String>>,
    inner: inner::Client<S, T, D, H>,
    session_handle: Arc<RwLock<SessionHandle<S>>>,
}

impl<T, D, H, S> OAuthSession<T, D, H, S>
where
    T: HttpClient + Send + Sync,
    S: SessionStore + Send + Sync + 'static,
{
    pub(crate) async fn new(
        server_agent: OAuthServerAgent<T, D, H>,
        http_client: Arc<T>,
        session_handle: SessionHandle<S>,
    ) -> Result<Self, Error> {
        // initialize SessionWithEndpointStore
        let (dpop_key, token_set) = {
            let s = session_handle.read().await;
            (s.dpop_key.clone(), s.token_set.clone())
        };
        let store = Arc::new(SessionWithEndpointStore::new(
            MemorySessionStore::default(),
            token_set.aud.clone(),
        ));
        store.set(token_set.access_token.clone()).await?;
        // initialize inner client
        let session_handle = Arc::new(RwLock::new(session_handle));
        let inner = inner::Client::new(
            Arc::clone(&store),
            DpopClient::new(
                dpop_key,
                http_client,
                false,
                &server_agent.server_metadata.token_endpoint_auth_signing_alg_values_supported,
            )?,
            server_agent,
            Arc::clone(&session_handle),
        );
        Ok(Self { store, inner, session_handle })
    }
}

impl<T, D, H, S> HttpClient for OAuthSession<T, D, H, S>
where
    T: HttpClient + Send + Sync + 'static,
    D: Send + Sync,
    H: Send + Sync,
    S: Send + Sync,
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
    T: HttpClient + Send + Sync + 'static,
    D: DidResolver + Send + Sync + 'static,
    H: HandleResolver + Send + Sync + 'static,
    S: SessionStore + Send + Sync + 'static,
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
    T: HttpClient + Send + Sync + 'static,
    D: DidResolver + Send + Sync + 'static,
    H: HandleResolver + Send + Sync + 'static,
    S: SessionStore + Send + Sync + 'static,
{
    async fn did(&self) -> Option<Did> {
        Some(self.session_handle.read().await.read().await.token_set.sub.clone())
    }
}

impl<T, D, H, S> Configure for OAuthSession<T, D, H, S>
where
    T: HttpClient + Send + Sync,
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
{
    fn clone_with_proxy(&self, did: Did, service_type: impl AsRef<str>) -> Self {
        Self {
            store: self.store.clone(),
            inner: self.inner.clone_with_proxy(did, service_type),
            session_handle: Arc::clone(&self.session_handle),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        jose::jwt::Claims,
        resolver::OAuthResolver,
        store::session::Session,
        types::{
            OAuthAuthorizationServerMetadata, OAuthProtectedResourceMetadata, OAuthTokenResponse,
            OAuthTokenType, RefreshRequestParameters, TokenSet, TryIntoOAuthClientMetadata,
        },
        AtprotoLocalhostClientMetadata, OAuthResolverConfig,
    };
    use atrium_api::{
        agent::{Agent, AtprotoServiceType},
        client::Service,
        did_doc::{DidDocument, Service as DidDocumentService},
        types::string::Handle,
        xrpc::http::{header::CONTENT_TYPE, HeaderMap, HeaderName, HeaderValue, StatusCode},
    };
    use atrium_common::{resolver::Resolver, store::Store};
    use atrium_identity::{did::DidResolver, handle::HandleResolver};
    use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
    use jose_jwk::Key;
    use std::{collections::HashMap, time::Duration};
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
            // tick tokio time
            tokio::time::sleep(std::time::Duration::from_micros(0)).await;

            match request.uri().path() {
                // Resolve protected resource
                "/.well-known/oauth-protected-resource" => {
                    assert_eq!(request.uri().host(), Some("aud.example.com"));
                    return Response::builder()
                        .status(StatusCode::OK)
                        .header(CONTENT_TYPE, "application/json")
                        .body(serde_json::to_vec(&OAuthProtectedResourceMetadata {
                            resource: String::from("https://aud.example.com"),
                            authorization_servers: Some(vec![String::from(
                                "https://iss.example.com",
                            )]),
                            ..Default::default()
                        })?)
                        .map_err(|e| e.into());
                }
                // Resolve authorization server metadata
                "/.well-known/oauth-authorization-server" => {
                    assert_eq!(request.uri().host(), Some("iss.example.com"));
                    return Response::builder()
                        .status(StatusCode::OK)
                        .header(CONTENT_TYPE, "application/json")
                        .body(serde_json::to_vec(&oauth_server_metadata())?)
                        .map_err(|e| e.into());
                }
                _ => {}
            }

            let mut headers = request.headers().clone();
            let Some(authorization) = headers
                .remove("authorization")
                .and_then(|value| value.to_str().map(String::from).ok())
            else {
                let response = if request.uri().path() == "/token" {
                    let parameters =
                        serde_html_form::from_bytes::<RefreshRequestParameters>(request.body())?;
                    if parameters.refresh_token == "refreshtoken" {
                        Response::builder()
                            .status(StatusCode::OK)
                            .header(CONTENT_TYPE, "application/json")
                            .body(serde_json::to_vec(&OAuthTokenResponse {
                                access_token: String::from("new_accesstoken"),
                                token_type: OAuthTokenType::DPoP,
                                expires_in: Some(10),
                                refresh_token: Some(String::from("new_refreshtoken")),
                                scope: None,
                                sub: None,
                            })?)?
                    } else {
                        Response::builder()
                            .status(StatusCode::UNAUTHORIZED)
                            .header("WWW-Authenticate", "DPoP error=\"invalid_token\"")
                            .body(Vec::new())?
                    }
                } else {
                    Response::builder().status(StatusCode::UNAUTHORIZED).body(Vec::new())?
                };
                return Ok(response);
            };
            let Some(token) = authorization.strip_prefix("DPoP ") else {
                panic!("authorization header should start with DPoP");
            };
            if token == "expired" {
                return Ok(Response::builder()
                    .status(StatusCode::UNAUTHORIZED)
                    .header("WWW-Authenticate", "DPoP error=\"invalid_token\"")
                    .body(Vec::new())?);
            }
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

    struct MockDidResolver;

    impl Resolver for MockDidResolver {
        type Input = Did;
        type Output = DidDocument;
        type Error = atrium_identity::Error;
        async fn resolve(&self, did: &Self::Input) -> Result<Self::Output, Self::Error> {
            Ok(DidDocument {
                context: None,
                id: did.as_ref().to_string(),
                also_known_as: None,
                verification_method: None,
                service: Some(vec![DidDocumentService {
                    id: String::from("#atproto_pds"),
                    r#type: String::from("AtprotoPersonalDataServer"),
                    service_endpoint: String::from("https://aud.example.com"),
                }]),
            })
        }
    }

    impl DidResolver for MockDidResolver {}

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

    struct MockSessionStore {
        data: Arc<Mutex<HashMap<Did, Session>>>,
    }

    impl Store<Did, Session> for MockSessionStore {
        type Error = Error;

        async fn get(&self, key: &Did) -> Result<Option<Session>, Self::Error> {
            Ok(self.data.lock().await.get(key).cloned())
        }
        async fn set(&self, key: Did, value: Session) -> Result<(), Self::Error> {
            self.data.lock().await.insert(key, value);
            Ok(())
        }
        async fn del(&self, _: &Did) -> Result<(), Self::Error> {
            unimplemented!()
        }
        async fn clear(&self) -> Result<(), Self::Error> {
            unimplemented!()
        }
    }

    impl SessionStore for MockSessionStore {}

    fn oauth_server_metadata() -> OAuthAuthorizationServerMetadata {
        OAuthAuthorizationServerMetadata {
            issuer: String::from("https://iss.example.com"),
            token_endpoint: String::from("https://iss.example.com/token"),
            token_endpoint_auth_methods_supported: Some(vec![
                String::from("none"),
                String::from("private_key_jwt"),
            ]),
            ..Default::default()
        }
    }

    async fn oauth_session(
        data: Arc<Mutex<Option<RecordData>>>,
        store: Arc<Mutex<HashMap<Did, Session>>>,
    ) -> OAuthSession<MockHttpClient, MockDidResolver, NoopHandleResolver, MockSessionStore> {
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
                did_resolver: MockDidResolver,
                handle_resolver: NoopHandleResolver,
                authorization_server_metadata: Default::default(),
                protected_resource_metadata: Default::default(),
            },
            Arc::clone(&http_client),
        ));
        let keyset = None;
        let server_agent = OAuthServerAgent::new(
            dpop_key.clone(),
            oauth_server_metadata(),
            AtprotoLocalhostClientMetadata::default()
                .try_into_client_metadata(&None)
                .expect("client metadata should be valid"),
            resolver,
            Arc::clone(&http_client),
            keyset,
        )
        .expect("failed to create server agent");
        let sub = Did::new(String::from("did:fake:sub.test")).expect("did should be valid");
        let token_set = TokenSet {
            iss: String::from("https://iss.example.com"),
            sub: sub.clone(),
            aud: String::from("https://aud.example.com"),
            scope: None,
            refresh_token: Some(String::from("refreshtoken")),
            access_token: String::from("accesstoken"),
            token_type: OAuthTokenType::DPoP,
            expires_at: None,
        };
        let session = SessionHandle::new(
            Session { token_set, dpop_key },
            Arc::new(MockSessionStore { data: Arc::clone(&store) }),
            sub.clone(),
        );
        store.lock().await.insert(sub, session.read().await.clone());
        OAuthSession::new(server_agent, http_client, session)
            .await
            .expect("failed to create oauth session")
    }

    async fn oauth_agent(
        data: Arc<Mutex<Option<RecordData>>>,
    ) -> Agent<impl SessionManager + Configure + CloneWithProxy> {
        Agent::new(oauth_session(data, Default::default()).await)
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
        let agent = oauth_agent(Default::default()).await;
        assert_eq!(agent.did().await.as_deref(), Some("did:fake:sub.test"));
        Ok(())
    }

    #[tokio::test]
    async fn test_configure_endpoint() -> Result<(), Box<dyn std::error::Error>> {
        let data = Default::default();
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
        let data = Default::default();
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
        let oauth_session = oauth_session(Default::default(), Default::default()).await;
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

    #[tokio::test]
    async fn test_xrpc_with_refresh() -> Result<(), Box<dyn std::error::Error>> {
        let session_data = Default::default();
        let oauth_session = oauth_session(Default::default(), Arc::clone(&session_data)).await;
        oauth_session.store.set("expired".into()).await?;
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
        match result {
            Ok(output) => {
                assert_eq!(output.token, "fake_token");
            }
            Err(err) => {
                panic!("unexpected error: {err:?}");
            }
        }
        // wait for async update
        tokio::time::sleep(Duration::from_micros(0)).await;
        {
            let token_set = session_data
                .lock()
                .await
                .get(&Did::new(String::from("did:fake:sub.test"))?)
                .expect("session should be present")
                .token_set
                .clone();
            assert_eq!(token_set.access_token, "new_accesstoken");
            assert_eq!(token_set.refresh_token, Some(String::from("new_refreshtoken")));
        }
        Ok(())
    }

    #[tokio::test]
    async fn test_xrpc_with_duplicated_refresh() -> Result<(), Box<dyn std::error::Error>> {
        let session_data = Default::default();
        let oauth_session = oauth_session(Default::default(), Arc::clone(&session_data)).await;
        oauth_session.store.set("expired".into()).await?;
        let agent = Arc::new(Agent::new(oauth_session));

        let handles = (0..3).map(|_| {
            let agent = Arc::clone(&agent);
            tokio::spawn(async move {
                agent
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
                    .await
            })
        });
        let results = futures::future::join_all(handles).await;
        for result in results {
            match result? {
                Ok(output) => {
                    assert_eq!(output.token, "fake_token");
                }
                Err(err) => {
                    panic!("unexpected error: {err:?}");
                }
            }
        }
        Ok(())
    }
}
