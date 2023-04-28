use async_trait::async_trait;
use http::{header, Method, Request, Response};
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
    async fn send(
        &self,
        method: Method,
        path: &str,
        query: Option<String>,
        input: Option<Vec<u8>>,
        encoding: Option<String>,
    ) -> Result<Vec<u8>, Box<dyn Error>> {
        let mut url = Url::parse(&format!("{}/xrpc/{path}", self.host())).expect("invalid url");
        if let Some(query) = query {
            url.set_query(Some(&query));
        }
        let mut builder = Request::builder().method(method).uri(url.as_str());
        if let Some(encoding) = encoding {
            builder = builder.header(header::CONTENT_TYPE, encoding);
        }
        if let Some(token) = self.auth() {
            builder = builder.header(header::AUTHORIZATION, format!("Bearer {}", token));
        }

        let body = HttpClient::send(self, builder.body(input.unwrap_or_default())?)
            .await?
            .into_body();
        // TODO: Error
        Ok(body)
    }
}
