use async_trait::async_trait;
use http::{header, Method, Request, Response};
use serde::{de::DeserializeOwned, Serialize};
use serde_json::{from_slice, to_vec};
use std::error::Error;

#[async_trait]
pub trait HttpClient {
    async fn send(&self, req: Request<Vec<u8>>) -> Result<Response<Vec<u8>>, Box<dyn Error>>;
}

#[async_trait]
pub trait XrpcClient: HttpClient {
    fn host(&self) -> &str {
        "https://bsky.social"
    }
    async fn send<Output>(
        &self,
        method: Method,
        path: &str,
        body: Option<impl Serialize + Send + Sync>,
    ) -> Result<Output, Box<dyn Error>>
    where
        Output: DeserializeOwned,
    {
        let builder = Request::builder()
            .method(method)
            .uri(format!("{}/xrpc/{path}", self.host()))
            .header(header::CONTENT_TYPE, "application/json");
        let res = HttpClient::send(self, builder.body(to_vec(&body)?)?).await?;
        let (parts, body) = res.into_parts();
        if parts.status.is_success() {
            Ok(from_slice(&body)?)
        } else {
            // TODO
            Err(format!("status: {}", parts.status).into())
        }
    }
}
