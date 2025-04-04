# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.19](https://github.com/atrium-rs/atrium/compare/bsky-sdk-v0.1.18...bsky-sdk-v0.1.19) - 2025-04-04

### Other

- Replace repository owner ([#301](https://github.com/atrium-rs/atrium/pull/301))

## [0.1.18](https://github.com/sugyan/atrium/compare/bsky-sdk-v0.1.17...bsky-sdk-v0.1.18) - 2025-04-02

### Other

- updated the following local packages: atrium-xrpc-client, atrium-api

## [0.1.17](https://github.com/sugyan/atrium/compare/bsky-sdk-v0.1.16...bsky-sdk-v0.1.17) - 2025-04-02

### Added

- Update generated API ([#298](https://github.com/sugyan/atrium/pull/298))

## [0.1.16](https://github.com/sugyan/atrium/compare/bsky-sdk-v0.1.15...bsky-sdk-v0.1.16) - 2025-02-17

### Added

- Agent rework (#282)

## [0.1.15](https://github.com/sugyan/atrium/compare/bsky-sdk-v0.1.14...bsky-sdk-v0.1.15) - 2025-01-21

### Other

- update schema based on current lexicon ([#276](https://github.com/sugyan/atrium/pull/276))

## [0.1.14](https://github.com/sugyan/atrium/compare/bsky-sdk-v0.1.13...bsky-sdk-v0.1.14) - 2024-12-10

### Other

- updated the following local packages: atrium-api

## [0.1.13](https://github.com/sugyan/atrium/compare/bsky-sdk-v0.1.12...bsky-sdk-v0.1.13) - 2024-11-19

### Other

- Update README
- Add example of how to create a Post on bsky ([#255](https://github.com/sugyan/atrium/pull/255))

## [0.1.12](https://github.com/sugyan/atrium/compare/bsky-sdk-v0.1.11...bsky-sdk-v0.1.12) - 2024-10-28

### Added

- Update API, based on the latest lexicon schemas ([#241](https://github.com/sugyan/atrium/pull/241))
- OAuth ([#219](https://github.com/sugyan/atrium/pull/219))
## [0.1.11](https://github.com/sugyan/atrium/compare/bsky-sdk-v0.1.10...bsky-sdk-v0.1.11) - 2024-09-20

### Removed
- remove async_trait crate due to increased MSRV ([#234](https://github.com/sugyan/atrium/pull/234)) by @Elaina
## [0.1.10](https://github.com/sugyan/atrium/compare/bsky-sdk-v0.1.9...bsky-sdk-v0.1.10) - 2024-09-20

### Other
- Bumping MSRV to 1.75 ([#233](https://github.com/sugyan/atrium/pull/233)) by @Elaina
- Proposed fix: configuring and formatting project. ([#229](https://github.com/sugyan/atrium/pull/229)) by @Elaina

## [0.1.9](https://github.com/sugyan/atrium/compare/bsky-sdk-v0.1.8...bsky-sdk-v0.1.9) - 2024-09-13

### Added

- Update API, based on the latest lexicon schemas ([#224](https://github.com/sugyan/atrium/pull/224))

## [0.1.8](https://github.com/sugyan/atrium/compare/bsky-sdk-v0.1.7...bsky-sdk-v0.1.8) - 2024-09-04

### Fixed
- Make bsky_sdk::error public ([#221](https://github.com/sugyan/atrium/pull/221))

## [0.1.7](https://github.com/sugyan/atrium/compare/bsky-sdk-v0.1.6...bsky-sdk-v0.1.7) - 2024-08-13

### Added
- Add expired/actor_target check to has_muted_word ([#211](https://github.com/sugyan/atrium/pull/211))
- Introduce atrium_api::types::Unknown for unknown fields  ([#209](https://github.com/sugyan/atrium/pull/209))

### Fixed
- Fix async_trait for SDK records ([#208](https://github.com/sugyan/atrium/pull/208))

## [0.1.6](https://github.com/sugyan/atrium/compare/bsky-sdk-v0.1.5...bsky-sdk-v0.1.6) - 2024-07-19

### Added
- *(bsky-sdk)* Add record operations ([#200](https://github.com/sugyan/atrium/pull/200))

## [0.1.5](https://github.com/sugyan/atrium/compare/bsky-sdk-v0.1.4...bsky-sdk-v0.1.5) - 2024-07-11

### Fixed
- Fix BskyAgent::moderator ([#199](https://github.com/sugyan/atrium/pull/199))

## [0.1.4](https://github.com/sugyan/atrium/compare/bsky-sdk-v0.1.3...bsky-sdk-v0.1.4) - 2024-07-03

### Added
- Update BskyAgent ([#197](https://github.com/sugyan/atrium/pull/197))

## [0.1.3](https://github.com/sugyan/atrium/compare/bsky-sdk-v0.1.2...bsky-sdk-v0.1.3) - 2024-06-26

### Added
- Update API, based on the latest lexicon schemas ([#194](https://github.com/sugyan/atrium/pull/194))
- Add `Clone` and `Debug` ([#193](https://github.com/sugyan/atrium/pull/193))

## [0.1.2](https://github.com/sugyan/atrium/compare/bsky-sdk-v0.1.1...bsky-sdk-v0.1.2) - 2024-06-21

### Added
- Rename atrium-cli to bsky-cli ([#191](https://github.com/sugyan/atrium/pull/191))

### Other
- Fix bsky-sdk/CHANGELOG

## [0.1.1](https://github.com/sugyan/atrium/compare/bsky-sdk-v0.1.0...bsky-sdk-v0.1.1) - 2024-06-19

### Added
- Update bsky-sdk, and redesign of object types in API ([#189](https://github.com/sugyan/atrium/pull/189))
- Update API, based on the latest lexicon schemas ([#188](https://github.com/sugyan/atrium/pull/188))

## [0.1.0](https://github.com/sugyan/atrium/releases/tag/bsky-sdk-v0.1.0) - 2024-06-13

### Added
- Add bsky-sdk ([#185](https://github.com/sugyan/atrium/pull/185))

### Other
- release
