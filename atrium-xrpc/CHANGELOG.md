# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.10.3](https://github.com/sugyan/atrium/compare/atrium-xrpc-v0.10.2...atrium-xrpc-v0.10.3) - 2024-03-16

### Added
- implement `std::fmt::Display` for all Error types ([#140](https://github.com/sugyan/atrium/pull/140))

## [0.10.2](https://github.com/sugyan/atrium/compare/atrium-xrpc-v0.10.1...atrium-xrpc-v0.10.2) - 2024-03-10

### Other
- update Cargo.toml dependencies

## [0.10.1](https://github.com/sugyan/atrium/compare/atrium-xrpc-v0.10.0...atrium-xrpc-v0.10.1) - 2024-03-05

### Other
- update Cargo.toml dependencies

## [0.10.0](https://github.com/sugyan/atrium/compare/atrium-xrpc-v0.9.0...atrium-xrpc-v0.10.0) - 2024-02-29

### Added
- Support wasm32 ([#119](https://github.com/sugyan/atrium/pull/119))

### Changed
- For traits defined using `async_trait`, the `Send` bound is now optional with `wasm32-*` targets.

## [0.9.0](https://github.com/sugyan/atrium/compare/atrium-xrpc-v0.8.0...atrium-xrpc-v0.9.0) - 2024-02-20

### Other
- Move other dependencies into workspace dependencies table
- Deduplicate package keys with workspace inheritance
- Set MSRV for main crates to 1.70

## [0.8.0](https://github.com/sugyan/atrium/compare/atrium-xrpc-v0.7.0...atrium-xrpc-v0.8.0) - 2023-11-22

### Added
- Update return type of base_uri ([#82](https://github.com/sugyan/atrium/pull/82))

## [0.7.0](https://github.com/sugyan/atrium/compare/atrium-xrpc-v0.6.0...atrium-xrpc-v0.7.0) - 2023-11-12

### Added
- Make `XrpcClient::auth` asynchronous ([#72](https://github.com/sugyan/atrium/pull/72))

## [0.6.0](https://github.com/sugyan/atrium/compare/atrium-xrpc-v0.5.0...atrium-xrpc-v0.6.0) - 2023-11-10

### Added
- *(xrpc)* Remove client implementations from XRPC ([#68](https://github.com/sugyan/atrium/pull/68))

## [0.5.0](https://github.com/sugyan/atrium/compare/atrium-xrpc-v0.4.1...atrium-xrpc-v0.5.0) - 2023-11-09

### Added
- *(xrpc)* Rename XrpcClient method: `host` to `base_uri` ([#64](https://github.com/sugyan/atrium/pull/64))

## [0.4.1](https://github.com/sugyan/atrium/compare/atrium-xrpc-v0.4.0...atrium-xrpc-v0.4.1) - 2023-11-02

### Added
- *(xrpc)* Update XRPC interface ([#55](https://github.com/sugyan/atrium/pull/55))

## [0.4.0](https://github.com/sugyan/atrium/compare/atrium-xrpc-v0.3.0...atrium-xrpc-v0.4.0) - 2023-08-21

### Added
- Change trait method names ([#40](https://github.com/sugyan/atrium/pull/40))

## [0.3.0](https://github.com/sugyan/atrium/compare/atrium-xrpc-v0.2.0...atrium-xrpc-v0.3.0) - 2023-06-27

### Other
- Update docs
- Fix default implementation of XrpcClient
- Fix tests
- Update XrpcClient trait, add tests
- Update XrpcClient trait

## [0.2.0](https://github.com/sugyan/atrium/compare/atrium-xrpc-v0.1.0...atrium-xrpc-v0.2.0) - 2023-06-11

### Other
- Fix reference to serde_qs::ser::Error
- Fix broken array URL encoding
- release

## [0.1.0](https://github.com/sugyan/atrium/releases/tag/atrium-xrpc-v0.1.0) - 2023-06-07

### Added
- update xrpc

### Other
- Update atrium-api to 0.3
- Update README
- Update cli
- Update codegen, use macro
- Update codegen and api, add create-record to cli
- Rename project
