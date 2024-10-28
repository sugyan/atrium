# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.5.9](https://github.com/sugyan/atrium/compare/atrium-xrpc-client-v0.5.8...atrium-xrpc-client-v0.5.9) - 2024-10-28

### Other

- update Cargo.toml dependencies
## [0.5.8](https://github.com/sugyan/atrium/compare/atrium-xrpc-client-v0.5.7...atrium-xrpc-client-v0.5.8) - 2024-09-20

### Removed
- remove async_trait crate due to increased MSRV ([#234](https://github.com/sugyan/atrium/pull/234)) by @Elaina
## [0.5.7](https://github.com/sugyan/atrium/compare/atrium-xrpc-client-v0.5.6...atrium-xrpc-client-v0.5.7) - 2024-09-20

### Other
- Proposed fix: configuring and formatting project. ([#229](https://github.com/sugyan/atrium/pull/229)) by @Elaina

## [0.5.6](https://github.com/sugyan/atrium/compare/atrium-xrpc-client-v0.5.5...atrium-xrpc-client-v0.5.6) - 2024-08-13

### Added
- Add `atrium-crypto` ([#169](https://github.com/sugyan/atrium/pull/169))

### Fixed
- Remove Arc from xrpc-clients ([#206](https://github.com/sugyan/atrium/pull/206))

## [0.5.5](https://github.com/sugyan/atrium/compare/atrium-xrpc-client-v0.5.4...atrium-xrpc-client-v0.5.5) - 2024-06-13

### Added
- Add bsky-sdk ([#185](https://github.com/sugyan/atrium/pull/185))

## [0.5.4](https://github.com/sugyan/atrium/compare/atrium-xrpc-client-v0.5.3...atrium-xrpc-client-v0.5.4) - 2024-05-22

### Added
- Add supporting atproto headers ([#175](https://github.com/sugyan/atrium/pull/175))

## [0.5.3](https://github.com/sugyan/atrium/compare/atrium-xrpc-client-v0.5.2...atrium-xrpc-client-v0.5.3) - 2024-05-20

### Other
- update Cargo.toml dependencies

## [0.5.2](https://github.com/sugyan/atrium/compare/atrium-xrpc-client-v0.5.1...atrium-xrpc-client-v0.5.2) - 2024-04-17

### Added
- Upgrade `http` crate to 1.1 ([#152](https://github.com/sugyan/atrium/pull/152))

## [0.5.1](https://github.com/sugyan/atrium/compare/atrium-xrpc-client-v0.5.0...atrium-xrpc-client-v0.5.1) - 2024-03-27

### Other
- update Cargo.toml dependencies

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
