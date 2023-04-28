// This file is generated by atrium-codegen. Do not edit.
//! Definitions for the `com.atproto.server.requestPasswordReset` namespace.

/// Initiate a user account password reset via email.
#[async_trait::async_trait]
pub trait RequestPasswordReset: crate::xrpc::XrpcClient {
    async fn request_password_reset(&self, input: Input) -> Result<(), Box<dyn std::error::Error>> {
        crate::xrpc::XrpcClient::send_unit(
            self,
            http::Method::POST,
            "com.atproto.server.requestPasswordReset",
            Option::<()>::None,
            Some(input),
        )
        .await
    }
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Input {
    pub email: String,
}

pub enum Error {
}
