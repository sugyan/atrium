use crate::{error::Result, Algorithm};
use ecdsa::VerifyingKey;
use k256::Secp256k1;
use p256::NistP256;

pub(crate) fn compress_pubkey(alg: Algorithm, key: &[u8]) -> Result<Vec<u8>> {
    pubkey_bytes(alg, key, true)
}

pub(crate) fn decompress_pubkey(alg: Algorithm, key: &[u8]) -> Result<Vec<u8>> {
    pubkey_bytes(alg, key, false)
}

fn pubkey_bytes(alg: Algorithm, key: &[u8], compress: bool) -> Result<Vec<u8>> {
    Ok(match alg {
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

#[cfg(test)]
mod tests {
    use super::{compress_pubkey, decompress_pubkey};
    use crate::did::parse_did_key;
    use crate::keypair::{Did, P256Keypair, Secp256k1Keypair};
    use crate::Algorithm;
    use rand::rngs::ThreadRng;

    #[test]
    fn p256_compress_decompress() {
        let did = P256Keypair::create(&mut ThreadRng::default()).did();
        let (alg, key) = parse_did_key(&did).expect("parsing did key should succeed");
        assert_eq!(alg, Algorithm::P256);
        // compress a key to the correct length
        let compressed = compress_pubkey(alg, &key).expect("compressing public key should succeed");
        assert_eq!(compressed.len(), 33);
        // decompress a key to the original
        let decompressed =
            decompress_pubkey(alg, &compressed).expect("decompressing public key should succeed");
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
        let compressed =
            keys.iter().filter_map(|key| compress_pubkey(alg, &key).ok()).collect::<Vec<_>>();
        let decompressed = compressed
            .iter()
            .filter_map(|key| decompress_pubkey(alg, &key).ok())
            .collect::<Vec<_>>();
        assert_eq!(keys, decompressed);
    }

    #[test]
    fn secp256k1_compress_decompress() {
        let did = Secp256k1Keypair::create(&mut ThreadRng::default()).did();
        let (alg, key) = parse_did_key(&did).expect("parsing did key should succeed");
        assert_eq!(alg, Algorithm::Secp256k1);
        // compress a key to the correct length
        let compressed = compress_pubkey(alg, &key).expect("compressing public key should succeed");
        assert_eq!(compressed.len(), 33);
        // decompress a key to the original
        let decompressed =
            decompress_pubkey(alg, &compressed).expect("decompressing public key should succeed");
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
        let compressed =
            keys.iter().filter_map(|key| compress_pubkey(alg, key).ok()).collect::<Vec<_>>();
        let decompressed = compressed
            .iter()
            .filter_map(|key| decompress_pubkey(alg, key).ok())
            .collect::<Vec<_>>();
        assert_eq!(keys, decompressed);
    }
}
