use crate::error::{Error, Result};
use crate::keypair::verify_signature;
use crate::DID_KEY_PREFIX;
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
        let mut bytes = match self {
            Algorithm::P256 => Self::MULTICODE_PREFIX_P256,
            Algorithm::Secp256k1 => Self::MULTICODE_PREFIX_SECP256K1,
        }
        .to_vec();
        bytes.extend(self.pubkey_bytes(key, true)?);
        Ok(multibase::encode(Base::Base58Btc, bytes))
    }
    pub fn decompress_pubkey(&self, key: &[u8]) -> Result<Vec<u8>> {
        self.pubkey_bytes(key, false)
    }
    pub fn verify_signature(&self, did: &str, msg: &[u8], signature: &[u8]) -> Result<()> {
        if let Some(multikey) = did.strip_prefix(DID_KEY_PREFIX) {
            let (_, decoded) = multibase::decode(multikey)?;
            if decoded[..2] == self.prefix() {
                return match self {
                    Algorithm::P256 => unimplemented!(),
                    Algorithm::Secp256k1 => {
                        verify_signature::<Secp256k1>(&decoded[2..], msg, signature)
                    }
                };
            }
        }
        Err(Error::IncorrectDIDKeyPrefix(did.to_string()))
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
