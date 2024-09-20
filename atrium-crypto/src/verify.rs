//! Verifies a signature for a message using a public key.
use crate::did::parse_did_key;
use crate::error::{Error, Result};
use crate::Algorithm;
use ecdsa::der::{MaxOverhead, MaxSize};
use ecdsa::elliptic_curve::{
    generic_array::ArrayLength,
    sec1::{FromEncodedPoint, ModulusSize, ToEncodedPoint},
    AffinePoint, CurveArithmetic, FieldBytesSize, PrimeCurve,
};
use ecdsa::hazmat::{DigestPrimitive, VerifyPrimitive};
use ecdsa::{SignatureSize, VerifyingKey};
use k256::Secp256k1;
use p256::NistP256;
use std::ops::Add;

/// Verify a signature for a message using the given DID key formatted public key.
///
/// This function verifies a signature using [`Verifier::default()`].
///
/// # Examples
///
/// ```
/// use atrium_crypto::verify::verify_signature;
///
/// # fn main() -> atrium_crypto::Result<()> {
/// let did_key = "did:key:zQ3shtNTBUUCARYFEkRPZQ9NCaM5i5hVHPeEsEKXpmVkR2Upq";
/// let signature = hex::decode(
///     "fdaa28ab03d6767c11d71fa39627c770ff62f91ca9661401ca0e2c475ae96a8c27064fbde3c355fa8121d2e8bbcf87a2de308e1d72b9bf4270f1e7cd8a1575ab"
/// ).unwrap();
/// assert!(verify_signature(did_key, b"Hello, world!", &signature).is_ok());
/// assert!(verify_signature(did_key, b"Hello, world?", &signature).is_err());
/// # Ok(())
/// # }
/// ```
pub fn verify_signature(did_key: &str, msg: &[u8], signature: &[u8]) -> Result<()> {
    let (alg, public_key) = parse_did_key(did_key)?;
    Verifier::default().verify(alg, &public_key, msg, signature)
}

/// Verifier for verifying signatures for a message using a public key.
///
/// This verifier can be configured to `allow_malleable` mode, which allows
/// verifying signatures with "high-S" or DER-encoded ones.
/// By default, this verifier allows only "low-S" signatures.
///
/// See also: [https://github.com/bluesky-social/atproto/pull/1839](https://github.com/bluesky-social/atproto/pull/1839)
#[derive(Debug, Default)]
pub struct Verifier {
    allow_malleable: bool,
}

