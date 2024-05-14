use crate::error::Result;
use crate::keypair::verify_signature;
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

    pub fn prefix(&self) -> [u8; 2] {
        match self {
            Self::P256 => Self::MULTICODE_PREFIX_P256,
            Self::Secp256k1 => Self::MULTICODE_PREFIX_SECP256K1,
        }
    }
    pub fn from_prefix(prefix: [u8; 2]) -> Option<Self> {
        match prefix {
            Self::MULTICODE_PREFIX_P256 => Some(Self::P256),
            Self::MULTICODE_PREFIX_SECP256K1 => Some(Self::Secp256k1),
            _ => None,
        }
    }
    pub fn format_multikey(&self, key: &[u8]) -> Result<String> {
        Ok(self.format_mulikey_compressed(&self.pubkey_bytes(key, true)?))
    }
    pub(crate) fn format_mulikey_compressed(&self, key: &[u8]) -> String {
        let mut v = Vec::with_capacity(2 + key.len());
        v.extend_from_slice(&self.prefix());
        v.extend_from_slice(key);
        multibase::encode(Base::Base58Btc, v)
    }
    pub fn decompress_pubkey(&self, key: &[u8]) -> Result<Vec<u8>> {
        self.pubkey_bytes(key, false)
    }
    pub fn verify_signature(&self, public_key: &[u8], msg: &[u8], signature: &[u8]) -> Result<()> {
        match self {
            Algorithm::P256 => verify_signature::<NistP256>(public_key, msg, signature),
            Algorithm::Secp256k1 => verify_signature::<Secp256k1>(public_key, msg, signature),
        }
    }
    fn pubkey_bytes(&self, key: &[u8], compress: bool) -> Result<Vec<u8>> {
        Ok(match self {
            Algorithm::P256 => VerifyingKey::<NistP256>::from_sec1_bytes(key)?
                .to_encoded_point(compress)
                .as_bytes()
                .to_vec(),
            Algorithm::Secp256k1 => VerifyingKey::<Secp256k1>::from_sec1_bytes(key)?
                .to_encoded_point(compress)
                .as_bytes()
                .to_vec(),
        })
    }
}
