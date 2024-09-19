//! Functions for parsing and formatting DID keys.
use crate::encoding::{compress_pubkey, decompress_pubkey};
use crate::error::{Error, Result};
use crate::{Algorithm, DID_KEY_PREFIX};

/// Format a public key as a DID key string.
///
/// The public key will be compressed and encoded with multibase and multicode.
/// The resulting string will start with `did:key:`.
///
/// Details:
/// [https://atproto.com/specs/cryptography#public-key-encoding](https://atproto.com/specs/cryptography#public-key-encoding)
///
/// # Examples
///
/// ```
/// use atrium_crypto::Algorithm;
/// use atrium_crypto::did::format_did_key;
///
/// # fn main() -> atrium_crypto::Result<()> {
/// let signing_key = ecdsa::SigningKey::<k256::Secp256k1>::from_slice(
///     &hex::decode("9085d2bef69286a6cbb51623c8fa258629945cd55ca705cc4e66700396894e0c").unwrap()
/// )?;
/// let public_key = signing_key.verifying_key();
/// let did_key = format_did_key(Algorithm::Secp256k1, &public_key.to_sec1_bytes())?;
/// assert_eq!(did_key, "did:key:zQ3shokFTS3brHcDQrn82RUDfCZESWL1ZdCEJwekUDPQiYBme");
/// # Ok(())
/// # }
/// ```
pub fn format_did_key(alg: Algorithm, key: &[u8]) -> Result<String> {
    Ok(prefix_did_key(&alg.format_mulikey_compressed(&compress_pubkey(alg, key)?)))
}

/// Parse a DID key string.
///
/// Input should be a string starting with `did:key:`.
/// The rest of the string is the multibase and multicode encoded public key,
/// which will be parsed with [`parse_multikey`].
///
/// Returns the parsed [`Algorithm`] and bytes of the public key.
///
/// # Examples
///
/// ```
/// use atrium_crypto::Algorithm;
/// use atrium_crypto::did::parse_did_key;
///
/// # fn main() -> atrium_crypto::Result<()> {
/// let (alg, key): (Algorithm, Vec<u8>) = parse_did_key("did:key:zQ3shokFTS3brHcDQrn82RUDfCZESWL1ZdCEJwekUDPQiYBme")?;
/// assert_eq!(alg, Algorithm::Secp256k1);
/// assert_eq!(key.len(), 65);
/// # Ok(())
/// # }
/// ```
pub fn parse_did_key(did: &str) -> Result<(Algorithm, Vec<u8>)> {
    if let Some(multikey) = did.strip_prefix(DID_KEY_PREFIX) {
        parse_multikey(multikey)
    } else {
        Err(Error::IncorrectDIDKeyPrefix(did.to_string()))
    }
}

/// Parse a multibase and multicode encoded public key string.
///
/// Details:
/// [https://atproto.com/specs/cryptography#public-key-encoding](https://atproto.com/specs/cryptography#public-key-encoding)
///
/// Returns the parsed [`Algorithm`] and bytes of the public key.
///
/// # Examples
///
/// ```
/// use atrium_crypto::Algorithm;
/// use atrium_crypto::did::parse_multikey;
///
/// # fn main() -> atrium_crypto::Result<()> {
/// let (alg, key): (Algorithm, Vec<u8>) = parse_multikey("zQ3shokFTS3brHcDQrn82RUDfCZESWL1ZdCEJwekUDPQiYBme")?;
/// assert_eq!(alg, Algorithm::Secp256k1);
/// assert_eq!(key.len(), 65);
/// # Ok(())
/// # }
/// ```
pub fn parse_multikey(multikey: &str) -> Result<(Algorithm, Vec<u8>)> {
    let (_, decoded) = multibase::decode(multikey)?;
    if let Ok(prefix) = decoded[..2].try_into() {
        if let Some(alg) = Algorithm::from_prefix(prefix) {
            return Ok((alg, decompress_pubkey(alg, &decoded[2..])?));
        }
    }
    Err(Error::UnsupportedMultikeyType)
}

