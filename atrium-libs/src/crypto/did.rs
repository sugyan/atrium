use super::error::{Error, Result};
use super::{JwtAlg, DID_KEY_PREFIX};

pub fn parse_multikey(multikey: &str) -> Result<(JwtAlg, Vec<u8>)> {
    let (_, decoded) = multibase::decode(multikey)?;
    match &decoded[..2] {
        [0x80, 0x24] => Ok((JwtAlg::P256, decoded[2..].to_vec())),
        [0xe7, 0x01] => Ok((JwtAlg::Secp256k1, decoded[2..].to_vec())),
        _ => Err(Error::UnsupportedMultikeyType),
    }
}

pub fn format_did_key(jwt_alg: JwtAlg, key: &[u8]) -> String {
    DID_KEY_PREFIX.to_string() + &format_multikey(jwt_alg, key)
}

fn format_multikey(jwt_alg: JwtAlg, key: &[u8]) -> String {
    todo!()
}
