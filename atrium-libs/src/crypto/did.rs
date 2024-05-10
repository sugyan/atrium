use super::error::{Error, Result};
use super::{Algorithm, DID_KEY_PREFIX};

pub fn parse_multikey(multikey: &str) -> Result<(Algorithm, Vec<u8>)> {
    let (_, decoded) = multibase::decode(multikey)?;
    if let Ok(prefix) = decoded[..2].try_into() {
        if let Some(jwt_arg) = Algorithm::from_prefix(prefix) {
            return Ok((jwt_arg, decoded[2..].to_vec()));
        }
    }
    Err(Error::UnsupportedMultikeyType)
}

pub fn format_did_key_str(alg: Algorithm, s: &str) -> Result<String> {
    let (_, key) = multibase::decode(s)?;
    format_did_key(alg, &key)
}

pub fn format_did_key(alg: Algorithm, key: &[u8]) -> Result<String> {
    Ok(DID_KEY_PREFIX.to_string() + &alg.format_multikey(key)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ecdsa::SigningKey;
    use k256::Secp256k1;

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

    #[test]
    fn secp256k1() {
        for (seed, id) in secp256k1_vectors() {
            let bytes = hex::decode(seed).expect("hex decoding should succeed");
            let sign = SigningKey::<Secp256k1>::from_slice(&bytes)
                .expect("initializing signing key should succeed");
            let result =
                format_did_key(Algorithm::Secp256k1, &sign.verifying_key().to_sec1_bytes())
                    .expect("formatting DID key should succeed");
            assert_eq!(result, id);
        }
    }
}
