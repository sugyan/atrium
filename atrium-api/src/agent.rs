//! An ATP "Agent".
//! Manages session token lifecycles and provides all XRPC methods.
use crate::client_services::Service;
use async_trait::async_trait;
use atrium_xrpc::error::{Error, XrpcErrorKind};
use atrium_xrpc::{HttpClient, OutputDataOrBytes, XrpcClient, XrpcRequest, XrpcResult};
use http::{Method, Request, Response};
use serde::{de::DeserializeOwned, Serialize};
use std::sync::{Arc, RwLock};

/// Type alias for the [com::atproto::server::create_session::Output](crate::com::atproto::server::create_session::Output)
pub type Session = crate::com::atproto::server::create_session::Output;

pub struct AuthWrapper<T>
where
    T: XrpcClient + Send + Sync,
{
    session: Arc<RwLock<Option<Session>>>,
    inner: T,
}

#[async_trait]
impl<T> HttpClient for AuthWrapper<T>
where
    T: XrpcClient + Send + Sync,
{
    async fn send_http(
        &self,
        req: Request<Vec<u8>>,
    ) -> Result<Response<Vec<u8>>, Box<dyn std::error::Error + Send + Sync + 'static>> {
        self.inner.send_http(req).await
    }
}

impl<T> AuthWrapper<T>
where
    T: XrpcClient + Send + Sync,
{
    fn new(session: Arc<RwLock<Option<Session>>>, inner: T) -> Self {
        Self { session, inner }
    }
}

impl<T> XrpcClient for AuthWrapper<T>
where
    T: XrpcClient + Send + Sync,
{
    fn host(&self) -> &str {
        self.inner.host()
    }
    fn auth(&self, is_refresh: bool) -> Option<String> {
        self.session.read().ok().and_then(|lock| {
            lock.as_ref().map(|session| {
                if is_refresh {
                    session.refresh_jwt.clone()
                } else {
                    session.access_jwt.clone()
                }
            })
        })
    }
}

pub struct RefreshWrapper<T>
where
    T: XrpcClient + Send + Sync,
{
    #[allow(dead_code)]
    session: Arc<RwLock<Option<Session>>>,
    inner: T,
}

impl<T> RefreshWrapper<T>
where
    T: XrpcClient + Send + Sync,
{
    fn new(session: Arc<RwLock<Option<Session>>>, inner: T) -> Self {
        Self { session, inner }
    }
    async fn do_send<P, I, O, E>(&self, request: &XrpcRequest<P, I>) -> XrpcResult<O, E>
    where
        P: Serialize + Send + Sync,
        I: Serialize + Send + Sync,
        O: DeserializeOwned + Send + Sync,
        E: DeserializeOwned + Send + Sync,
    {
        self.inner.send_xrpc(request).await
    }
    async fn refresh_session(
        &self,
    ) -> Result<
        crate::com::atproto::server::refresh_session::Output,
        Error<crate::com::atproto::server::refresh_session::Error>,
    > {
        let response = self
            .inner
            .send_xrpc::<(), (), _, _>(&XrpcRequest {
                method: Method::POST,
                path: "com.atproto.server.refreshSession".into(),
                parameters: None,
                input: None,
                encoding: None,
            })
            .await?;
        match response {
            OutputDataOrBytes::Data(data) => Ok(data),
            _ => Err(Error::UnexpectedResponseType),
        }
    }
}

#[async_trait]
impl<T> HttpClient for RefreshWrapper<T>
where
    T: XrpcClient + Send + Sync,
{
    async fn send_http(
        &self,
        req: Request<Vec<u8>>,
    ) -> Result<Response<Vec<u8>>, Box<dyn std::error::Error + Send + Sync + 'static>> {
        self.inner.send_http(req).await
    }
}

