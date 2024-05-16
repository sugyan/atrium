# ATrium Crypto

Cryptographic library providing basic helpers for AT Protocol.

This package implements the two currently supported cryptographic systems:

- [`p256`](https://crates.io/crates/p256) elliptic curve: aka "NIST P-256", aka `secp256r1` (note the `r`), aka `prime256v1`
- [`k256`](https://crates.io/crates/k256) elliptic curve: aka "NIST K-256", aka `secp256k1` (note the `k`)

The details of cryptography in atproto are described in [the specification](https://atproto.com/specs/cryptography). This includes string encodings, validity of "low-S" signatures, byte representation "compression", hashing, and more.

## Usage

```rust
use atrium_crypto::keypair::{Secp256k1Keypair, Did};
use atrium_crypto::verify::verify_signature;
use rand::rngs::ThreadRng;

fn main() -> Result<(), Box<dyn std::error::Error>>{
    // generate a new random K-256 private key
    let keypair = Secp256k1Keypair::create(&mut ThreadRng::default());

    // sign binary data, resulting signature bytes.
    // SHA-256 hash of data is what actually gets signed.
    let msg = [1, 2, 3, 4, 5, 6, 7, 8];
    let signature = keypair.sign(&msg)?;

    // serialize the public key as a did:key string, which includes key type metadata
    let pub_did_key = keypair.did();
    println!("{pub_did_key}");
    // output would look something like: 'did:key:zQ3shVRtgqTRHC7Lj4DYScoDgReNpsDp3HBnuKBKt1FSXKQ38'

    // verify signature using public key
    match verify_signature(&pub_did_key, &msg, &signature) {
        Ok(()) => println!("Success"),
        Err(_) => panic!("Uh oh, something is fishy"),
    }
    Ok(())
}
```
