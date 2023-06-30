# ATrium API: Rust library for Bluesky's atproto services

[![](https://img.shields.io/crates/v/atrium-api)](https://crates.io/crates/atrium-api)
[![](https://img.shields.io/docsrs/atrium-api)](https://docs.rs/atrium-api)
[![](https://img.shields.io/crates/l/atrium-api)](https://github.com/sugyan/atrium/blob/main/LICENSE)
[![Rust](https://github.com/sugyan/atrium/actions/workflows/api.yml/badge.svg?branch=main)](https://github.com/sugyan/atrium/actions/workflows/api.yml)

ATrium API is a Rust library that includes the definitions of XRPC requests and their associated input/output model types. These codes are generated from the Lexicon schema on [atproto.com](https://atproto.com/).

## Usage

You can use any HTTP client that implements [`atrium_xrpc::HttpClient`](https://docs.rs/atrium-xrpc/latest/atrium_xrpc/trait.HttpClient.html) to make use of the XRPC requests. [`atrium_xrpc`](https://docs.rs/atrium-xrpc) also includes a default implementation using [`reqwest`](https://crates.io/crates/reqwest).

```rust,no_run
use atrium_api::client::AtpServiceClient;
use atrium_api::com::atproto::server::create_session::Input;
use atrium_xrpc::client::reqwest::ReqwestClient;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = AtpServiceClient::new(Arc::new(ReqwestClient::new("https://bsky.social".into())));
    let result = client
        .com
        .atproto
        .server
        .create_session(Input {
            identifier: "example.bsky.social".into(),
            password: "********".into(),
        })
        .await;
    println!("{:?}", result);
    Ok(())
}
```