#[async_trait]
impl<T> XrpcClient for RefreshWrapper<T>
where
    T: XrpcClient + Send + Sync,
{
    fn host(&self) -> &str {
        self.inner.host()
    }
    fn auth(&self, is_refresh: bool) -> Option<String> {
        self.inner.auth(is_refresh)
    }
    async fn send_xrpc<P, I, O, E>(&self, request: &XrpcRequest<P, I>) -> XrpcResult<O, E>
    where
        P: Serialize + Send + Sync,
        I: Serialize + Send + Sync,
        O: DeserializeOwned + Send + Sync,
        E: DeserializeOwned + Send + Sync,
    {
        let result = self.do_send(request).await;
        if let Err(Error::XrpcResponse(res)) = &result {
            if let Some(XrpcErrorKind::Undefined(body)) = &res.error {
                if let Some("ExpiredToken") = &body.error.as_deref() {
                    let result = self.refresh_session().await;
                    match result {
                        Ok(output) => {
                            let mut session = self
                                .session
                                .write()
                                .expect("write lock on session should not be poisoned");
                            let email = session.as_ref().and_then(|session| session.email.clone());
                            let email_confirmed =
                                session.as_ref().and_then(|session| session.email_confirmed);
                            session.replace(Session {
                                access_jwt: output.access_jwt,
                                did: output.did,
                                email,
                                email_confirmed,
                                handle: output.handle,
                                refresh_jwt: output.refresh_jwt,
                            });
                        }
                        Err(err) => {
                            return Err(Error::Other(err.to_string()));
                        }
                    }
                    return self.do_send(request).await;
                }
            }
        }
        result
    }
}

pub struct AtpAgent<T>
where
    T: XrpcClient + Send + Sync,
{
    pub api: Service<RefreshWrapper<AuthWrapper<T>>>,
    session: Arc<RwLock<Option<Session>>>,
}

