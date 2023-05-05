# ATrium API: Rust library for Bluesky's atproto services

![](https://img.shields.io/crates/v/atrium-api)
![](https://img.shields.io/docsrs/atrium-api)
![](https://img.shields.io/crates/l/atrium-api)
![Rust](https://github.com/sugyan/atrium/actions/workflows/api.yml/badge.svg?branch=main)](https://github.com/sugyan/atrium/actions/workflows/api.yml)

ATrium API is a Rust library that includes the definitions of XRPC requests and their associated input/output model types. These codes are generated from the Lexicon schema on [atproto.com](https://atproto.com/).

## Usage

You can use any HTTP client that implements `atrium_api::xrpc::HttpClient` to make use of the XRPC requests. Below is the simplest example using `reqwest`.

```rust,ignore
#[derive(Default)]
struct MyClient(reqwest::Client);

#[async_trait::async_trait]
impl atrium_api::xrpc::HttpClient for MyClient {
    async fn send(
        &self,
        req: http::Request<Vec<u8>>,
    ) -> Result<http::Response<Vec<u8>>, Box<dyn std::error::Error>> {
        let res = self.0.execute(req.try_into()?).await?;
        let mut builder = http::Response::builder().status(res.status());
        for (k, v) in res.headers() {
            builder = builder.header(k, v);
        }
        builder
            .body(res.bytes().await?.to_vec())
            .map_err(Into::into)
    }
}

#[async_trait::async_trait]
impl atrium_api::xrpc::XrpcClient for MyClient {
    fn host(&self) -> &str {
        "https://bsky.social"
    }
    fn auth(&self) -> Option<&str> {
        None
    }
}

atrium_api::impl_traits!(MyClient);

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    use atrium_api::com::atproto::server::create_session::{CreateSession, Input};
    let session = MyClient::default()
        .create_session(Input {
            identifier: "<your handle>.bsky.social".into(),
            password: "<your app password>".into(),
        })
        .await?;
    println!("{:?}", session);
    Ok(())
}
```