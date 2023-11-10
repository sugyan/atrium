# ATrium XRPC Client

[![](https://img.shields.io/crates/v/atrium-xrpc-client)](https://crates.io/crates/atrium-xrpc-client)
[![](https://img.shields.io/docsrs/atrium-xrpc-client)](https://docs.rs/atrium-xrpc-client)
[![](https://img.shields.io/crates/l/atrium-xrpc-client)](https://github.com/sugyan/atrium/blob/main/LICENSE)
[![Rust](https://github.com/sugyan/atrium/actions/workflows/xrpc-client.yml/badge.svg?branch=main)](https://github.com/sugyan/atrium/actions/workflows/xrpc-client.yml)

This library provides clients that implement the [`XrpcClient`](https://docs.rs/atrium-xrpc/latest/atrium_xrpc/trait.XrpcClient.html) defined in [`atrium-xrpc`](../atrium-xrpc/). To accommodate a wide range of use cases, four feature flags are provided to allow developers to choose the best asynchronous HTTP client library for their project as a backend.

## Features

- `reqwest-native` (default)
- `reqwest-rustls`
- `isahc`
- `surf`

Usage examples are provided below.

### `reqwest-native` and `reqwest-rustls`

If you are using [`tokio`](https://crates.io/crates/tokio) as your asynchronous runtime, you may find it convenient to utilize the [`reqwest`](https://crates.io/crates/reqwest) backend with this feature, which is a high-level asynchronous HTTP Client. Within this crate, you have the choice of configuring `reqwest` with either `reqwest/native-tls` or `reqwest/rustls-tls`.

```toml
[dependencies]
atrium-xrpc-client = "*"
```

To use the `reqwest::Client` with the `rustls` TLS backend, specify the feature as follows:

```toml
[dependencies]
atrium-xrpc-client = { version = "*", default-features = false, features = ["reqwest-rustls"]}
```

In either case, you can use the `ReqwestClient`:

```rust
use atrium_xrpc_client::reqwest::ReqwestClient;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = ReqwestClient::new("https://bsky.social");
    Ok(())
}
```

You can also directly specify a `reqwest::Client` with your own configuration:

```toml
[dependencies]
atrium-xrpc-client = { version = "*", default-features = false }
reqwest = { version = "0.11.22", default-features = false, features = ["rustls-tls"] }
```

```rust
use atrium_xrpc_client::reqwest::ReqwestClientBuilder;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = ReqwestClientBuilder::new("https://bsky.social")
        .client(
            reqwest::ClientBuilder::new()
                .timeout(std::time::Duration::from_millis(1000))
                .use_rustls_tls()
                .build()?,
        )
        .build();
    Ok(())
}
```

For more details, refer to the [`reqwest` documentation](https://docs.rs/reqwest).

### `isahc`

The `reqwest` client may not work on asynchronous runtimes other than `tokio`. As an alternative, we offer the feature that uses [`isahc`](https://crates.io/crates/isahc) as the backend.

```toml
[dependencies]
atrium-xrpc-client = { version = "*", default-features = false, features = ["isahc"]}
```

```rust
use atrium_xrpc_client::isahc::IsahcClient;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = IsahcClient::new("https://bsky.social");
    Ok(())
}
```

Similarly, you can directly specify an isahc::HttpClient with your own settings:

```toml
[dependencies]
atrium-xrpc-client = { version = "*", default-features = false, features = ["isahc"]}
isahc = "1.7.2"
```

```rust
use atrium_xrpc_client::isahc::IsahcClientBuilder;
use isahc::config::Configurable;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = IsahcClientBuilder::new("https://bsky.social")
        .client(
            isahc::HttpClientBuilder::new()
                .timeout(std::time::Duration::from_millis(1000))
                .build()?,
        )
        .build();
    Ok(())
}
```

For more details, refer to the [`isahc` documentation](https://docs.rs/isahc).


### `surf`

For cases such as using `rustls` with asynchronous runtimes other than `tokio`, we also provide a feature that uses [`surf`](https://crates.io/crates/surf) built with [`async-std`](https://crates.io/crates/async-std) as a backend.

Using `DefaultClient` with `surf` is complicated by the various feature flags. Therefore, unlike the first two options, you must always specify surf::Client when creating a client with this module.

```toml
[dependencies]
atrium-xrpc-client = { version = "*", default-features = false, features = ["surf"]}
surf = { version = "2.3.2", default-features = false, features = ["h1-client-rustls"] }
```

```rust
use atrium_xrpc_client::surf::SurfClient;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = SurfClient::new("https://bsky.social", surf::Client::new());
    Ok(())
}
```

Using [`http_client`](https://crates.io/crates/http-client) and its bundled implementation may clarify which backend you are using:

```toml
[dependencies]
atrium-xrpc-client = { version = "*", default-features = false, features = ["surf"]}
surf = { version = "2.3.2", default-features = false }
http-client = { version = "6.5.3", default-features = false, features = ["h1_client", "rustls"] }
```

```rust
use atrium_xrpc_client::surf::SurfClient;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = SurfClient::new(
        "https://bsky.social",
        surf::Client::with_http_client(http_client::h1::H1Client::try_from(
            http_client::Config::default()
                .set_timeout(Some(std::time::Duration::from_millis(1000))),
        )?),
    );
    Ok(())
}
```

For more details, refer to the [`surf` documentation](https://docs.rs/surf).
