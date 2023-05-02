// This file is generated by atrium-codegen. Do not edit.
//! Definitions for the `com.atproto.server.deleteSession` namespace.

/// Delete the current session.
#[async_trait::async_trait]
pub trait DeleteSession: crate::xrpc::XrpcClient {
    async fn delete_session(&self, input: Vec<u8>) -> Result<(), Box<dyn std::error::Error>> {
        let _ = crate::xrpc::XrpcClient::send::<Error>(
            self,
            http::Method::POST,
            "com.atproto.server.deleteSession",
            None,
            Some(input),
            None,
        )
        .await?;
        Ok(())
    }
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(tag = "error", content = "message")]
pub enum Error {
}