# ATrium XRPC

[![](https://img.shields.io/crates/v/atrium-xrpc)](https://crates.io/crates/atrium-xrpc)
[![](https://img.shields.io/docsrs/atrium-xrpc)](https://docs.rs/atrium-xrpc)
[![](https://img.shields.io/crates/l/atrium-xrpc)](https://github.com/sugyan/atrium/blob/main/LICENSE)
[![Rust](https://github.com/sugyan/atrium/actions/workflows/xrpc.yml/badge.svg?branch=main)](https://github.com/sugyan/atrium/actions/workflows/xrpc.yml)

Definitions for ATProto's [XRPC](https://atproto.com/specs/xrpc) request/response, and their associated errors.

The `XrpcClient` trait inherits from and uses `HttpClient` to provide a default implementation for handling XRPC requests. So developers can create their own Client for XRPC by implementing an `HttpClient` that sends asynchronous HTTP requests according to this interface.
