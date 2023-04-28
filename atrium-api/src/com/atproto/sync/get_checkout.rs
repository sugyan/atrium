// This file is generated by atrium-codegen. Do not edit.
//! Definitions for the `com.atproto.sync.getCheckout` namespace.

/// Gets the repo state.
#[async_trait::async_trait]
pub trait GetCheckout: crate::xrpc::XrpcClient {
    async fn get_checkout(&self, params: Parameters) -> Result<(), Box<dyn std::error::Error>> {
        crate::xrpc::XrpcClient::send_unit(
            self,
            http::Method::GET,
            "com.atproto.sync.getCheckout",
            Some(params),
            Option::<()>::None,
        )
        .await
    }
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Parameters {
    /// The commit to get the checkout from. Defaults to current HEAD.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub commit: Option<String>,
    /// The DID of the repo.
    pub did: String,
}


pub enum Error {
}
