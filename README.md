# ATrium

ATrium is a collection of Rust libraries designed to work with the [AT Protocol](https://atproto.com/), providing a versatile and coherent ecosystem for developers. The name is inspired by the concept of an "atrium" with a view of the open [bluesky](https://bsky.app/), reflecting the open nature of the project.

Our goal is to provide a comprehensive, easy-to-use, and efficient library that caters to various use cases and scenarios involving the AT Protocol.

Please note that ATrium is still under active development and many features may be subject to change or enhancement. We appreciate your understanding and patience during this phase.

## Overview

ATrium is divided into several sub-projects to address different aspects of the AT Protocol and provide a modular design:

### [`atrium-api`](./atrium-api/)

[![](https://img.shields.io/crates/v/atrium-api)](https://crates.io/crates/atrium-api)
[![](https://img.shields.io/docsrs/atrium-api)](https://docs.rs/atrium-api)

A library consisting of models and messaging definitions for XRPC, primarily generated using the codegen library.

### [`atrium-xrpc`](./atrium-xrpc/)

[![](https://img.shields.io/crates/v/atrium-xrpc)](https://crates.io/crates/atrium-xrpc)
[![](https://img.shields.io/docsrs/atrium-xrpc)](https://docs.rs/atrium-xrpc)

Definitions for XRPC request/response, and their associated errors.

### [`atrium-xrpc-client`](./atrium-xrpc-client/)

[![](https://img.shields.io/crates/v/atrium-xrpc-client)](https://crates.io/crates/atrium-xrpc-client)
[![](https://img.shields.io/docsrs/atrium-xrpc-client)](https://docs.rs/atrium-xrpc-client)

A library provides clients that implement the `XrpcClient` defined in [atrium-xrpc](./atrium-xrpc/)

### [`atrium-xrpc-wss`](./atrium-xrpc-wss/)

Definitions for traits, types and utilities for dealing with WebSocket XRPC subscriptions. (WIP)

### [`atrium-xrpc-wss-client`](./atrium-xrpc-wss-client/)

A library that provides default implementations of the `XrpcWssClient`, `Handlers` and `Subscription` defined in [atrium-xrpc-wss](./atrium-xrpc-wss/) for interacting with the variety of subscriptions in ATProto (WIP)

### [`bsky-sdk`](./bsky-sdk/)

[![](https://img.shields.io/crates/v/bsky-sdk)](https://crates.io/crates/bsky-sdk)

ATrium-based SDK for Bluesky.

### [`bsky-cli`](./bsky-cli/)

[![](https://img.shields.io/crates/v/bsky-cli)](https://crates.io/crates/bsky-cli)

A command-line app using this API library.

## Code generation

The models and messaging definitions for XRPC are generated with these crates:

### [`atrium-lex`](./lexicon/atrium-lex/)

A library that provides type definitions for parsing the AT Protocol's [Lexicon](https://atproto.com/guides/lexicon) schema, ensuring compatibility with the lexicon.

### [`atrium-codegen`](./lexicon/atrium-codegen/)

A library that generates Rust code for the `atrium-api` based on the analyzed lexicon definitions.

### `lexgen` command

```sh
cd lexicon && cargo run -p lexgen -- --lexdir $HOME/.ghq/github.com/bluesky-social/atproto/lexicons
```

## Contribution

We welcome contributions from the community to help us improve and expand ATrium. If you're interested in contributing, please feel free to submit issues or pull requests on the GitHub repository. We appreciate your support!

## License

ATrium is released under the [MIT License](./LICENSE).

## Related works

Below are some related projects that might be of interest:

- `atproto` https://github.com/bluesky-social/atproto
  - The leading protocol implementation
- `adenosine` https://gitlab.com/bnewbold/adenosine
- `atproto-rs` https://github.com/ngerakines/atproto-rs
- `atproto-rs` https://github.com/Maaarcocr/atproto-rs
- `bisky` https://github.com/jesopo/bisky
- `lexicon-rs` https://github.com/Matrix89/lexicon-rs
