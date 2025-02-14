pub mod atp_agent;
#[cfg(feature = "bluesky")]
pub mod bluesky;
mod inner;
mod session_manager;
pub mod utils;

pub use self::session_manager::SessionManager;
use crate::{client::Service, types::string::Did};
use atrium_xrpc::types::AuthorizationToken;
use std::{future::Future, sync::Arc};

#[cfg_attr(not(target_arch = "wasm32"), trait_variant::make(Send))]
pub trait AuthorizationProvider {
    #[allow(unused_variables)]
    fn authorization_token(
        &self,
        is_refresh: bool,
    ) -> impl Future<Output = Option<AuthorizationToken>>;
}

pub trait Configure {
    /// Set the current endpoint.
    fn configure_endpoint(&self, endpoint: String);
    /// Configures the moderation services to be applied on requests.
    fn configure_labelers_header(&self, labeler_dids: Option<Vec<(Did, bool)>>);
    /// Configures the atproto-proxy header to be applied on requests.
    fn configure_proxy_header(&self, did: Did, service_type: impl AsRef<str>);
}

pub trait CloneWithProxy {
    fn clone_with_proxy(&self, did: Did, service_type: impl AsRef<str>) -> Self;
}

/// Supported proxy targets.
#[cfg(feature = "bluesky")]
pub type AtprotoServiceType = self::bluesky::AtprotoServiceType;

#[cfg(not(feature = "bluesky"))]
pub enum AtprotoServiceType {
    AtprotoLabeler,
}

#[cfg(not(feature = "bluesky"))]
impl AsRef<str> for AtprotoServiceType {
    fn as_ref(&self) -> &str {
        match self {
            Self::AtprotoLabeler => "atproto_labeler",
        }
    }
}

pub struct Agent<M>
where
    M: SessionManager + Send + Sync,
{
    session_manager: Arc<inner::Wrapper<M>>,
    pub api: Service<inner::Wrapper<M>>,
}

impl<M> Agent<M>
where
    M: SessionManager + Send + Sync,
{
    pub fn new(session_manager: M) -> Self {
        let session_manager = Arc::new(inner::Wrapper::new(session_manager));
        let api = Service::new(session_manager.clone());
        Self { session_manager, api }
    }
    pub async fn did(&self) -> Option<Did> {
        self.session_manager.did().await
    }
}

impl<M> Agent<M>
where
    M: CloneWithProxy + SessionManager + Send + Sync,
{
    /// Configures the atproto-proxy header to be applied on requests.
    ///
    /// Returns a new client service with the proxy header configured.
    pub fn api_with_proxy(
        &self,
        did: Did,
        service_type: impl AsRef<str>,
    ) -> Service<inner::Wrapper<M>> {
        Service::new(Arc::new(self.session_manager.clone_with_proxy(did, service_type)))
    }
}

impl<M> Configure for Agent<M>
where
    M: Configure + SessionManager + Send + Sync,
{
    fn configure_endpoint(&self, endpoint: String) {
        self.session_manager.configure_endpoint(endpoint);
    }
    fn configure_labelers_header(&self, labeler_dids: Option<Vec<(Did, bool)>>) {
        self.session_manager.configure_labelers_header(labeler_dids);
    }
    fn configure_proxy_header(&self, did: Did, service_type: impl AsRef<str>) {
        self.session_manager.configure_proxy_header(did, service_type);
    }
}

#[cfg(test)]
mod tests {
    use super::inner::Wrapper;
    use super::utils::{SessionClient, SessionWithEndpointStore};
    use super::*;
    use atrium_common::store::Store;
    use atrium_xrpc::{Error, HttpClient, OutputDataOrBytes, XrpcClient, XrpcRequest};
    use http::{header::CONTENT_TYPE, HeaderMap, HeaderName, HeaderValue, Request, Response};
    use serde::{de::DeserializeOwned, Serialize};
    use std::fmt::Debug;
    use tokio::sync::Mutex;

    #[derive(Default)]
    struct RecordData {
        host: Option<String>,
        headers: HeaderMap<HeaderValue>,
    }

    struct MockClient {
        data: Arc<Mutex<Option<RecordData>>>,
    }

