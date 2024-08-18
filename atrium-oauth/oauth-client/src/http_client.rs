use atrium_xrpc::HttpClient;
#[cfg(feature = "default-client")]
use reqwest::Client;
use std::sync::{Arc, OnceLock};

static HTTP_CLIENT: OnceLock<Arc<dyn HttpClient + Send + Sync + 'static>> = OnceLock::new();

pub fn set_http_client(
    client: impl HttpClient + Send + Sync + 'static,
) -> Result<(), Arc<dyn HttpClient + Send + Sync + 'static>> {
    HTTP_CLIENT.set(Arc::new(client))
}

pub fn get_http_client() -> Arc<dyn HttpClient + Send + Sync + 'static> {
    HTTP_CLIENT.get_or_init(get_default_client).clone()
}

#[cfg(feature = "default-client")]
fn get_default_client() -> Arc<dyn HttpClient + Send + Sync + 'static> {
    Arc::new(ReqwestClient::default())
}

#[cfg(not(feature = "default-client"))]
fn get_default_client() -> Arc<dyn HttpClient + Send + Sync + 'static> {
    panic!("no default client available")
}

#[cfg(feature = "default-client")]
struct ReqwestClient {
    client: Client,
}

#[cfg(feature = "default-client")]
#[cfg_attr(target_arch = "wasm32", async_trait::async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait::async_trait)]
impl HttpClient for ReqwestClient {
    async fn send_http(
        &self,
        request: atrium_xrpc::http::Request<Vec<u8>>,
    ) -> core::result::Result<
        atrium_xrpc::http::Response<Vec<u8>>,
        Box<dyn std::error::Error + Send + Sync + 'static>,
    > {
        let response = self.client.execute(request.try_into()?).await?;
        let mut builder = atrium_xrpc::http::Response::builder().status(response.status());
        for (k, v) in response.headers() {
            builder = builder.header(k, v);
        }
        builder
            .body(response.bytes().await?.to_vec())
            .map_err(Into::into)
    }
}

#[cfg(feature = "default-client")]
impl Default for ReqwestClient {
    fn default() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }
}