impl<T> AtpAgent<T>
where
    T: XrpcClient + Send + Sync,
{
    pub fn new(xrpc: T) -> Self {
        let session = Arc::new(RwLock::new(None));
        let api = Service::new(Arc::new(RefreshWrapper::new(
            Arc::clone(&session),
            AuthWrapper::new(Arc::clone(&session), xrpc),
        )));
        Self { api, session }
    }
    pub fn get_session(&self) -> Option<Session> {
        self.session
            .read()
            .expect("read lock on session should not be poisoned")
            .clone()
    }
    /// Start a new session with this agent.
    pub async fn login(
        &self,
        identifier: &str,
        password: &str,
    ) -> Result<Session, Error<crate::com::atproto::server::create_session::Error>> {
        let result = self
            .api
            .com
            .atproto
            .server
            .create_session(crate::com::atproto::server::create_session::Input {
                identifier: identifier.into(),
                password: password.into(),
            })
            .await?;
        self.session
            .write()
            .expect("write lock on session should not be poisoned")
            .replace(result.clone());
        Ok(result)
    }
    /// Resume a pre-existing session with this agent.
    pub async fn resume_session(
        &self,
        session: Session,
    ) -> Result<(), Error<crate::com::atproto::server::get_session::Error>> {
        self.session
            .write()
            .expect("write lock on session should not be poisoned")
            .replace(session.clone());
        let res = self.api.com.atproto.server.get_session().await;
        match res {
            Ok(result) => {
                assert_eq!(result.did, session.did);
                if let Some(session) = self
                    .session
                    .write()
                    .expect("write lock on session should not be poisoned")
                    .as_mut()
                {
                    session.email = result.email;
                    session.email_confirmed = result.email_confirmed;
                    session.handle = result.handle;
                }
                Ok(())
            }
            Err(err) => {
                self.session
                    .write()
                    .expect("write lock on session should not be poisoned")
                    .take();
                Err(err)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use atrium_xrpc::client::reqwest::ReqwestClient;

    #[derive(Default)]
    struct DummyResponses {
        create_session: Option<crate::com::atproto::server::create_session::Output>,
        get_session: Option<crate::com::atproto::server::get_session::Output>,
    }

    #[derive(Default)]
    struct DummyClient {
        responses: DummyResponses,
    }

    #[async_trait]
    impl HttpClient for DummyClient {
        async fn send_http(
            &self,
            request: Request<Vec<u8>>,
        ) -> Result<Response<Vec<u8>>, Box<dyn std::error::Error + Send + Sync + 'static>> {
            let builder =
                Response::builder().header(http::header::CONTENT_TYPE, "application/json");
            if request
                .headers()
                .get(http::header::AUTHORIZATION)
                .map_or(false, |value| value.to_str().unwrap().contains("expired"))
            {
                return Ok(builder.status(http::StatusCode::BAD_REQUEST).body(
                    serde_json::to_vec(&atrium_xrpc::error::ErrorResponseBody {
                        error: Some(String::from("ExpiredToken")),
                        message: Some(String::from("Token has expired")),
                    })?,
                )?);
            }
            let mut body = Vec::new();
            match request.uri().path().strip_prefix("/xrpc/") {
                Some("com.atproto.server.createSession") => {
                    if let Some(output) = &self.responses.create_session {
                        body.extend(serde_json::to_vec(output)?);
                    }
                }
                Some("com.atproto.server.getSession") => {
                    if let Some(output) = &self.responses.get_session {
                        body.extend(serde_json::to_vec(output)?);
                    }
                }
                Some("com.atproto.server.refreshSession") => {
                    body.extend(serde_json::to_vec(
                        &crate::com::atproto::server::refresh_session::Output {
                            access_jwt: String::from("access"),
                            did: String::from("did"),
                            handle: String::from("handle"),
                            refresh_jwt: String::from("refresh"),
                        },
                    )?);
                }
                _ => {}
            }
            if body.is_empty() {
                Ok(builder
                    .status(http::StatusCode::UNAUTHORIZED)
                    .body(serde_json::to_vec(
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

    impl XrpcClient for DummyClient {
        fn host(&self) -> &str {
            "http://localhost:8080"
        }
    }

    #[test]
    fn test_new() {
        let agent = AtpAgent::new(ReqwestClient::new("http://localhost:8080".into()));
        assert_eq!(agent.get_session(), None);
    }

    #[tokio::test]
    async fn test_login() {
        let session = Session {
            access_jwt: String::from("access"),
            did: String::from("did"),
            email: None,
            email_confirmed: None,
            handle: String::from("handle"),
            refresh_jwt: String::from("refresh"),
        };
        // success
        {
            let client = DummyClient {
                responses: DummyResponses {
                    create_session: Some(crate::com::atproto::server::create_session::Output {
                        ..session.clone()
                    }),
                    ..Default::default()
                },
            };
            let agent = AtpAgent::new(client);
            agent
                .login("test", "pass")
                .await
                .expect("login should be succeeded");
            assert_eq!(agent.get_session(), Some(session));
        }
        // failure with `createSession` error
        {
            let client = DummyClient {
                responses: DummyResponses {
                    ..Default::default()
                },
            };
            let agent = AtpAgent::new(client);
            agent
                .login("test", "bad")
                .await
                .expect_err("login should be failed");
            assert_eq!(agent.get_session(), None);
        }
    }

    #[tokio::test]
    async fn test_resume_session() {
        let session = Session {
            access_jwt: String::from("access"),
            did: String::from("did"),
            email: None,
            email_confirmed: None,
            handle: String::from("handle"),
            refresh_jwt: String::from("refresh"),
        };
        // success
        {
            let client = DummyClient {
                responses: DummyResponses {
                    get_session: Some(crate::com::atproto::server::get_session::Output {
                        did: session.did.clone(),
                        email: session.email.clone(),
                        email_confirmed: session.email_confirmed,
                        handle: session.handle.clone(),
                    }),
                    ..Default::default()
                },
            };
            let agent = AtpAgent::new(client);
            assert_eq!(agent.get_session(), None);
            agent
                .resume_session(Session {
                    email: Some(String::from("test@example.com")),
                    ..session.clone()
                })
                .await
                .expect("resume_session should be succeeded");
            assert_eq!(agent.get_session(), Some(session.clone()));
        }
        // failure with `getSession` error
        {
            let client = DummyClient {
                responses: DummyResponses {
                    ..Default::default()
                },
            };
            let agent = AtpAgent::new(client);
            assert_eq!(agent.get_session(), None);
            agent
                .resume_session(session)
                .await
                .expect_err("resume_session should be failed");
            assert_eq!(agent.get_session(), None);
        }
    }

    #[tokio::test]
    async fn test_refresh_session() {
        let session = Session {
            access_jwt: String::from("access"),
            did: String::from("did"),
            email: None,
            email_confirmed: None,
            handle: String::from("handle"),
            refresh_jwt: String::from("refresh"),
        };
        let client = DummyClient {
            responses: DummyResponses {
                get_session: Some(crate::com::atproto::server::get_session::Output {
                    did: session.did.clone(),
                    email: session.email.clone(),
                    email_confirmed: session.email_confirmed,
                    handle: session.handle.clone(),
                }),
                ..Default::default()
            },
        };
        let agent = AtpAgent::new(client);
        agent
            .resume_session(Session {
                access_jwt: "expired".into(),
                ..session.clone()
            })
            .await
            .expect("resume_session should be succeeded");
        assert_eq!(agent.get_session(), Some(session));
    }
}
