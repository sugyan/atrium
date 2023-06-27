# ATrium XRPC

[![](https://img.shields.io/crates/v/atrium-xrpc)](https://crates.io/crates/atrium-xrpc)
[![](https://img.shields.io/docsrs/atrium-xrpc)](https://docs.rs/atrium-xrpc)
[![](https://img.shields.io/crates/l/atrium-xrpc)](https://github.com/sugyan/atrium/blob/main/LICENSE)
[![Rust](https://github.com/sugyan/atrium/actions/workflows/xrpc.yml/badge.svg?branch=main)](https://github.com/sugyan/atrium/actions/workflows/xrpc.yml)

Definitions for ATProto's [XRPC](https://atproto.com/specs/xrpc) request/response, and their associated errors.
And a client using [`reqwest`](https://crates.io/crates/reqwest) that can be used as its default implementation is included.

```rust
use atrium_xrpc::client::reqwest::ReqwestClient;

let client = ReqwestClient::new("https://bsky.social".into());
```
