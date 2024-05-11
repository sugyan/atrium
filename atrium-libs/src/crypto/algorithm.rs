use super::error::Result;
use ecdsa::VerifyingKey;
use k256::Secp256k1;
use multibase::Base;
use p256::NistP256;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Algorithm {
    P256,
    Secp256k1,
}

impl Algorithm {
    const MULTICODE_PREFIX_P256: [u8; 2] = [0x80, 0x24];
    const MULTICODE_PREFIX_SECP256K1: [u8; 2] = [0xe7, 0x01];

    pub fn from_prefix(prefix: [u8; 2]) -> Option<Self> {
        match prefix {
            Self::MULTICODE_PREFIX_P256 => Some(Self::P256),
            Self::MULTICODE_PREFIX_SECP256K1 => Some(Self::Secp256k1),
            _ => None,
        }
    }
    pub fn format_multikey(&self, key: &[u8]) -> Result<String> {
        let prefixed_bytes = match self {
            Algorithm::P256 => {
                let point = VerifyingKey::<NistP256>::from_sec1_bytes(key)?.to_encoded_point(true);
                [
                    Self::MULTICODE_PREFIX_P256.to_vec(),
                    point.as_bytes().to_vec(),
                ]
                .concat()
            }
            Algorithm::Secp256k1 => {
                let point = VerifyingKey::<Secp256k1>::from_sec1_bytes(key)?.to_encoded_point(true);
                [
                    Self::MULTICODE_PREFIX_SECP256K1.to_vec(),
                    point.as_bytes().to_vec(),
                ]
                .concat()
            }
        };
        Ok(multibase::encode(Base::Base58Btc, prefixed_bytes))
    }
    pub fn decompress_pubkey(&self, key: &[u8]) -> Result<Vec<u8>> {
        let point = match self {
            Algorithm::P256 => {
                let point = VerifyingKey::<NistP256>::from_sec1_bytes(key)?.to_encoded_point(false);
                point.to_bytes().to_vec()
            }
            Algorithm::Secp256k1 => {
                let point =
                    VerifyingKey::<Secp256k1>::from_sec1_bytes(key)?.to_encoded_point(false);
                point.to_bytes().to_vec()
            }
        };
        Ok(point)
    }
}