pub(crate) fn prefix_did_key(multikey: &str) -> String {
    let mut ret = String::with_capacity(DID_KEY_PREFIX.len() + multikey.len());
    ret.push_str(DID_KEY_PREFIX);
    ret.push_str(multikey);
    ret
}

#[cfg(test)]
mod tests {
    use super::*;
    use ecdsa::SigningKey;
    use k256::Secp256k1;
    use multibase::Base;
    use p256::NistP256;

    // did:key secp256k1 test vectors from W3C
    // https://github.com/w3c-ccg/did-method-key/blob/main/test-vectors/secp256k1.json
    fn secp256k1_vectors() -> Vec<(&'static str, &'static str)> {
        vec![
            (
                "9085d2bef69286a6cbb51623c8fa258629945cd55ca705cc4e66700396894e0c",
                "did:key:zQ3shokFTS3brHcDQrn82RUDfCZESWL1ZdCEJwekUDPQiYBme",
            ),
            (
                "f0f4df55a2b3ff13051ea814a8f24ad00f2e469af73c363ac7e9fb999a9072ed",
                "did:key:zQ3shtxV1FrJfhqE1dvxYRcCknWNjHc3c5X1y3ZSoPDi2aur2",
            ),
            (
                "6b0b91287ae3348f8c2f2552d766f30e3604867e34adc37ccbb74a8e6b893e02",
                "did:key:zQ3shZc2QzApp2oymGvQbzP8eKheVshBHbU4ZYjeXqwSKEn6N",
            ),
            (
                "c0a6a7c560d37d7ba81ecee9543721ff48fea3e0fb827d42c1868226540fac15",
                "did:key:zQ3shadCps5JLAHcZiuX5YUtWHHL8ysBJqFLWvjZDKAWUBGzy",
            ),
            (
                "175a232d440be1e0788f25488a73d9416c04b6f924bea6354bf05dd2f1a75133",
                "did:key:zQ3shptjE6JwdkeKN4fcpnYQY3m9Cet3NiHdAfpvSUZBFoKBj",
            ),
        ]
    }

    // did:key p-256 test vectors from W3C
    // https://github.com/w3c-ccg/did-method-key/blob/main/test-vectors/nist-curves.json
    fn p256_vectors() -> Vec<(&'static str, &'static str)> {
        vec![(
            "9p4VRzdmhsnq869vQjVCTrRry7u4TtfRxhvBFJTGU2Cp",
            "did:key:zDnaeTiq1PdzvZXUaMdezchcMJQpBdH2VN4pgrrEhMCCbmwSb",
        )]
    }

    #[test]
    fn secp256k1() {
        for (seed, id) in secp256k1_vectors() {
            let bytes = hex::decode(seed).expect("hex decoding should succeed");
            let sig_key = SigningKey::<Secp256k1>::from_slice(&bytes)
                .expect("initializing signing key should succeed");
            let did_key =
                format_did_key(Algorithm::Secp256k1, &sig_key.verifying_key().to_sec1_bytes())
                    .expect("formatting DID key should succeed");
            assert_eq!(did_key, id);

            let (alg, key) = parse_did_key(&did_key).expect("parsing DID key should succeed");
            assert_eq!(alg, Algorithm::Secp256k1);
            assert_eq!(&key, sig_key.verifying_key().to_encoded_point(false).as_bytes());
        }
    }

    #[test]
    fn p256() {
        for (private_key_base58, id) in p256_vectors() {
            let bytes = Base::Base58Btc
                .decode(private_key_base58)
                .expect("multibase decoding should succeed");
            let sig_key = SigningKey::<NistP256>::from_slice(&bytes)
                .expect("initializing signing key should succeed");
            let did_key = format_did_key(Algorithm::P256, &sig_key.verifying_key().to_sec1_bytes())
                .expect("formatting DID key should succeed");
            assert_eq!(did_key, id);

            let (alg, key) = parse_did_key(&did_key).expect("parsing DID key should succeed");
            assert_eq!(alg, Algorithm::P256);
            assert_eq!(&key, sig_key.verifying_key().to_encoded_point(false).as_bytes());
        }
    }
}
