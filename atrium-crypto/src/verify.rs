use crate::{did::parse_did_key, error::Result};

pub fn verify_signature(did_key: &str, msg: &[u8], signature: &[u8]) -> Result<()> {
    let (alg, public_key) = parse_did_key(did_key)?;
    alg.verify_signature(&public_key, msg, signature)
}
