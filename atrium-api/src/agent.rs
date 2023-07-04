//! An ATP "Agent".
//! Manages session token lifecycles and provides all XRPC methods.
use crate::client::AtpServiceClient;
use async_trait::async_trait;
use atrium_xrpc::error::Error;
use atrium_xrpc::{HttpClient, XrpcClient};
use http::{Request, Response};
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
    async fn send(
        &self,
        req: Request<Vec<u8>>,
    ) -> Result<Response<Vec<u8>>, Box<dyn std::error::Error + Send + Sync + 'static>> {
        HttpClient::send(&self.xrpc, req).await
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

pub struct AtpAgent<T>
where
    T: XrpcClient + Send + Sync,
{
    pub api: crate::client::AtpServiceClient<T>,
    session: Arc<RwLock<Option<Session>>>,
}

impl<T> AtpAgent<BaseClient<T>>
where
    T: XrpcClient + Send + Sync,
{
    pub fn new(xrpc: T) -> Self {
        let session = Arc::new(RwLock::new(None));
        let base = BaseClient {
            xrpc,
            session: Arc::clone(&session),
        };
        let api = AtpServiceClient::new(Arc::new(base));
        Self { api, session }
    }
    pub fn get_session(&self) -> Option<Session> {
        self.session.read().expect("read lock").clone()
    }
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
            .expect("write lock")
            .replace(result.clone());
        Ok(result)
    }
    pub async fn resume_session(
        &self,
        session: Session,
    ) -> Result<(), Error<crate::com::atproto::server::get_session::Error>> {
        self.session
            .write()
            .expect("write lock")
            .replace(session.clone());
        match self.api.com.atproto.server.get_session().await {
            Ok(result) => {
                assert_eq!(result.did, session.did);
                self.session.write().expect("write lock").replace(Session {
                    email: result.email,
                    handle: result.handle,
                    ..session
                });
                Ok(())
            }
            Err(err) => {
                self.session.write().expect("write lock").take();
                Err(err)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct DummyClient {
        session: Option<Session>,
    }

    #[async_trait]
    impl HttpClient for DummyClient {
        async fn send(
            &self,
            _req: Request<Vec<u8>>,
        ) -> Result<Response<Vec<u8>>, Box<dyn std::error::Error + Send + Sync + 'static>> {
            let builder =
                Response::builder().header(http::header::CONTENT_TYPE, "application/json");
            if let Some(session) = &self.session {
                Ok(builder
                    .status(http::StatusCode::OK)
                    .body(serde_json::to_vec(&session)?)?)
            } else {
                Ok(builder
                    .status(http::StatusCode::UNAUTHORIZED)
                    .body(serde_json::to_vec(
                        &atrium_xrpc::error::ErrorResponseBody {
                            error: Some(String::from("AuthenticationRequired")),
                            message: Some(String::from("Invalid identifier or password")),
                        },
                    )?)?)
            }
        }
    }

    impl XrpcClient for DummyClient {
        fn host(&self) -> &str {
            "http://localhost:8080"
        }
    }

    #[test]
    fn new_agent() {
        let agent = AtpAgent::new(atrium_xrpc::client::reqwest::ReqwestClient::new(
            "http://localhost:8080".into(),
        ));
        assert_eq!(agent.get_session(), None);
    }

    #[tokio::test]
    async fn login() {
        let session = Session {
            access_jwt: "access".into(),
            did: "did".into(),
            email: None,
            handle: "handle".into(),
            refresh_jwt: "refresh".into(),
        };
        // success
        {
            let client = DummyClient {
                session: Some(session.clone()),
            };
            let agent = AtpAgent::new(client);
            agent.login("test", "pass").await.expect("failed to login");
            assert_eq!(agent.get_session(), Some(session));
        }
        // failure with `createSession` error
        {
            let client = DummyClient { session: None };
            let agent = AtpAgent::new(client);
            agent
                .login("test", "bad")
                .await
                .expect_err("should failed to login");
            assert_eq!(agent.get_session(), None);
        }
    }

    #[tokio::test]
    async fn resume_session() {
        let session = Session {
            access_jwt: "access".into(),
            did: "did".into(),
            email: None,
            handle: "handle".into(),
            refresh_jwt: "refresh".into(),
        };
        // success
        {
            let client = DummyClient {
                session: Some(session.clone()),
            };
            let agent = AtpAgent::new(client);
            assert_eq!(agent.get_session(), None);
            agent
                .resume_session(Session {
                    email: Some(String::from("test@example.com")),
                    ..session.clone()
                })
                .await
                .expect("failed to resume session");
            assert_eq!(agent.get_session(), Some(session.clone()));
        }
        // failure with `getSession` error
        {
            let client = DummyClient { session: None };
            let agent = AtpAgent::new(client);
            assert_eq!(agent.get_session(), None);
            agent
                .resume_session(session)
                .await
                .expect_err("should failed to resume session");
            assert_eq!(agent.get_session(), None);
        }
    }
}
