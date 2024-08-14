# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.24.1](https://github.com/sugyan/atrium/compare/atrium-api-v0.24.0...atrium-api-v0.24.1) - 2024-08-14

### Fixed
- Fix serialization of Unknown(DataModel) ([#214](https://github.com/sugyan/atrium/pull/214))

## [0.24.0](https://github.com/sugyan/atrium/compare/atrium-api-v0.23.2...atrium-api-v0.24.0) - 2024-08-14

### Added
- Update API, based on the latest lexicon schemas ([#210](https://github.com/sugyan/atrium/pull/210))
- Introduce atrium_api::types::Unknown for unknown fields  ([#209](https://github.com/sugyan/atrium/pull/209))
  - Add `atrium_api::types::Unknown`
- Add `atrium-crypto` ([#169](https://github.com/sugyan/atrium/pull/169))

### Changed
- `unknown` field types that don't have a well-known format now have the type
  `atrium_api::types::Unknown` instead of `atrium_api::records::Record`.

## [0.23.2](https://github.com/sugyan/atrium/compare/atrium-api-v0.23.1...atrium-api-v0.23.2) - 2024-07-03

### Added
- Update API, based on the latest lexicon schemas ([#195](https://github.com/sugyan/atrium/pull/195))

## [0.23.1](https://github.com/sugyan/atrium/compare/atrium-api-v0.23.0...atrium-api-v0.23.1) - 2024-06-26

### Added
- Update API, based on the latest lexicon schemas ([#194](https://github.com/sugyan/atrium/pull/194))
- Add `Clone` and `Debug` ([#193](https://github.com/sugyan/atrium/pull/193))

## [0.23.0](https://github.com/sugyan/atrium/compare/atrium-api-v0.22.3...atrium-api-v0.23.0) - 2024-06-20

### Added
- Update bsky-sdk, and redesign of object types in API ([#189](https://github.com/sugyan/atrium/pull/189))
- Update API, based on the latest lexicon schemas ([#188](https://github.com/sugyan/atrium/pull/188))

## [0.22.3](https://github.com/sugyan/atrium/compare/atrium-api-v0.22.2...atrium-api-v0.22.3) - 2024-06-13

### Added
- Add bsky-sdk ([#185](https://github.com/sugyan/atrium/pull/185))

## [0.22.2](https://github.com/sugyan/atrium/compare/atrium-api-v0.22.1...atrium-api-v0.22.2) - 2024-05-27

### Added
- Update api ([#182](https://github.com/sugyan/atrium/pull/182))

## [0.22.1](https://github.com/sugyan/atrium/compare/atrium-api-v0.22.0...atrium-api-v0.22.1) - 2024-05-23

### Added
- Update default features of API ([#181](https://github.com/sugyan/atrium/pull/181))
- Generate lexicon token as const &str ([#179](https://github.com/sugyan/atrium/pull/179))

### Other
- Add methods to retrieve AtpAgent info ([#178](https://github.com/sugyan/atrium/pull/178))

## [0.22.0](https://github.com/sugyan/atrium/compare/atrium-api-v0.21.3...atrium-api-v0.22.0) - 2024-05-22

### Added
- Add supporting atproto headers ([#175](https://github.com/sugyan/atrium/pull/175))

## [0.21.3](https://github.com/sugyan/atrium/compare/atrium-api-v0.21.2...atrium-api-v0.21.3) - 2024-05-20

### Added
- *(api)* Add namespace features ([#174](https://github.com/sugyan/atrium/pull/174))

## [0.21.2](https://github.com/sugyan/atrium/compare/atrium-api-v0.21.1...atrium-api-v0.21.2) - 2024-05-17

### Added
- Update API, based on the latest lexicon schemas ([#171](https://github.com/sugyan/atrium/pull/171))

## [0.21.1](https://github.com/sugyan/atrium/compare/atrium-api-v0.21.0...atrium-api-v0.21.1) - 2024-05-17

### Added
- Add headers() to `XrpcClient` ([#170](https://github.com/sugyan/atrium/pull/170))
- Update API, based on the latest lexicon schemas ([#165](https://github.com/sugyan/atrium/pull/165))

## [0.21.0](https://github.com/sugyan/atrium/compare/atrium-api-v0.20.1...atrium-api-v0.21.0) - 2024-04-18

### Added
- Add tid/record-key string format ([#155](https://github.com/sugyan/atrium/pull/155))
  - `atrium_api::types::string::Tid`
  - `atrium_api::types::string::RecordKey`
    - moved from `atrium_api::types::RecordKey`

### Removed
- `atrium_api::types::RecordKey`
  - moved to `atrium_api::types::string::RecordKey`

## [0.20.1](https://github.com/sugyan/atrium/compare/atrium-api-v0.20.0...atrium-api-v0.20.1) - 2024-04-17

### Added
- Update API, based on the latest lexicon schemas ([#157](https://github.com/sugyan/atrium/pull/157))

## [0.20.0](https://github.com/sugyan/atrium/compare/atrium-api-v0.19.0...atrium-api-v0.20.0) - 2024-03-27

### Added
- Introduce "open" union types ([#149](https://github.com/sugyan/atrium/pull/149))

## [0.19.0](https://github.com/sugyan/atrium/compare/atrium-api-v0.18.6...atrium-api-v0.19.0) - 2024-03-27

### Added
- Update dependencies ([#148](https://github.com/sugyan/atrium/pull/148))
- Add clippy check ([#146](https://github.com/sugyan/atrium/pull/146))

### Removed
- dag-cbor feature from API ([#147](https://github.com/sugyan/atrium/pull/147))

## [0.18.6](https://github.com/sugyan/atrium/compare/atrium-api-v0.18.5...atrium-api-v0.18.6) - 2024-03-16

### Added
- implement `std::fmt::Display` for all Error types ([#140](https://github.com/sugyan/atrium/pull/140))

## [0.18.5](https://github.com/sugyan/atrium/compare/atrium-api-v0.18.4...atrium-api-v0.18.5) - 2024-03-14

### Fixed
- Regenerating atrium-api and Dirty Fixing label/defs ([#138](https://github.com/sugyan/atrium/pull/138))

## [0.18.4](https://github.com/sugyan/atrium/compare/atrium-api-v0.18.3...atrium-api-v0.18.4) - 2024-03-13

### Added
- Update API, based on the latest lexicon schemas ([#134](https://github.com/sugyan/atrium/pull/134))

## [0.18.3](https://github.com/sugyan/atrium/compare/atrium-api-v0.18.2...atrium-api-v0.18.3) - 2024-03-10

### Other
- update Cargo.toml dependencies

## [0.18.2](https://github.com/sugyan/atrium/compare/atrium-api-v0.18.1...atrium-api-v0.18.2) - 2024-03-05

### Other
- update Cargo.toml dependencies

## [0.18.1](https://github.com/sugyan/atrium/compare/atrium-api-v0.18.0...atrium-api-v0.18.1) - 2024-03-03

### Other
- Generate structs corresponding to collections

### Added
- `atrium_api::types::Collection` trait, which binds together a record type and its NSID.
- Collection structs for the current record types:
  - `atrium_api::app::bsky::actor::Profile`
  - `atrium_api::app::bsky::feed`:
    - `Generator`
    - `Like`
    - `Post`
    - `Repost`
    - `Threadgate`
  - `atrium_api::app::bsky::graph`:
    - `Block`
    - `Follow`
    - `List`
    - `Listblock`
    - `Listitem`

## [0.18.0](https://github.com/sugyan/atrium/compare/atrium-api-v0.17.2...atrium-api-v0.18.0) - 2024-02-29

### Added
- Update API, based on the latest lexicon schemas ([#123](https://github.com/sugyan/atrium/pull/123))
- Support wasm32 ([#119](https://github.com/sugyan/atrium/pull/119))

### Changed
- For traits defined using `async_trait`, the `Send` bound is now optional with `wasm32-*` targets.

### Fixed
- `atrium_api::types::string::{Cid, Datetime}` can now be deserialized with `serde`. ([#121](https://github.com/sugyan/atrium/pull/121))

## [0.17.2](https://github.com/sugyan/atrium/compare/atrium-api-v0.17.1...atrium-api-v0.17.2) - 2024-02-21

### Other
- update Cargo.toml dependencies

## [0.17.1](https://github.com/sugyan/atrium/compare/atrium-api-v0.17.0...atrium-api-v0.17.1) - 2024-02-20

### Other
- update Cargo.toml dependencies

## [0.17.0](https://github.com/sugyan/atrium/compare/atrium-api-v0.16.0...atrium-api-v0.17.0) - 2024-02-20

### Added
- Update API, based on the latest lexicon schemas ([#104](https://github.com/sugyan/atrium/pull/104))

### Other
- Merge pull request [#110](https://github.com/sugyan/atrium/pull/110) from str4d/lexicon-integer-min-max
- Add `MIN, MAX` associated constants to Lexicon integer types
- Merge pull request [#107](https://github.com/sugyan/atrium/pull/107) from str4d/lexicon-integer-conversion
- Add direct conversions between the Lexicon integer types and primitives
- Introduce dedicated types for DID and handle Lexicon string formats
- Introduce types guaranteed to fit the range of each Lexicon integer
- Move other dependencies into workspace dependencies table
- Move intra-workspace dependencies into workspace dependencies table
- Deduplicate package keys with workspace inheritance
- Set MSRV for main crates to 1.70

### Added
- `atrium_api::types`:
  - `RecordKey`
  - `LimitedU8`, `LimitedNonZeroU8`, `BoundedU8`
  - `LimitedU16`, `LimitedNonZeroU16`, `BoundedU16`
  - `LimitedU32`, `LimitedNonZeroU32`, `BoundedU32`
  - `LimitedU64`, `LimitedNonZeroU64`, `BoundedU64`
  - `string` module, containing dedicated types for formatted Lexicon strings.

### Changed
- All Lexicon integer fields now have a type that matches their minimum and maximum
  accepted values, instead of `i32`.
- All Lexicon string fields with one of the following formats now have the corresponding
  dedicated type, instead of `String`:
  - `at-identifier` (`atrium_api::types::string::AtIdentifier`)
  - `cid` (`atrium_api::types::string::Cid`)
  - `datetime` (`atrium_api::types::string::Datetime`)
  - `did` (`atrium_api::types::string::Did`)
  - `handle` (`atrium_api::types::string::Handle`)
  - `nsid` (`atrium_api::types::string::Nsid`)
  - `language` (`atrium_api::types::string::Language`)

## [0.16.0](https://github.com/sugyan/atrium/compare/atrium-api-v0.15.0...atrium-api-v0.16.0) - 2024-02-09

### Added
- Update API, based on the latest lexicon schemas ([#99](https://github.com/sugyan/atrium/pull/99))
- *(api)* Implement CidLink, BlobRef types ([#96](https://github.com/sugyan/atrium/pull/96))

## [0.15.0](https://github.com/sugyan/atrium/compare/atrium-api-v0.14.0...atrium-api-v0.15.0) - 2023-12-23

### Added
- Update API, based on the latest lexicon schemas ([#92](https://github.com/sugyan/atrium/pull/92))

## [0.14.0](https://github.com/sugyan/atrium/compare/atrium-api-v0.13.1...atrium-api-v0.14.0) - 2023-11-23

### Added
- Switch PDS API endpoint dynamically ([#88](https://github.com/sugyan/atrium/pull/88))

## [0.13.1](https://github.com/sugyan/atrium/compare/atrium-api-v0.13.0...atrium-api-v0.13.1) - 2023-11-22

### Added
- Update xrpc and xrpc-client version ([#86](https://github.com/sugyan/atrium/pull/86))

## [0.13.0](https://github.com/sugyan/atrium/compare/atrium-api-v0.12.0...atrium-api-v0.13.0) - 2023-11-13

### Added
- Allow AtpAgent to be excluded as a default feature ([#79](https://github.com/sugyan/atrium/pull/79))
- Update xprc, use tokio::sync::RwLock for agent ([#76](https://github.com/sugyan/atrium/pull/76))

### Fixed
- Update formatter ([#80](https://github.com/sugyan/atrium/pull/80))

## [0.12.0](https://github.com/sugyan/atrium/compare/atrium-api-v0.11.0...atrium-api-v0.12.0) - 2023-11-11

### Added
- Update API, based on the latest lexicon schemas ([#69](https://github.com/sugyan/atrium/pull/69))

### Fixed
- Add DidDocument ([#71](https://github.com/sugyan/atrium/pull/71))

## [0.11.0](https://github.com/sugyan/atrium/compare/atrium-api-v0.10.0...atrium-api-v0.11.0) - 2023-11-05

### Added
- *(api)* Implement refresh_session wrapper ([#60](https://github.com/sugyan/atrium/pull/60))

## [0.10.0](https://github.com/sugyan/atrium/compare/atrium-api-v0.9.2...atrium-api-v0.10.0) - 2023-11-02

### Added
- Update API, based on the latest lexicon schemas ([#58](https://github.com/sugyan/atrium/pull/58))

## [0.9.2](https://github.com/sugyan/atrium/compare/atrium-api-v0.9.1...atrium-api-v0.9.2) - 2023-11-02

### Added
- Implement AtpAgent ([#53](https://github.com/sugyan/atrium/pull/53))

## [0.9.1](https://github.com/sugyan/atrium/compare/atrium-api-v0.9.0...atrium-api-v0.9.1) - 2023-10-28

### Other
- Delegate to inner in AtpServiceWrapper ([#52](https://github.com/sugyan/atrium/pull/52))

## [0.9.0](https://github.com/sugyan/atrium/compare/atrium-api-v0.8.0...atrium-api-v0.9.0) - 2023-10-06

### Added
- Update API, based on the latest lexicon schemas ([#49](https://github.com/sugyan/atrium/pull/49))

## [0.8.0](https://github.com/sugyan/atrium/compare/atrium-api-v0.7.0...atrium-api-v0.8.0) - 2023-09-25

### Other
- Update API, based on the latest lexicon schemas ([#47](https://github.com/sugyan/atrium/pull/47))

## [0.7.0](https://github.com/sugyan/atrium/compare/atrium-api-v0.6.0...atrium-api-v0.7.0) - 2023-09-13

### Added
- Update API from latest lexicon schemas ([#45](https://github.com/sugyan/atrium/pull/45))

## [0.6.0](https://github.com/sugyan/atrium/compare/atrium-api-v0.5.0...atrium-api-v0.6.0) - 2023-08-29

### Other
- Update API from latest lexicon schemas ([#43](https://github.com/sugyan/atrium/pull/43))

## [0.5.0](https://github.com/sugyan/atrium/compare/atrium-api-v0.4.1...atrium-api-v0.5.0) - 2023-08-28

### Added
- Update API client ([#41](https://github.com/sugyan/atrium/pull/41))

## [0.4.1](https://github.com/sugyan/atrium/compare/atrium-api-v0.4.0...atrium-api-v0.4.1) - 2023-08-21

### Added
- re-export atrium_xrpc as xrpc ([#35](https://github.com/sugyan/atrium/pull/35))

### Other
- remove unused codes

## [0.4.0](https://github.com/sugyan/atrium/compare/atrium-api-v0.3.2...atrium-api-v0.4.0) - 2023-06-30

### Added
- update api, add client/agent ([#32](https://github.com/sugyan/atrium/pull/32))

## [0.3.2](https://github.com/sugyan/atrium/compare/atrium-api-v0.3.1...atrium-api-v0.3.2) - 2023-06-14

### Other
- Update atrium_api::xrpc::XrpcClient for refresh

## [0.3.1](https://github.com/sugyan/atrium/compare/atrium-api-v0.3.0...atrium-api-v0.3.1) - 2023-06-11

### Other
- Fix broken array URL encoding

## [0.3.0](https://github.com/sugyan/atrium/compare/atrium-api-v0.2.0...atrium-api-v0.3.0) - 2023-05-22

### Added
- *(api)* Re-export `http` in `xrpc` module
- *(api)* Update API from latest lexicon schemas
- Implement subscription, add firehose examples (#15)

## [0.2.0](https://github.com/sugyan/atrium/compare/atrium-api-v0.1.1...atrium-api-v0.2.0) - 2023-05-13

### Added
- *(api)* Update API from latest lexicon schemas

### Other
- Update README
- Update atrium-api by new codegen
- Update atrium-api by new codegen

## [0.1.1](https://github.com/sugyan/atrium/compare/atrium-api-v0.1.0...atrium-api-v0.1.1) - 2023-05-07

### Other
- Apply `rustfmt` to generated codes
- Update code_writer
- Update readme (#5)
- Update README, workflows (#4)