    impl HttpClient for MockClient {
        async fn send_http(
            &self,
            request: Request<Vec<u8>>,
        ) -> Result<Response<Vec<u8>>, Box<dyn std::error::Error + Send + Sync + 'static>> {
            self.data.lock().await.replace(RecordData {
                host: request.uri().host().map(String::from),
                headers: request.headers().clone(),
            });
            let output = crate::com::atproto::server::get_service_auth::OutputData {
                token: String::from("fake_token"),
            };
            Response::builder()
                .header(CONTENT_TYPE, "application/json")
                .body(serde_json::to_vec(&output)?)
                .map_err(|e| e.into())
        }
    }

    impl XrpcClient for MockClient {
        fn base_uri(&self) -> String {
            unimplemented!()
        }
    }

    #[derive(thiserror::Error, Debug)]
    enum MockStoreError {}

    struct MockStore;

    impl Store<(), ()> for MockStore {
        type Error = MockStoreError;

        async fn get(&self, _key: &()) -> Result<Option<()>, Self::Error> {
            unimplemented!()
        }
        async fn set(&self, _key: (), _value: ()) -> Result<(), Self::Error> {
            unimplemented!()
        }
        async fn del(&self, _key: &()) -> Result<(), Self::Error> {
            unimplemented!()
        }
        async fn clear(&self) -> Result<(), Self::Error> {
            unimplemented!()
        }
    }

    impl AuthorizationProvider for MockStore {
        async fn authorization_token(&self, _: bool) -> Option<AuthorizationToken> {
            None
        }
    }

    struct MockSessionManager {
        inner: SessionClient<MockStore, MockClient, ()>,
    }

    impl HttpClient for MockSessionManager {
        async fn send_http(
            &self,
            request: Request<Vec<u8>>,
        ) -> Result<Response<Vec<u8>>, Box<dyn std::error::Error + Send + Sync + 'static>> {
            self.inner.send_http(request).await
        }
    }

    impl XrpcClient for MockSessionManager {
        fn base_uri(&self) -> String {
            self.inner.base_uri()
        }
        async fn send_xrpc<P, I, O, E>(
            &self,
            request: &XrpcRequest<P, I>,
        ) -> Result<OutputDataOrBytes<O>, Error<E>>
        where
            P: Serialize + Send + Sync,
            I: Serialize + Send + Sync,
            O: DeserializeOwned + Send + Sync,
            E: DeserializeOwned + Send + Sync + Debug,
        {
            self.inner.send_xrpc(request).await
        }
    }

    impl SessionManager for MockSessionManager {
        async fn did(&self) -> Option<Did> {
            Did::new(String::from("did:fake:handle.test")).ok()
        }
    }

    impl Configure for MockSessionManager {
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

    impl CloneWithProxy for MockSessionManager {
        fn clone_with_proxy(&self, did: Did, service_type: impl AsRef<str>) -> Self {
            Self { inner: self.inner.clone_with_proxy(did, service_type) }
        }
    }

    fn agent(data: Arc<Mutex<Option<RecordData>>>) -> Agent<MockSessionManager> {
        let inner = SessionClient::new(
            Arc::new(SessionWithEndpointStore::new(
                MockStore {},
                String::from("https://example.com"),
            )),
            MockClient { data },
        );
        Agent::new(MockSessionManager { inner })
    }

    async fn call_service(
        service: &Service<Wrapper<MockSessionManager>>,
    ) -> Result<(), Error<crate::com::atproto::server::get_service_auth::Error>> {
        let output = service
            .com
            .atproto
            .server
            .get_service_auth(
                crate::com::atproto::server::get_service_auth::ParametersData {
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
        let agent = agent(Arc::new(Mutex::new(Default::default())));
        assert_eq!(agent.did().await, Some(Did::new(String::from("did:fake:handle.test"))?));
        Ok(())
    }

    #[tokio::test]
    async fn test_configure_endpoint() -> Result<(), Box<dyn std::error::Error>> {
        let data = Arc::new(Mutex::new(Default::default()));
        let agent = agent(data.clone());
        call_service(&agent.api).await?;
        assert_eq!(
            data.lock().await.as_ref().expect("data should be recorded").host.as_deref(),
            Some("example.com")
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
        let agent = agent(data.clone());
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
        let agent = agent(data.clone());
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
}
