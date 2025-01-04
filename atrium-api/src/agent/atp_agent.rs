//! Implementation of [`AtpAgent`] and definitions of [`SessionStore`] for it.

mod inner;
pub mod store;

use self::store::AtpSessionStore;
use super::{inner::Wrapper, Agent, CloneWithProxy, Configure, InnerStore, SessionManager};
use crate::{
    client::com::atproto::Service,
    did_doc::DidDocument,
    types::{string::Did, TryFromUnknown},
};
use atrium_xrpc::{Error, HttpClient, OutputDataOrBytes, XrpcClient, XrpcRequest};
use http::{Request, Response};
use serde::{de::DeserializeOwned, Serialize};
use std::{convert, fmt::Debug, ops::Deref, sync::Arc};

/// Type alias for the [com::atproto::server::create_session::Output](crate::com::atproto::server::create_session::Output)
pub type AtpSession = crate::com::atproto::server::create_session::Output;

pub struct CredentialSession<S, T>
where
    S: AtpSessionStore + Send + Sync,
    T: XrpcClient + Send + Sync,
    S::Error: std::error::Error + Send + Sync + 'static,
{
    store: Arc<InnerStore<S, AtpSession>>,
    inner: Arc<inner::Client<S, T>>,
    atproto_service: Service<inner::Client<S, T>>,
}

impl<S, T> CredentialSession<S, T>
where
    S: AtpSessionStore + Send + Sync,
    T: XrpcClient + Send + Sync,
    S::Error: std::error::Error + Send + Sync + 'static,
{
    pub fn new(xrpc: T, store: S) -> Self {
        let store = Arc::new(InnerStore::new(store, xrpc.base_uri()));
        let inner = Arc::new(inner::Client::new(Arc::clone(&store), xrpc));
        let atproto_service = Service::new(Arc::clone(&inner));
        Self { store, inner, atproto_service }
    }
    /// Start a new session with this agent.
    pub async fn login(
        &self,
        identifier: impl AsRef<str>,
        password: impl AsRef<str>,
    ) -> Result<AtpSession, Error<crate::com::atproto::server::create_session::Error>> {
        let result = self
            .atproto_service
            .server
            .create_session(
                crate::com::atproto::server::create_session::InputData {
                    auth_factor_token: None,
                    identifier: identifier.as_ref().into(),
                    password: password.as_ref().into(),
                }
                .into(),
            )
            .await?;
        self.store.set(result.clone()).await.ok();
        if let Some(did_doc) = result
            .did_doc
            .as_ref()
            .and_then(|value| DidDocument::try_from_unknown(value.clone()).ok())
        {
            self.store.update_endpoint(&did_doc);
        }
        Ok(result)
    }
    /// Resume a pre-existing session with this agent.
    pub async fn resume_session(
        &self,
        session: AtpSession,
    ) -> Result<(), Error<crate::com::atproto::server::get_session::Error>> {
        self.store.set(session.clone()).await.ok();
        let result = self.atproto_service.server.get_session().await;
        match result {
            Ok(output) => {
                assert_eq!(output.data.did, session.data.did);
                if let Ok(Some(mut session)) = self.store.get().await {
                    session.did_doc = output.data.did_doc.clone();
                    session.email = output.data.email;
                    session.email_confirmed = output.data.email_confirmed;
                    session.handle = output.data.handle;
                    self.store.set(session).await.ok();
                }
                if let Some(did_doc) = output
                    .data
                    .did_doc
                    .as_ref()
                    .and_then(|value| DidDocument::try_from_unknown(value.clone()).ok())
                {
                    self.store.update_endpoint(&did_doc);
                }
                Ok(())
            }
            Err(err) => {
                self.store.clear().await.ok();
                Err(err)
            }
        }
    }
    /// Get the current session.
    pub async fn get_session(&self) -> Option<AtpSession> {
        self.store.get().await.ok().and_then(convert::identity)
    }
    /// Get the current endpoint.
    pub async fn get_endpoint(&self) -> String {
        self.store.get_endpoint()
    }
    /// Get the current labelers header.
    pub async fn get_labelers_header(&self) -> Option<Vec<String>> {
        self.inner.get_labelers_header().await
    }
    /// Get the current proxy header.
    pub async fn get_proxy_header(&self) -> Option<String> {
        self.inner.get_proxy_header().await
    }
}

