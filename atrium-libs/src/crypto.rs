pub mod did;
pub mod error;
pub mod multibase;
mod utils;

const DID_KEY_PREFIX: &str = "did:key:";

#[derive(Debug)]
pub enum JwtAlg {
    P256,
    Secp256k1,
}
