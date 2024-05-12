# ATrium Crypto

Cryptographic library providing basic helpers for AT Protocol.

This package implements the two currently supported cryptographic systems:

- [`p256`](https://crates.io/crates/p256) elliptic curve: aka "NIST P-256", aka `secp256r1` (note the `r`), aka `prime256v1`
- [`k256`](https://crates.io/crates/k256) elliptic curve: aka "NIST K-256", aka `secp256k1` (note the `k`)

The details of cryptography in atproto are described in [the specification](https://atproto.com/specs/cryptography). This includes string encodings, validity of "low-S" signatures, byte representation "compression", hashing, and more.
