use super::DnsTxtResolver;
use async_trait::async_trait;
use atrium_xrpc::http::{uri::InvalidUri, StatusCode, Uri};
use atrium_xrpc::HttpClient;
use hickory_proto::op::{Message, Query};
use hickory_proto::rr::{RData, RecordType};
use std::sync::Arc;
use thiserror::Error;

const DOH_MEDIA_TYPE: &str = "application/dns-message";

#[derive(Error, Debug)]
pub enum Error {
    #[error("http status: {0:?}")]
    HttpStatus(StatusCode),
}

#[derive(Clone, Debug)]
pub struct DohDnsTxtResolverConfig<T> {
    pub service_url: String,
    pub http_client: Arc<T>,
}

pub struct DohDnsTxtResolver<T> {
    service_url: Uri,
    http_client: Arc<T>,
}

impl<T> DohDnsTxtResolver<T> {
    #[allow(dead_code)]
    pub fn new(config: DohDnsTxtResolverConfig<T>) -> core::result::Result<Self, InvalidUri> {
        Ok(Self {
            service_url: config.service_url.parse()?,
            http_client: config.http_client,
        })
    }
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl<T> DnsTxtResolver for DohDnsTxtResolver<T>
where
    T: HttpClient + Send + Sync + 'static,
{
    async fn resolve(
        &self,
        query: &str,
    ) -> core::result::Result<Vec<String>, Box<dyn std::error::Error + Send + Sync + 'static>> {
        let mut message = Message::new();
        message
            .set_recursion_desired(true)
            .add_query(Query::query(query.parse()?, RecordType::TXT));
        let res = self
            .http_client
            .send_http(
                atrium_xrpc::http::Request::builder()
                    .method(atrium_xrpc::http::Method::POST)
                    .header(atrium_xrpc::http::header::CONTENT_TYPE, DOH_MEDIA_TYPE)
                    .uri(&self.service_url)
                    .body(message.to_vec()?)?,
            )
            .await?;
        if res.status().is_success() {
            Ok(Message::from_vec(res.body())?
                .answers()
                .iter()
                .filter_map(|answer| match answer.data() {
                    Some(RData::TXT(txt)) => Some(txt.to_string()),
                    _ => None,
                })
                .collect())
        } else {
            Err(Box::new(Error::HttpStatus(res.status())))
        }
    }
}