impl<S, T> HttpClient for CredentialSession<S, T>
where
    S: AtpSessionStore + Send + Sync,
    T: XrpcClient + Send + Sync,
    S::Error: std::error::Error + Send + Sync + 'static,
{
    async fn send_http(
        &self,
        request: Request<Vec<u8>>,
    ) -> Result<Response<Vec<u8>>, Box<dyn std::error::Error + Send + Sync + 'static>> {
        self.inner.send_http(request).await
    }
}

impl<S, T> XrpcClient for CredentialSession<S, T>
where
    S: AtpSessionStore + Send + Sync,
    T: XrpcClient + Send + Sync,
    S::Error: std::error::Error + Send + Sync + 'static,
{
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

impl<S, T> SessionManager for CredentialSession<S, T>
where
    S: AtpSessionStore + Send + Sync,
    T: XrpcClient + Send + Sync,
    S::Error: std::error::Error + Send + Sync + 'static,
{
    async fn did(&self) -> Option<Did> {
        self.store.get().await.ok().and_then(|session| session.map(|session| session.data.did))
    }
}

impl<S, T> Configure for CredentialSession<S, T>
where
    S: AtpSessionStore + Send + Sync,
    T: XrpcClient + Send + Sync,
    S::Error: std::error::Error + Send + Sync + 'static,
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

impl<S, T> CloneWithProxy for CredentialSession<S, T>
where
    S: AtpSessionStore + Send + Sync,
    S::Error: std::error::Error + Send + Sync + 'static,
    T: XrpcClient + Send + Sync,
{
    fn clone_with_proxy(&self, did: Did, service_type: impl AsRef<str>) -> Self {
        let inner = Arc::new(self.inner.clone_with_proxy(did, service_type));
        let atproto_service = Service::new(Arc::clone(&inner));
        Self { store: Arc::clone(&self.store), inner, atproto_service }
    }
}

/// An ATP "Agent".
/// Manages session token lifecycles and provides convenience methods.
///
/// This will be deprecated in the near future. Use [`Agent`] directly
/// with a [`CredentialSession`] instead:
/// ```
/// use atrium_api::agent::atp_agent::{store::MemorySessionStore, CredentialSession};
/// use atrium_api::agent::Agent;
/// use atrium_xrpc_client::reqwest::ReqwestClient;
///
/// let session = CredentialSession::new(
///     ReqwestClient::new("https://bsky.social"),
///     MemorySessionStore::default(),
/// );
/// let agent = Agent::new(session);
/// ```
pub struct AtpAgent<S, T>
where
    S: AtpSessionStore + Send + Sync,
    T: XrpcClient + Send + Sync,
    S::Error: std::error::Error + Send + Sync + 'static,
{
    session_manager: Wrapper<CredentialSession<S, T>>,
    inner: Agent<Wrapper<CredentialSession<S, T>>>,
}

impl<S, T> AtpAgent<S, T>
where
    S: AtpSessionStore + Send + Sync,
    T: XrpcClient + Send + Sync,
    S::Error: std::error::Error + Send + Sync + 'static,
{
    /// Create a new agent.
    pub fn new(xrpc: T, store: S) -> Self {
        let session_manager = Wrapper::new(CredentialSession::new(xrpc, store));
        let inner = Agent::new(session_manager.clone());
        Self { session_manager, inner }
    }
    /// Start a new session with this agent.
    pub async fn login(
        &self,
        identifier: impl AsRef<str>,
        password: impl AsRef<str>,
    ) -> Result<AtpSession, Error<crate::com::atproto::server::create_session::Error>> {
        self.session_manager.login(identifier, password).await
    }
    // /// Resume a pre-existing session with this agent.
    pub async fn resume_session(
        &self,
        session: AtpSession,
    ) -> Result<(), Error<crate::com::atproto::server::get_session::Error>> {
        self.session_manager.resume_session(session).await
    }
    /// Get the current session.
    pub async fn get_session(&self) -> Option<AtpSession> {
        self.session_manager.get_session().await
    }
    /// Get the current endpoint.
    pub async fn get_endpoint(&self) -> String {
        self.session_manager.get_endpoint().await
    }
    /// Get the current labelers header.
    pub async fn get_labelers_header(&self) -> Option<Vec<String>> {
        self.session_manager.get_labelers_header().await
    }
    /// Get the current proxy header.
    pub async fn get_proxy_header(&self) -> Option<String> {
        self.session_manager.get_proxy_header().await
    }
}

impl<S, T> Deref for AtpAgent<S, T>
where
    S: AtpSessionStore + Send + Sync,
    T: XrpcClient + Send + Sync,
    S::Error: std::error::Error + Send + Sync + 'static,
{
    type Target = Agent<Wrapper<CredentialSession<S, T>>>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

#[cfg(test)]
mod tests {
    use super::store::MemorySessionStore;
    use super::*;
    use crate::{
        agent::AtprotoServiceType,
        com::atproto::server::create_session::OutputData,
        did_doc::{DidDocument, Service, VerificationMethod},
        types::TryIntoUnknown,
    };
    use atrium_xrpc::HttpClient;
    use http::{HeaderMap, HeaderName, HeaderValue, Request, Response};
    use std::collections::HashMap;
    use tokio::sync::RwLock;
    #[cfg(target_arch = "wasm32")]
    use wasm_bindgen_test::wasm_bindgen_test;

    #[derive(Default)]
    struct MockResponses {
        create_session: Option<crate::com::atproto::server::create_session::OutputData>,
        get_session: Option<crate::com::atproto::server::get_session::OutputData>,
    }

    #[derive(Default)]
    struct MockClient {
        responses: MockResponses,
        counts: Arc<RwLock<HashMap<String, usize>>>,
        headers: Arc<RwLock<Vec<HeaderMap<HeaderValue>>>>,
    }

    impl HttpClient for MockClient {
        async fn send_http(
            &self,
            request: Request<Vec<u8>>,
        ) -> Result<Response<Vec<u8>>, Box<dyn std::error::Error + Send + Sync + 'static>> {
            #[cfg(not(target_arch = "wasm32"))]
            tokio::time::sleep(std::time::Duration::from_micros(10)).await;

            self.headers.write().await.push(request.headers().clone());
            let builder =
                Response::builder().header(http::header::CONTENT_TYPE, "application/json");
            let token = request
                .headers()
                .get(http::header::AUTHORIZATION)
                .and_then(|value| value.to_str().ok())
                .and_then(|value| value.split(' ').last());
            if token == Some("expired") {
                return Ok(builder.status(http::StatusCode::BAD_REQUEST).body(
                    serde_json::to_vec(&atrium_xrpc::error::ErrorResponseBody {
                        error: Some(String::from("ExpiredToken")),
                        message: Some(String::from("Token has expired")),
                    })?,
                )?);
            }
            let mut body = Vec::new();
            if let Some(nsid) = request.uri().path().strip_prefix("/xrpc/") {
                *self.counts.write().await.entry(nsid.into()).or_default() += 1;
                match nsid {
                    crate::com::atproto::server::create_session::NSID => {
                        if let Some(output) = &self.responses.create_session {
                            body.extend(serde_json::to_vec(output)?);
                        }
                    }
                    crate::com::atproto::server::get_session::NSID => {
                        if token == Some("access") {
                            if let Some(output) = &self.responses.get_session {
                                body.extend(serde_json::to_vec(output)?);
                            }
                        }
                    }
                    crate::com::atproto::server::refresh_session::NSID => {
                        if token == Some("refresh") {
                            body.extend(serde_json::to_vec(
                                &crate::com::atproto::server::refresh_session::OutputData {
                                    access_jwt: String::from("access"),
                                    active: None,
                                    did: "did:web:example.com".parse().expect("valid"),
                                    did_doc: None,
                                    handle: "example.com".parse().expect("valid"),
                                    refresh_jwt: String::from("refresh"),
                                    status: None,
                                },
                            )?);
                        }
                    }
                    crate::com::atproto::server::describe_server::NSID => {
                        body.extend(serde_json::to_vec(
                            &crate::com::atproto::server::describe_server::OutputData {
                                available_user_domains: Vec::new(),
                                contact: None,
                                did: "did:web:example.com".parse().expect("valid"),
                                invite_code_required: None,
                                links: None,
                                phone_verification_required: None,
                            },
                        )?);
                    }
                    _ => {}
                }
            }
            if body.is_empty() {
                Ok(builder.status(http::StatusCode::UNAUTHORIZED).body(serde_json::to_vec(
                    &atrium_xrpc::error::ErrorResponseBody {
                        error: Some(String::from("AuthenticationRequired")),
                        message: Some(String::from("Invalid identifier or password")),
                    },
                )?)?)
            } else {
                Ok(builder.status(http::StatusCode::OK).body(body)?)
            }
        }
    }

    impl XrpcClient for MockClient {
        fn base_uri(&self) -> String {
            "http://localhost:8080".into()
        }
    }

    fn session_data() -> OutputData {
        OutputData {
            access_jwt: String::from("access"),
            active: None,
            did: "did:web:example.com".parse().expect("valid"),
            did_doc: None,
            email: None,
            email_auth_factor: None,
            email_confirmed: None,
            handle: "example.com".parse().expect("valid"),
            refresh_jwt: String::from("refresh"),
            status: None,
        }
    }

    #[tokio::test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    async fn test_new() {
        let agent = AtpAgent::new(MockClient::default(), MemorySessionStore::default());
        assert_eq!(agent.get_session().await, None);
    }

    #[tokio::test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    async fn test_login() {
        let session_data = session_data();
        // success
        {
            let client = MockClient {
                responses: MockResponses {
                    create_session: Some(crate::com::atproto::server::create_session::OutputData {
                        ..session_data.clone()
                    }),
                    ..Default::default()
                },
                ..Default::default()
            };
            let agent = AtpAgent::new(client, MemorySessionStore::default());
            agent.login("test", "pass").await.expect("login should be succeeded");
            assert_eq!(agent.get_session().await, Some(session_data.into()));
        }
        // failure with `createSession` error
        {
            let client = MockClient {
                responses: MockResponses { ..Default::default() },
                ..Default::default()
            };
            let agent = AtpAgent::new(client, MemorySessionStore::default());
            agent.login("test", "bad").await.expect_err("login should be failed");
            assert_eq!(agent.get_session().await, None);
        }
    }

    #[tokio::test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    async fn test_xrpc_get_session() {
        let session_data = session_data();
        let client = MockClient {
            responses: MockResponses {
                get_session: Some(crate::com::atproto::server::get_session::OutputData {
                    active: session_data.active,
                    did: session_data.did.clone(),
                    did_doc: session_data.did_doc.clone(),
                    email: session_data.email.clone(),
                    email_auth_factor: session_data.email_auth_factor,
                    email_confirmed: session_data.email_confirmed,
                    handle: session_data.handle.clone(),
                    status: session_data.status.clone(),
                }),
                ..Default::default()
            },
            ..Default::default()
        };
        let agent = AtpAgent::new(client, MemorySessionStore::default());
        agent
            .session_manager
            .store
            .set(session_data.clone().into())
            .await
            .expect("set session should be succeeded");
        let output = agent
            .api
            .com
            .atproto
            .server
            .get_session()
            .await
            .expect("get session should be succeeded");
        assert_eq!(output.did.as_str(), "did:web:example.com");
    }

    #[tokio::test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    async fn test_xrpc_get_session_with_refresh() {
        let mut session_data = session_data();
        session_data.access_jwt = String::from("expired");
        let client = MockClient {
            responses: MockResponses {
                get_session: Some(crate::com::atproto::server::get_session::OutputData {
                    active: session_data.active,
                    did: session_data.did.clone(),
                    did_doc: session_data.did_doc.clone(),
                    email: session_data.email.clone(),
                    email_auth_factor: session_data.email_auth_factor,
                    email_confirmed: session_data.email_confirmed,
                    handle: session_data.handle.clone(),
                    status: session_data.status.clone(),
                }),
                ..Default::default()
            },
            ..Default::default()
        };
        let agent = AtpAgent::new(client, MemorySessionStore::default());
        agent
            .session_manager
            .store
            .set(session_data.clone().into())
            .await
            .expect("set session should be succeeded");
        let output = agent
            .api
            .com
            .atproto
            .server
            .get_session()
            .await
            .expect("get session should be succeeded");
        assert_eq!(output.did.as_str(), "did:web:example.com");
        assert_eq!(
            agent
                .session_manager
                .store
                .get()
                .await
                .expect("session should be stored")
                .map(|session| session.data.access_jwt),
            Some("access".into())
        );
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[tokio::test]
    async fn test_xrpc_get_session_with_duplicated_refresh() {
        let mut session_data = session_data();
        session_data.access_jwt = String::from("expired");
        let client = MockClient {
            responses: MockResponses {
                get_session: Some(crate::com::atproto::server::get_session::OutputData {
                    active: session_data.active,
                    did: session_data.did.clone(),
                    did_doc: session_data.did_doc.clone(),
                    email: session_data.email.clone(),
                    email_auth_factor: session_data.email_auth_factor,
                    email_confirmed: session_data.email_confirmed,
                    handle: session_data.handle.clone(),
                    status: session_data.status.clone(),
                }),
                ..Default::default()
            },
            ..Default::default()
        };
        let counts = Arc::clone(&client.counts);
        let agent = Arc::new(AtpAgent::new(client, MemorySessionStore::default()));
        agent
            .session_manager
            .store
            .set(session_data.clone().into())
            .await
            .expect("set session should be succeeded");
        let handles = (0..3).map(|_| {
            let agent = Arc::clone(&agent);
            tokio::spawn(async move { agent.api.com.atproto.server.get_session().await })
        });
        let results = futures::future::join_all(handles).await;
        for result in &results {
            let output = result
                .as_ref()
                .expect("task should be successfully executed")
                .as_ref()
                .expect("get session should be succeeded");
            assert_eq!(output.did.as_str(), "did:web:example.com");
        }
        assert_eq!(
            agent
                .session_manager
                .store
                .get()
                .await
                .expect("session should be stored")
                .map(|session| session.data.access_jwt),
            Some("access".into())
        );
        assert_eq!(
            counts.read().await.clone(),
            HashMap::from_iter([
                ("com.atproto.server.refreshSession".into(), 1),
                ("com.atproto.server.getSession".into(), 3)
            ])
        );
    }

    #[tokio::test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    async fn test_resume_session() {
        let session_data = session_data();
        // success
        {
            let client = MockClient {
                responses: MockResponses {
                    get_session: Some(crate::com::atproto::server::get_session::OutputData {
                        active: session_data.active,
                        did: session_data.did.clone(),
                        did_doc: session_data.did_doc.clone(),
                        email: session_data.email.clone(),
                        email_auth_factor: session_data.email_auth_factor,
                        email_confirmed: session_data.email_confirmed,
                        handle: session_data.handle.clone(),
                        status: session_data.status.clone(),
                    }),
                    ..Default::default()
                },
                ..Default::default()
            };
            let agent = AtpAgent::new(client, MemorySessionStore::default());
            assert_eq!(agent.get_session().await, None);
            agent
                .resume_session(
                    OutputData {
                        email: Some(String::from("test@example.com")),
                        ..session_data.clone()
                    }
                    .into(),
                )
                .await
                .expect("resume_session should be succeeded");
            assert_eq!(agent.get_session().await, Some(session_data.clone().into()));
        }
        // failure with `getSession` error
        {
            let client = MockClient {
                responses: MockResponses { ..Default::default() },
                ..Default::default()
            };
            let agent = AtpAgent::new(client, MemorySessionStore::default());
            assert_eq!(agent.get_session().await, None);
            agent
                .resume_session(session_data.clone().into())
                .await
                .expect_err("resume_session should be failed");
            assert_eq!(agent.get_session().await, None);
        }
    }

    #[tokio::test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    async fn test_resume_session_with_refresh() {
        let session_data = session_data();
        let client = MockClient {
            responses: MockResponses {
                get_session: Some(crate::com::atproto::server::get_session::OutputData {
                    active: session_data.active,
                    did: session_data.did.clone(),
                    did_doc: session_data.did_doc.clone(),
                    email: session_data.email.clone(),
                    email_auth_factor: session_data.email_auth_factor,
                    email_confirmed: session_data.email_confirmed,
                    handle: session_data.handle.clone(),
                    status: session_data.status.clone(),
                }),
                ..Default::default()
            },
            ..Default::default()
        };
        let agent = AtpAgent::new(client, MemorySessionStore::default());
        agent
            .resume_session(
                OutputData { access_jwt: "expired".into(), ..session_data.clone() }.into(),
            )
            .await
            .expect("resume_session should be succeeded");
        assert_eq!(agent.get_session().await, Some(session_data.clone().into()));
    }

    #[tokio::test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    async fn test_login_with_diddoc() {
        let session_data = session_data();
        let did_doc = DidDocument {
            context: None,
            id: "did:plc:ewvi7nxzyoun6zhxrhs64oiz".into(),
            also_known_as: Some(vec!["at://atproto.com".into()]),
            verification_method: Some(vec![VerificationMethod {
                id: "did:plc:ewvi7nxzyoun6zhxrhs64oiz#atproto".into(),
                r#type: "Multikey".into(),
                controller: "did:plc:ewvi7nxzyoun6zhxrhs64oiz".into(),
                public_key_multibase: Some(
                    "zQ3shXjHeiBuRCKmM36cuYnm7YEMzhGnCmCyW92sRJ9pribSF".into(),
                ),
            }]),
            service: Some(vec![Service {
                id: "#atproto_pds".into(),
                r#type: "AtprotoPersonalDataServer".into(),
                service_endpoint: "https://bsky.social".into(),
            }]),
        };
        // success
        {
            let client = MockClient {
                responses: MockResponses {
                    create_session: Some(crate::com::atproto::server::create_session::OutputData {
                        did_doc: Some(
                            did_doc
                                .clone()
                                .try_into_unknown()
                                .expect("failed to convert to unknown"),
                        ),
                        ..session_data.clone()
                    }),
                    ..Default::default()
                },
                ..Default::default()
            };
            let agent = AtpAgent::new(client, MemorySessionStore::default());
            agent.login("test", "pass").await.expect("login should be succeeded");
            assert_eq!(agent.get_endpoint().await, "https://bsky.social");
            assert_eq!(agent.api.com.atproto.server.xrpc.base_uri(), "https://bsky.social");
        }
        // invalid services
        {
            let client = MockClient {
                responses: MockResponses {
                    create_session: Some(crate::com::atproto::server::create_session::OutputData {
                        did_doc: Some(
                            DidDocument {
                                service: Some(vec![
                                    Service {
                                        id: "#pds".into(), // not `#atproto_pds`
                                        r#type: "AtprotoPersonalDataServer".into(),
                                        service_endpoint: "https://bsky.social".into(),
                                    },
                                    Service {
                                        id: "#atproto_pds".into(),
                                        r#type: "AtprotoPersonalDataServer".into(),
                                        service_endpoint: "htps://bsky.social".into(), // invalid url (not `https`)
                                    },
                                ]),
                                ..did_doc.clone()
                            }
                            .try_into_unknown()
                            .expect("failed to convert to unknown"),
                        ),
                        ..session_data.clone()
                    }),
                    ..Default::default()
                },
                ..Default::default()
            };
            let agent = AtpAgent::new(client, MemorySessionStore::default());
            agent.login("test", "pass").await.expect("login should be succeeded");
            // not updated
            assert_eq!(agent.get_endpoint().await, "http://localhost:8080");
            assert_eq!(agent.api.com.atproto.server.xrpc.base_uri(), "http://localhost:8080");
        }
    }

    #[tokio::test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    async fn test_configure_labelers_header() {
        let client = MockClient::default();
        let headers = Arc::clone(&client.headers);
        let agent = AtpAgent::new(client, MemorySessionStore::default());

        agent
            .api
            .com
            .atproto
            .server
            .describe_server()
            .await
            .expect("describe_server should be succeeded");
        assert_eq!(headers.read().await.last(), Some(&HeaderMap::new()));

        agent.configure_labelers_header(Some(vec![(
            "did:plc:test1".parse().expect("did should be valid"),
            false,
        )]));
        agent
            .api
            .com
            .atproto
            .server
            .describe_server()
            .await
            .expect("describe_server should be succeeded");
        assert_eq!(
            headers.read().await.last(),
            Some(&HeaderMap::from_iter([(
                HeaderName::from_static("atproto-accept-labelers"),
                HeaderValue::from_static("did:plc:test1"),
            )]))
        );

        agent.configure_labelers_header(Some(vec![
            ("did:plc:test1".parse().expect("did should be valid"), true),
            ("did:plc:test2".parse().expect("did should be valid"), false),
        ]));
        agent
            .api
            .com
            .atproto
            .server
            .describe_server()
            .await
            .expect("describe_server should be succeeded");
        assert_eq!(
            headers.read().await.last(),
            Some(&HeaderMap::from_iter([(
                HeaderName::from_static("atproto-accept-labelers"),
                HeaderValue::from_static("did:plc:test1;redact, did:plc:test2"),
            )]))
        );

        assert_eq!(
            agent.get_labelers_header().await,
            Some(vec![String::from("did:plc:test1;redact"), String::from("did:plc:test2")])
        );
    }

    #[tokio::test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    async fn test_configure_proxy_header() {
        let client = MockClient::default();
        let headers = Arc::clone(&client.headers);
        let agent = AtpAgent::new(client, MemorySessionStore::default());

        agent
            .api
            .com
            .atproto
            .server
            .describe_server()
            .await
            .expect("describe_server should be succeeded");
        assert_eq!(headers.read().await.last(), Some(&HeaderMap::new()));

        agent.configure_proxy_header(
            "did:plc:test1".parse().expect("did should be valid"),
            AtprotoServiceType::AtprotoLabeler,
        );
        agent
            .api
            .com
            .atproto
            .server
            .describe_server()
            .await
            .expect("describe_server should be succeeded");
        assert_eq!(
            headers.read().await.last(),
            Some(&HeaderMap::from_iter([(
                HeaderName::from_static("atproto-proxy"),
                HeaderValue::from_static("did:plc:test1#atproto_labeler"),
            ),]))
        );

        agent.configure_proxy_header(
            "did:plc:test1".parse().expect("did should be valid"),
            "atproto_labeler",
        );
        agent
            .api
            .com
            .atproto
            .server
            .describe_server()
            .await
            .expect("describe_server should be succeeded");
        assert_eq!(
            headers.read().await.last(),
            Some(&HeaderMap::from_iter([(
                HeaderName::from_static("atproto-proxy"),
                HeaderValue::from_static("did:plc:test1#atproto_labeler"),
            ),]))
        );

        agent
            .api_with_proxy(
                "did:plc:test2".parse().expect("did should be valid"),
                "atproto_labeler",
            )
            .com
            .atproto
            .server
            .describe_server()
            .await
            .expect("describe_server should be succeeded");
        assert_eq!(
            headers.read().await.last(),
            Some(&HeaderMap::from_iter([(
                HeaderName::from_static("atproto-proxy"),
                HeaderValue::from_static("did:plc:test2#atproto_labeler"),
            ),]))
        );

        agent
            .api
            .com
            .atproto
            .server
            .describe_server()
            .await
            .expect("describe_server should be succeeded");
        assert_eq!(
            headers.read().await.last(),
            Some(&HeaderMap::from_iter([(
                HeaderName::from_static("atproto-proxy"),
                HeaderValue::from_static("did:plc:test1#atproto_labeler"),
            ),]))
        );

        assert_eq!(
            agent.get_proxy_header().await,
            Some(String::from("did:plc:test1#atproto_labeler"))
        );
    }

    #[tokio::test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    async fn test_agent_did() {
        let session_data = session_data();
        let client = MockClient { responses: MockResponses::default(), ..Default::default() };
        let agent = AtpAgent::new(client, MemorySessionStore::default());
        assert_eq!(agent.did().await, None);
        agent
            .session_manager
            .store
            .set(session_data.clone().into())
            .await
            .expect("set session should be succeeded");
        assert_eq!(agent.did().await, Some(session_data.did));
    }
}
