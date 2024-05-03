use super::error::{Error, Result};
use super::{plc_resolver::DidPlcResolver, web_resolver::DidWebResolver};
use super::{Fetch, Resolve};
use async_trait::async_trait;

#[derive(Debug)]
pub struct DidResolver<T> {
    pub plc: DidPlcResolver<T>,
    pub web: DidWebResolver<T>,
}

#[async_trait]
impl<T> Resolve for DidResolver<T>
where
    T: Fetch + Send + Sync,
{
    async fn resolve_no_check(&self, did: &str) -> Result<Option<Vec<u8>>> {
        let parts = did.split(':').collect::<Vec<_>>();
        if parts.len() < 3 || parts[0] != "did" {
            return Err(Error::PoorlyFormattedDid(did.to_string()));
        }
        match parts[1] {
            "web" => self.web.resolve_no_check(did).await,
            "plc" => self.plc.resolve_no_check(did).await,
            _ => Err(Error::UnsupportedDidMethod(did.to_string())),
        }
    }
}

impl<T> Default for DidResolver<T> {
    fn default() -> Self {
        let timeout = Some(3000);
        let plc_url = String::from("https://plc.directory");
        Self {
            plc: DidPlcResolver::new(plc_url, timeout),
            web: DidWebResolver::new(timeout),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common_web::did_doc::{DidDocument, Service};
    use mockito::{Matcher, Server, ServerGuard};
    use reqwest::{header::CONTENT_TYPE, Client};
    use std::time::Duration;

    struct ReqwestFetcher;

    #[async_trait]
    impl Fetch for ReqwestFetcher {
        async fn fetch(
            url: &str,
            timeout: Option<u64>,
        ) -> std::result::Result<Option<Vec<u8>>, Box<dyn std::error::Error + Send + Sync + 'static>>
        {
            let mut builder = Client::builder();
            if let Some(timeout) = timeout {
                builder = builder.timeout(Duration::from_millis(timeout));
            }
            match builder.build()?.get(url).send().await?.error_for_status() {
                Ok(response) => Ok(Some(response.bytes().await?.to_vec())),
                Err(err) => {
                    if err
                        .status()
                        .map_or(false, |status| status.is_client_error())
                    {
                        Ok(None)
                    } else {
                        Err(Box::new(err))
                    }
                }
            }
        }
    }

    fn did_doc_example() -> DidDocument {
        DidDocument {
            context: None,
            id: String::from("did:plc:234567abcdefghijklmnopqr"),
            also_known_as: Some(vec![String::from("at://alice.test")]),
            verification_method: None,
            service: Some(vec![Service {
                id: String::from("#atproto_pds"),
                r#type: String::from("AtprotoPersonalDataServer"),
                service_endpoint: String::from("https://service.test"),
            }]),
        }
    }

    async fn web_server() -> (ServerGuard, DidDocument) {
        let mut did_doc = did_doc_example();
        let mut server = Server::new_async().await;
        did_doc.id = format!(
            "did:web:{}",
            urlencoding::encode(&server.host_with_port()).into_owned()
        );
        server
            .mock("GET", "/.well-known/did.json")
            .with_status(200)
            .with_header(CONTENT_TYPE.as_str(), "application/did+ld+json")
            .with_body(serde_json::to_vec(&did_doc).expect("failed to serialize did_doc"))
            .create();
        (server, did_doc)
    }

    async fn plc_server() -> (ServerGuard, DidDocument) {
        let did_doc = did_doc_example();
        let mut server = Server::new_async().await;
        server
            .mock(
                "GET",
                format!("/{}", urlencoding::encode(&did_doc.id)).as_str(),
            )
            .with_status(200)
            .with_header(CONTENT_TYPE.as_str(), "application/did+ld+json")
            .with_body(serde_json::to_vec(&did_doc_example()).expect("failed to serialize did_doc"))
            .create();
        server
            .mock("GET", Matcher::Regex(String::from(r"^/[^/]+$")))
            .with_status(404)
            .create();
        (server, did_doc)
    }

    fn resolver(plc_url: Option<String>) -> DidResolver<ReqwestFetcher> {
        let timeout = Some(3000);
        DidResolver {
            plc: DidPlcResolver::new(
                plc_url.unwrap_or(String::from("https://plc.directory")),
                timeout,
            ),
            web: DidWebResolver::new(timeout),
        }
    }

    #[tokio::test]
    async fn resolve_did_web_valid() {
        let (_server, did_doc) = web_server().await;
        let resolver = resolver(None);
        let result = resolver
            .ensure_resolve(&did_doc.id, false)
            .await
            .expect("ensure_resolve shoud succeed with a valid did:web");
        assert_eq!(result, did_doc);
    }

    #[tokio::test]
    async fn resolve_did_web_malformed() {
        let resolver = resolver(None);

        let err = resolver
            .ensure_resolve("did:web:asdf", false)
            .await
            .expect_err("ensure_resolve should fail with a malformed did:web");
        assert!(
            matches!(err, Error::Fetch(_)),
            "error should be Fetch: {err:?}"
        );

        let err = resolver
            .ensure_resolve("did:web:", false)
            .await
            .expect_err("ensure_resolve should fail with a malformed did:web");
        assert!(
            matches!(err, Error::PoorlyFormattedDid(_)),
            "error should be PoorlyFormattedDid: {err:?}"
        );

        let err = resolver
            .ensure_resolve("", false)
            .await
            .expect_err("ensure_resolve should fail with a malformed did:web");
        assert!(
            matches!(err, Error::PoorlyFormattedDid(_)),
            "error should be PoorlyFormattedDid: {err:?}"
        );
    }

    #[tokio::test]
    async fn resolve_did_web_with_path_components() {
        let resolver = resolver(None);
        let err = resolver
            .ensure_resolve("did:web:example.com:u:bob", false)
            .await
            .expect_err("ensure_resolve should fail with did:web with path components");
        assert!(
            matches!(err, Error::UnsupportedDidWebPath(_)),
            "error should be UnsupportedDidWebPath: {err:?}"
        );
    }

    #[tokio::test]
    async fn resolve_did_plc_valid() {
        let (server, did_doc) = plc_server().await;
        let resolver = resolver(Some(server.url()));
        let result = resolver
            .ensure_resolve(&did_doc.id, false)
            .await
            .expect("ensure_resolve shoud succeed with a valid did:plc");
        assert_eq!(result, did_doc);
    }

    #[tokio::test]
    async fn resolve_did_plc_malformed() {
        let (server, _) = plc_server().await;
        let resolver = resolver(Some(server.url()));

        let err = resolver
            .ensure_resolve("did:plc:asdf", false)
            .await
            .expect_err("ensure_resolve should fail with a malformed did:plc");
        assert!(
            matches!(err, Error::DidNotFound(_)),
            "error should be DidNotFound: {err:?}"
        );

        let err = resolver
            .ensure_resolve("did:plc", false)
            .await
            .expect_err("ensure_resolve should fail with a malformed did:plc");
        assert!(
            matches!(err, Error::PoorlyFormattedDid(_)),
            "error should be PoorlyFormattedDid: {err:?}"
        );
    }
}
