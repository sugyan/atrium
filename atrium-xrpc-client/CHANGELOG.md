# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.5.0](https://github.com/sugyan/atrium/compare/atrium-xrpc-client-v0.4.0...atrium-xrpc-client-v0.5.0) - 2024-03-05

### Removed
- Remove `surf` client ([#130](https://github.com/sugyan/atrium/pull/130))

## [0.4.0](https://github.com/sugyan/atrium/compare/atrium-xrpc-client-v0.3.0...atrium-xrpc-client-v0.4.0) - 2024-02-29

### Added
- Support wasm32 ([#119](https://github.com/sugyan/atrium/pull/119))
  - WASM support with `reqwest::ReqwestClient`

### Changed
- `reqwest-native` feature was renamed to `reqwest-default-tls`
- `reqwest-rustls` feature was removed. Use `reqwest` feature and `reqwest` crate to configure yourself.

## [0.3.0](https://github.com/sugyan/atrium/compare/atrium-xrpc-client-v0.2.0...atrium-xrpc-client-v0.3.0) - 2024-02-20

### Added
- Update API, based on the latest lexicon schemas ([#104](https://github.com/sugyan/atrium/pull/104))

### Other
- Move other dependencies into workspace dependencies table
- Move intra-workspace dependencies into workspace dependencies table
- Deduplicate package keys with workspace inheritance
- Set MSRV for main crates to 1.70

## [0.2.0](https://github.com/sugyan/atrium/compare/atrium-xrpc-client-v0.1.2...atrium-xrpc-client-v0.2.0) - 2023-11-22

### Added
- Update xrpc version, fix base_uri ([#84](https://github.com/sugyan/atrium/pull/84))

## [0.1.2](https://github.com/sugyan/atrium/compare/atrium-xrpc-client-v0.1.1...atrium-xrpc-client-v0.1.2) - 2023-11-12

### Added
- Update dependencies ([#74](https://github.com/sugyan/atrium/pull/74))

## [0.1.1](https://github.com/sugyan/atrium/compare/atrium-xrpc-client-v0.1.0...atrium-xrpc-client-v0.1.1) - 2023-11-10

### Other
- Update README
- release

## [0.1.0](https://github.com/sugyan/atrium/releases/tag/atrium-xrpc-client-v0.1.0) - 2023-11-10

### Added
- Add xrpc-client package ([#63](https://github.com/sugyan/atrium/pull/63))