impl Verifier {
    /// Create a new verifier with the given malleable mode.
    pub fn new(allow_malleable: bool) -> Self {
        Self { allow_malleable }
    }
    /// Verify a signature for a message using the given public key.
    /// The `algorithm` is used to determine the curve for the public key.
    pub fn verify(
        &self,
        algorithm: Algorithm,
        public_key: &[u8],
        msg: &[u8],
        signature: &[u8],
    ) -> Result<()> {
        match algorithm {
            Algorithm::P256 => self.verify_inner::<NistP256>(public_key, msg, signature),
            Algorithm::Secp256k1 => self.verify_inner::<Secp256k1>(public_key, msg, signature),
        }
    }
    /// Verify a signature for a message using the given public key.
    /// Any elliptic curve of the generics implementation of [`ECDSA`](ecdsa) can be used for parameter `C`.
    pub fn verify_inner<C>(&self, public_key: &[u8], msg: &[u8], bytes: &[u8]) -> Result<()>
    where
        C: PrimeCurve + CurveArithmetic + DigestPrimitive,
        AffinePoint<C>: VerifyPrimitive<C> + FromEncodedPoint<C> + ToEncodedPoint<C>,
        FieldBytesSize<C>: ModulusSize,
        SignatureSize<C>: ArrayLength<u8>,
        MaxSize<C>: ArrayLength<u8>,
        <FieldBytesSize<C> as Add>::Output: Add<MaxOverhead> + ArrayLength<u8>,
    {
        let verifying_key = VerifyingKey::<C>::from_sec1_bytes(public_key)?;
        if let Ok(mut signature) = ecdsa::Signature::from_slice(bytes) {
            if let Some(normalized) = signature.normalize_s() {
                if !self.allow_malleable {
                    return Err(Error::LowSSignatureNotAllowed);
                }
                signature = normalized
            }
            Ok(ecdsa::signature::Verifier::verify(&verifying_key, msg, &signature)?)
        }
        // signature may be DER-encoded. If `allow_malleable` is true, try to parse and use it.
        else if self.allow_malleable {
            let signature = ecdsa::der::Signature::from_bytes(bytes)?;
            Ok(ecdsa::signature::Verifier::verify(&verifying_key, msg, &signature)?)
        } else {
            Err(Error::InvalidSignature)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use multibase::Base;
    use serde::{Deserialize, Serialize};
    use std::{fs::File, path::PathBuf};

    #[derive(Debug, Serialize, Deserialize)]
    enum Algorithm {
        ES256,
        ES256K,
    }

    #[derive(Debug, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct TestVector {
        comment: String,
        message_base64: String,
        algorithm: Algorithm,
        public_key_multibase: String,
        public_key_did: String,
        signature_base64: String,
        valid_signature: bool,
        tags: Vec<String>,
    }

    fn test_vectors(cond: Option<&str>) -> Vec<TestVector> {
        let data_path =
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/data/signature-fixtures.json");
        let file = File::open(data_path).expect("opening test data should succeed");
        let v = serde_json::from_reader::<_, Vec<TestVector>>(file)
            .expect("parsing test data should succeed");
        v.into_iter()
            .filter(|v| if let Some(s) = cond { v.tags.contains(&s.to_string()) } else { true })
            .collect()
    }

    #[test]
    fn verify() {
        let vectors = test_vectors(None);
        assert!(!vectors.is_empty());
        let verifier = Verifier::default();
        for vector in vectors {
            let message = Base::Base64
                .decode(vector.message_base64)
                .expect("decoding message should succeed");
            let signature = Base::Base64
                .decode(vector.signature_base64)
                .expect("decoding signature should succeed");

            let (base, decoded_key) = multibase::decode(vector.public_key_multibase)
                .expect("decoding multibase public key should succeed");
            assert_eq!(base, Base::Base58Btc);
            let (alg, parsed_key) =
                parse_did_key(&vector.public_key_did).expect("parsing DID key should succeed");

            // assert_eq!(decoded_key, parsed_key);
            match vector.algorithm {
                Algorithm::ES256 => assert_eq!(alg, crate::Algorithm::P256),
                Algorithm::ES256K => assert_eq!(alg, crate::Algorithm::Secp256k1),
            }
            assert_eq!(
                verifier.verify(alg, &decoded_key, &message, &signature).is_ok(),
                vector.valid_signature
            );
            assert_eq!(
                verifier.verify(alg, &parsed_key, &message, &signature).is_ok(),
                vector.valid_signature
            );
        }
    }

    #[test]
    fn verify_high_s() {
        let vectors = test_vectors(Some("high-s"));
        assert!(vectors.len() >= 2);
        let verifier = Verifier::new(true);
        for vector in vectors {
            let message = Base::Base64
                .decode(vector.message_base64)
                .expect("decoding message should succeed");
            let signature = Base::Base64
                .decode(vector.signature_base64)
                .expect("decoding signature should succeed");

            let (base, decoded_key) = multibase::decode(vector.public_key_multibase)
                .expect("decoding multibase public key should succeed");
            assert_eq!(base, Base::Base58Btc);
            let (alg, parsed_key) =
                parse_did_key(&vector.public_key_did).expect("parsing DID key should succeed");

            // assert_eq!(decoded_key, parsed_key);
            match vector.algorithm {
                Algorithm::ES256 => assert_eq!(alg, crate::Algorithm::P256),
                Algorithm::ES256K => assert_eq!(alg, crate::Algorithm::Secp256k1),
            }
            assert!(!vector.valid_signature);
            assert!(verifier.verify(alg, &decoded_key, &message, &signature).is_ok());
            assert!(verifier.verify(alg, &parsed_key, &message, &signature).is_ok());
        }
    }

    #[test]
    fn verify_der_encoded() {
        let vectors = test_vectors(Some("der-encoded"));
        assert!(vectors.len() >= 2);
        let verifier = Verifier::new(true);
        for vector in vectors {
            let message = Base::Base64
                .decode(vector.message_base64)
                .expect("decoding message should succeed");
            let signature = Base::Base64
                .decode(vector.signature_base64)
                .expect("decoding signature should succeed");

            let (base, decoded_key) = multibase::decode(vector.public_key_multibase)
                .expect("decoding multibase public key should succeed");
            assert_eq!(base, Base::Base58Btc);
            let (alg, parsed_key) =
                parse_did_key(&vector.public_key_did).expect("parsing DID key should succeed");

            // assert_eq!(decoded_key, parsed_key);
            match vector.algorithm {
                Algorithm::ES256 => assert_eq!(alg, crate::Algorithm::P256),
                Algorithm::ES256K => assert_eq!(alg, crate::Algorithm::Secp256k1),
            }
            assert!(!vector.valid_signature);
            assert!(verifier.verify(alg, &decoded_key, &message, &signature).is_ok());
            assert!(verifier.verify(alg, &parsed_key, &message, &signature).is_ok());
        }
    }
}
