# ATrium API: Rust library for Bluesky's atproto services

[![](https://img.shields.io/crates/v/atrium-api)](https://crates.io/crates/atrium-api)
[![](https://img.shields.io/docsrs/atrium-api)](https://docs.rs/atrium-api)
[![](https://img.shields.io/crates/l/atrium-api)](https://github.com/sugyan/atrium/blob/main/LICENSE)
[![Rust](https://github.com/sugyan/atrium/actions/workflows/api.yml/badge.svg?branch=main)](https://github.com/sugyan/atrium/actions/workflows/api.yml)

ATrium API is a Rust library that includes the definitions of XRPC requests and their associated input/output model types. These codes are generated from the Lexicon schema on [atproto.com](https://atproto.com/).

## Usage

Any HTTP client that implements [`atrium_xrpc::HttpClient`](https://docs.rs/atrium-xrpc/latest/atrium_xrpc/trait.HttpClient.html) can be used to handle XRPC requests. Since [`atrium_xrpc_client`](https://docs.rs/atrium-xrpc-client) provides several implementations, it is recommended to use one of them that fits your project requirements.


```rust,no_run
use atrium_api::client::AtpServiceClient;
use atrium_api::com::atproto::server::create_session::Input;
use atrium_xrpc_client::reqwest::ReqwestClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = AtpServiceClient::new(ReqwestClient::new("https://bsky.social"));
    let result = client
        .service
        .com
        .atproto
        .server
        .create_session(Input {
            auth_factor_token: None,
            identifier: "alice@mail.com".into(),
            password: "hunter2".into(),
        })
        .await;
    println!("{:?}", result);
    Ok(())
}
```

### `AtpAgent`

While `AtpServiceClient` can be used for simple XRPC calls, it is better to use `AtpAgent`, which has practical features such as session management.

```rust,no_run
use atrium_api::agent::{store::MemorySessionStore, AtpAgent};
use atrium_xrpc_client::reqwest::ReqwestClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let agent = AtpAgent::new(
        ReqwestClient::new("https://bsky.social"),
        MemorySessionStore::default(),
    );
    agent.login("alice@mail.com", "hunter2").await?;
    let result = agent
        .api
        .app
        .bsky
        .actor
        .get_profile(atrium_api::app::bsky::actor::get_profile::Parameters {
            actor: "bsky.app".parse()?,
        })
        .await?;
    println!("{:?}", result);
    Ok(())
}
```
