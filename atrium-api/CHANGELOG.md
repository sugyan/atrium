# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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
