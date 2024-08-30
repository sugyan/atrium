pub mod jws;
pub mod jwt;
pub mod signing;

pub use self::signing::create_signed_jwt;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Header {
    Jws(jws::Header),
    // TODO: JWE?
}

#[cfg(test)]
mod tests {
    use super::*;

    // #[test]
    // fn test_create_jwt() {
    //     let secret_key = SecretKey::<p256::NistP256>::from_slice(&[
    //         178, 249, 128, 41, 213, 198, 33, 120, 72, 132, 129, 161, 128, 134, 36, 120, 199, 128,
    //         234, 73, 217, 232, 94, 120, 78, 231, 64, 117, 105, 239, 160, 251,
    //     ])
    //     .expect("failed to create secret key");
    //     panic!("{:?}", create_jwt(&secret_key.into(), JwtClaims::default()));
    // }
}
