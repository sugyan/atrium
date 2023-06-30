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

### [`atrium-lex`](./atrium-lex/)

A library that provides type definitions for parsing the AT Protocol's [Lexicon](https://atproto.com/guides/lexicon) schema, ensuring compatibility with the lexicon.

### [`atrium-codegen`](./atrium-codegen/)

A library that generates Rust code for the `atrium-api` based on the analyzed lexicon definitions.

### [`atrium-xrpc`](./atrium-xrpc/)

[![](https://img.shields.io/crates/v/atrium-xrpc)](https://crates.io/crates/atrium-xrpc)
[![](https://img.shields.io/docsrs/atrium-xrpc)](https://docs.rs/atrium-xrpc)

A client library that offers a convenient way to interact with the `atrium-api` and utilize its features.

### [`atrium-cli`](./atrium-cli/)

A command-line app using this API library.

### `lexgen` command

```sh
cargo run --bin lexgen -- --lexdir $HOME/.ghq/github.com/bluesky-social/atproto/lexicons
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
