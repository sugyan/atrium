use super::error::{Error, Result};
use super::{plc_resolver::DidPlcResolver, web_resolver::DidWebResolver};
use super::{Fetcher, Resolver};
use async_trait::async_trait;

#[derive(Debug)]
pub struct DidResolver<T> {
    pub plc: DidPlcResolver<T>,
    pub web: DidWebResolver<T>,
}

#[async_trait]
impl<T> Resolver for DidResolver<T>
where
    T: Fetcher + Send + Sync,
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
    use crate::common_web::did_doc::{DidDocument, Service, VerificationMethod};
    use mockito::{Server, ServerGuard};
    use reqwest::{header::CONTENT_TYPE, Client};
    use std::time::Duration;

    struct ReqwestFetcher;

    #[async_trait]
    impl Fetcher for ReqwestFetcher {
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
            context: Some(vec![
                String::from("https://www.w3.org/ns/did/v1"),
                String::from("https://w3id.org/security/multikey/v1"),
                String::from("https://w3id.org/security/suites/secp256k1-2019/v1"),
            ]),
            id: String::from("did:plc:4ee6oesrsbtmuln4gqsqf6fp"),
            also_known_as: Some(vec![String::from("at://sugyan.com")]),
            verification_method: Some(vec![VerificationMethod {
                id: String::from("did:plc:4ee6oesrsbtmuln4gqsqf6fp#atproto"),
                r#type: String::from("Multikey"),
                controller: String::from("did:plc:4ee6oesrsbtmuln4gqsqf6fp"),
                public_key_multibase: Some(String::from(
                    "zQ3shnw8ChQwGUE6gMghuvn5g7Q9YVej1MUJENqMsLmxZwRSz",
                )),
            }]),
            service: Some(vec![Service {
                id: String::from("#atproto_pds"),
                r#type: String::from("AtprotoPersonalDataServer"),
                service_endpoint: String::from("https://puffball.us-east.host.bsky.network"),
            }]),
        }
    }

    async fn server() -> ServerGuard {
        let mut server = Server::new_async().await;
        server
            .mock("GET", "/.well-known/did.json")
            .with_status(200)
            .with_header(CONTENT_TYPE.as_str(), "application/did+ld+json")
            .with_body(serde_json::to_vec(&did_doc_example()).expect("failed to serialize did_doc"))
            .create();
        server
    }

    #[tokio::test]
    async fn resolve_valid_did_web() {
        let server = server().await;
        let resolver = DidResolver::<ReqwestFetcher> {
            plc: DidPlcResolver::new("https://plc.directory".to_string(), Some(3000)),
            web: DidWebResolver::new(Some(3000)),
        };
        let web_did = format!(
            "did:web:{}",
            urlencoding::encode(&server.host_with_port()).into_owned()
        );
        let result = resolver
            .ensure_resolve(&web_did, false)
            .await
            .expect("ensure_resolve shoud succeed with a valid did:web");
        assert_eq!(result, did_doc_example());
    }
}
