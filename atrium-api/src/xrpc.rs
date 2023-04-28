use async_trait::async_trait;
use http::{header, Method, Request, Response};
use serde::{de::DeserializeOwned, Serialize};
use serde_json::{from_slice, to_vec};
use std::error::Error;
use url::Url;

#[async_trait]
pub trait HttpClient {
    async fn send(&self, req: Request<Vec<u8>>) -> Result<Response<Vec<u8>>, Box<dyn Error>>;
}

#[async_trait]
pub trait XrpcClient: HttpClient {
    fn host(&self) -> &str;
    fn auth(&self) -> Option<&str>;

    async fn send_xrpc(
        &self,
        method: Method,
        path: &str,
        params: Option<impl Serialize + Send + Sync>,
        input: Option<impl Serialize + Send + Sync>,
    ) -> Result<Response<Vec<u8>>, Box<dyn Error>> {
        let mut url = Url::parse(&format!("{}/xrpc/{path}", self.host())).expect("invalid url");
        if let Some(params) = params {
            if let Ok(query) = serde_urlencoded::to_string(params) {
                url.set_query(Some(&query));
            }
        }
        let mut builder = Request::builder()
            .method(method)
            .uri(url.as_str())
            .header(header::CONTENT_TYPE, "application/json");
        if let Some(token) = self.auth() {
            builder = builder.header(header::AUTHORIZATION, format!("Bearer {}", token));
        }
        let body = if let Some(body) = input {
            to_vec(&body)?
        } else {
            Vec::new()
        };
        HttpClient::send(self, builder.body(body)?).await
    }
    async fn send_unit(
        &self,
        method: Method,
        path: &str,
        params: Option<impl Serialize + Send + Sync>,
        input: Option<impl Serialize + Send + Sync>,
    ) -> Result<(), Box<dyn Error>> {
        let (parts, _) = self
            .send_xrpc(method, path, params, input)
            .await?
            .into_parts();
        if parts.status.is_success() {
            Ok(())
        } else {
            // TODO
            Err(format!("status: {}", parts.status).into())
        }
    }
    async fn send<Output>(
        &self,
        method: Method,
        path: &str,
        params: Option<impl Serialize + Send + Sync>,
        input: Option<impl Serialize + Send + Sync>,
    ) -> Result<Output, Box<dyn Error>>
    where
        Output: DeserializeOwned,
    {
        let (parts, body) = self
            .send_xrpc(method, path, params, input)
            .await?
            .into_parts();
        if parts.status.is_success() {
            Ok(from_slice(&body)?)
        } else {
            // TODO
            Err(format!("status: {}", parts.status).into())
        }
    }
}
