use crate::error::Result;
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
        Ok(self.format_mulikey_compressed(&self.compress_pubkey(key)?))
    }
    pub(crate) fn format_mulikey_compressed(&self, key: &[u8]) -> String {
        let mut v = Vec::with_capacity(2 + key.len());
        v.extend_from_slice(&self.prefix());
        v.extend_from_slice(key);
        multibase::encode(Base::Base58Btc, v)
    }
    pub(crate) fn compress_pubkey(&self, key: &[u8]) -> Result<Vec<u8>> {
        self.pubkey_bytes(key, true)
    }
    pub(crate) fn decompress_pubkey(&self, key: &[u8]) -> Result<Vec<u8>> {
        self.pubkey_bytes(key, false)
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

#[cfg(test)]
mod tests {
    use super::Algorithm;
    use crate::did::parse_did_key;
    use crate::keypair::{Did, P256Keypair, Secp256k1Keypair};
    use rand::rngs::ThreadRng;

    #[test]
    fn p256_compress_decompress() {
        let did = P256Keypair::create(&mut ThreadRng::default()).did();
        let (alg, key) = parse_did_key(&did).expect("parsing did key should succeed");
        assert_eq!(alg, Algorithm::P256);
        // compress a key to the correct length
        let compressed = alg
            .pubkey_bytes(&key, true)
            .expect("compressing public key should succeed");
        assert_eq!(compressed.len(), 33);
        // decompress a key to the original
        let decompressed = alg
            .pubkey_bytes(&compressed, false)
            .expect("decompressing public key should succeed");
        assert_eq!(decompressed.len(), 65);
        assert_eq!(key, decompressed);

        // works consitesntly
        let keys = (0..100)
            .map(|_| {
                let did = P256Keypair::create(&mut ThreadRng::default()).did();
                let (_, key) = parse_did_key(&did).expect("parsing did key should succeed");
                key
            })
            .collect::<Vec<_>>();
        let compressed = keys
            .iter()
            .filter_map(|key| alg.pubkey_bytes(key, true).ok())
            .collect::<Vec<_>>();
        let decompressed = compressed
            .iter()
            .filter_map(|key| alg.pubkey_bytes(key, false).ok())
            .collect::<Vec<_>>();
        assert_eq!(keys, decompressed);
    }

    #[test]
    fn secp256k1_compress_decompress() {
        let did = Secp256k1Keypair::create(&mut ThreadRng::default()).did();
        let (alg, key) = parse_did_key(&did).expect("parsing did key should succeed");
        assert_eq!(alg, Algorithm::Secp256k1);
        // compress a key to the correct length
        let compressed = alg
            .pubkey_bytes(&key, true)
            .expect("compressing public key should succeed");
        assert_eq!(compressed.len(), 33);
        // decompress a key to the original
        let decompressed = alg
            .pubkey_bytes(&compressed, false)
            .expect("decompressing public key should succeed");
        assert_eq!(decompressed.len(), 65);
        assert_eq!(key, decompressed);

        // works consitesntly
        let keys = (0..100)
            .map(|_| {
                let did = Secp256k1Keypair::create(&mut ThreadRng::default()).did();
                let (_, key) = parse_did_key(&did).expect("parsing did key should succeed");
                key
            })
            .collect::<Vec<_>>();
        let compressed = keys
            .iter()
            .filter_map(|key| alg.pubkey_bytes(key, true).ok())
            .collect::<Vec<_>>();
        let decompressed = compressed
            .iter()
            .filter_map(|key| alg.pubkey_bytes(key, false).ok())
            .collect::<Vec<_>>();
        assert_eq!(keys, decompressed);
    }
}
