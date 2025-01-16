pub mod atp_agent;
#[cfg(feature = "bluesky")]
pub mod bluesky;
mod inner;
mod session_manager;

pub use self::session_manager::SessionManager;
use crate::{client::Service, did_doc::DidDocument, types::string::Did};
use atrium_common::store::Store;
use atrium_xrpc::{types::AuthorizationToken, HttpClient, XrpcClient};
use http::{Request, Response};
use std::{
    future::Future,
    marker::PhantomData,
    sync::{Arc, RwLock},
};

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

pub struct WrapperClient<S, T, U> {
    store: Arc<InnerStore<S, U>>,
    proxy_header: RwLock<Option<String>>,
    labelers_header: Arc<RwLock<Option<Vec<String>>>>,
    inner: Arc<T>,
}

impl<S, T, U> WrapperClient<S, T, U> {
    pub fn new(store: Arc<InnerStore<S, U>>, xrpc: T) -> Self {
        Self {
            store: Arc::clone(&store),
            labelers_header: Arc::new(RwLock::new(None)),
            proxy_header: RwLock::new(None),
            inner: Arc::new(xrpc),
        }
    }
}

impl<S, T, U> Configure for WrapperClient<S, T, U> {
    fn configure_endpoint(&self, endpoint: String) {
        *self.store.endpoint.write().expect("failed to write endpoint") = endpoint;
    }
    fn configure_labelers_header(&self, labelers_dids: Option<Vec<(Did, bool)>>) {
        *self.labelers_header.write().expect("failed to write labelers header") =
            labelers_dids.map(|dids| {
                dids.iter()
                    .map(|(did, redact)| {
                        if *redact {
                            format!("{};redact", did.as_ref())
                        } else {
                            did.as_ref().into()
                        }
                    })
                    .collect()
            })
    }
    fn configure_proxy_header(&self, did: Did, service_type: impl AsRef<str>) {
        self.proxy_header.write().expect("failed to write proxy header").replace(format!(
            "{}#{}",
            did.as_ref(),
            service_type.as_ref()
        ));
    }
}

impl<S, T, U> CloneWithProxy for WrapperClient<S, T, U> {
    fn clone_with_proxy(&self, did: Did, service_type: impl AsRef<str>) -> Self {
        let cloned = self.clone();
        cloned.configure_proxy_header(did, service_type);
        cloned
    }
}

impl<S, T, U> Clone for WrapperClient<S, T, U> {
    fn clone(&self) -> Self {
        Self {
            store: self.store.clone(),
            labelers_header: self.labelers_header.clone(),
            proxy_header: RwLock::new(
                self.proxy_header.read().expect("failed to read proxy header").clone(),
            ),
            inner: self.inner.clone(),
        }
    }
}

impl<S, T, U> HttpClient for WrapperClient<S, T, U>
where
    S: Store<(), U> + Send + Sync,
    T: HttpClient + Send + Sync,
    U: Clone + Send + Sync,
{
    async fn send_http(
        &self,
        request: Request<Vec<u8>>,
    ) -> core::result::Result<Response<Vec<u8>>, Box<dyn std::error::Error + Send + Sync + 'static>>
    {
        self.inner.send_http(request).await
    }
}

impl<S, T, U> XrpcClient for WrapperClient<S, T, U>
where
    S: Store<(), U> + AuthorizationProvider + Send + Sync,
    T: HttpClient + Send + Sync,
    U: Clone + Send + Sync,
{
    fn base_uri(&self) -> String {
        self.store.get_endpoint()
    }
    async fn authorization_token(&self, is_refresh: bool) -> Option<AuthorizationToken> {
        self.store.authorization_token(is_refresh).await
    }
    async fn atproto_proxy_header(&self) -> Option<String> {
        self.proxy_header.read().expect("failed to read proxy header").clone()
    }
    async fn atproto_accept_labelers_header(&self) -> Option<Vec<String>> {
        self.labelers_header.read().expect("failed to read labelers header").clone()
    }
}

pub struct InnerStore<S, U> {
    inner: S,
    endpoint: RwLock<String>,
    _phantom: PhantomData<U>,
}

impl<S, U> InnerStore<S, U> {
    pub fn new(inner: S, initial_endpoint: String) -> Self {
        Self { inner, endpoint: RwLock::new(initial_endpoint), _phantom: PhantomData }
    }
    pub fn get_endpoint(&self) -> String {
        self.endpoint.read().expect("failed to read endpoint").clone()
    }
    pub fn update_endpoint(&self, did_doc: &DidDocument) {
        if let Some(endpoint) = did_doc.get_pds_endpoint() {
            *self.endpoint.write().expect("failed to write endpoint") = endpoint;
        }
    }
}

impl<S, U> InnerStore<S, U>
where
    S: Store<(), U>,
    U: Clone,
{
    pub async fn get(&self) -> Result<Option<U>, S::Error> {
        self.inner.get(&()).await
    }
    pub async fn set(&self, value: U) -> Result<(), S::Error> {
        self.inner.set((), value).await
    }
    pub async fn clear(&self) -> Result<(), S::Error> {
        self.inner.clear().await
    }
}

impl<S, U> AuthorizationProvider for InnerStore<S, U>
where
    S: Store<(), U> + AuthorizationProvider + Send + Sync,
    U: Clone + Send + Sync,
{
    async fn authorization_token(&self, is_refresh: bool) -> Option<AuthorizationToken> {
        self.inner.authorization_token(is_refresh).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use atrium_xrpc::{Error, HttpClient, OutputDataOrBytes, XrpcClient, XrpcRequest};
    use http::{header::CONTENT_TYPE, HeaderMap, HeaderName, HeaderValue, Request, Response};
    use inner::Wrapper;
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
        inner: WrapperClient<MockStore, MockClient, ()>,
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
        let inner = WrapperClient::new(
            Arc::new(InnerStore::new(MockStore {}, String::from("https://example.com"))),
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
