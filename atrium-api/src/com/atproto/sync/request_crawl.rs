// This file is generated by atrium-codegen. Do not edit.
//! Definitions for the `com.atproto.sync.requestCrawl` namespace.

/// Request a service to persistently crawl hosted repos.
#[async_trait::async_trait]
pub trait RequestCrawl: crate::xrpc::XrpcClient {
    async fn request_crawl(&self, params: Parameters) -> Result<(), Box<dyn std::error::Error>> {
        let body = crate::xrpc::XrpcClient::send::<Error>(
            self,
            http::Method::GET,
            "com.atproto.sync.requestCrawl",
            Some(serde_urlencoded::to_string(&params)?),
            None,
            None,
        )
        .await?;
        serde_json::from_slice(&body).map_err(|e| e.into())
    }
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Parameters {
    /// Hostname of the service that is requesting to be crawled.
    pub hostname: String,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(tag = "error", content = "message")]
pub enum Error {
}