//! An ATP "Agent".
//! Manages session token lifecycles and provides all XRPC methods.
use crate::client::AtpService;
use crate::client::AtpServiceClient;
use crate::client_services::Service;
use async_trait::async_trait;
use atrium_xrpc::error::Error;
use atrium_xrpc::{HttpClient, InputDataOrBytes, OutputDataOrBytes, XrpcClient};
use http::{Method, Request, Response};
use serde::{de::DeserializeOwned, Serialize};
use std::sync::{Arc, RwLock};

/// Type alias for the [com::atproto::server::create_session::Output](crate::com::atproto::server::create_session::Output)
pub type Session = crate::com::atproto::server::create_session::Output;

pub struct BaseClient<T>
where
    T: XrpcClient,
{
    xrpc: T,
    session: Arc<RwLock<Option<Session>>>,
}

#[async_trait]
impl<T> HttpClient for BaseClient<T>
where
    T: XrpcClient + Send + Sync,
{
    async fn send_http(
        &self,
        req: Request<Vec<u8>>,
    ) -> Result<Response<Vec<u8>>, Box<dyn std::error::Error + Send + Sync + 'static>> {
        self.send_http(req).await
    }
}

#[async_trait]
impl<T> XrpcClient for BaseClient<T>
where
    T: XrpcClient + Send + Sync,
{
    fn host(&self) -> &str {
        self.xrpc.host()
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

#[async_trait]
impl<T> AtpService for BaseClient<T>
where
    T: XrpcClient + Send + Sync,
{
    async fn send<P, I, O, E>(
        &self,
        method: Method,
        path: &str,
        parameters: Option<P>,
        input: Option<InputDataOrBytes<I>>,
        encoding: Option<String>,
    ) -> Result<OutputDataOrBytes<O>, Error<E>>
    where
        P: Serialize + Send,
        I: Serialize + Send,
        O: DeserializeOwned,
        E: DeserializeOwned,
    {
        self.xrpc
            .send_xrpc(method, path, parameters, input, encoding)
            .await
    }
}

pub struct AtpAgent<T>
where
    T: AtpService,
{
    pub api: AtpServiceClient<T>,
    session: Arc<RwLock<Option<Session>>>,
}

impl<T> AtpAgent<BaseClient<T>>
where
    T: XrpcClient + Send + Sync,
{
    pub fn new(xrpc: T) -> Self {
        let session = Arc::new(RwLock::new(None));
        let api = AtpServiceClient {
            service: Service::new(Arc::new(BaseClient {
                xrpc,
                session: Arc::clone(&session),
            })),
        };
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
            .service
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
        let res = self.api.service.com.atproto.server.get_session().await;
        match res {
            Ok(result) => {
                assert_eq!(result.did, session.did);
                self.session
                    .write()
                    .expect("write lock on session should not be poisoned")
                    .replace(Session {
                        email: result.email,
                        handle: result.handle,
                        email_confirmed: result.email_confirmed,
                        ..session
                    });
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
            req: Request<Vec<u8>>,
        ) -> Result<Response<Vec<u8>>, Box<dyn std::error::Error + Send + Sync + 'static>> {
            let builder =
                Response::builder().header(http::header::CONTENT_TYPE, "application/json");
            // if req
            //     .headers()
            //     .get(http::header::AUTHORIZATION)
            //     .map_or(false, |value| value.to_str().unwrap().contains("expired"))
            // {
            //     return Ok(builder.status(http::StatusCode::BAD_REQUEST).body(
            //         serde_json::to_vec(&atrium_xrpc::error::ErrorResponseBody {
            //             error: Some(String::from("ExpiredToken")),
            //             message: Some(String::from("Token has expired"))),
            //         })?,
            //     )?);
            // }
            let mut body = Vec::new();
            match req.uri().path().strip_prefix("/xrpc/") {
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
    async fn test_refresh_token() {
        todo!()
    }
}
