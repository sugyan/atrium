use crate::did::prefix_did_key;
use crate::error::Result;
use crate::Algorithm;
use ecdsa::elliptic_curve::{
    generic_array::ArrayLength,
    ops::Invert,
    sec1::{FromEncodedPoint, ModulusSize, ToEncodedPoint},
    subtle::CtOption,
    AffinePoint, CurveArithmetic, FieldBytesSize, PrimeCurve, Scalar,
};
use ecdsa::hazmat::{DigestPrimitive, SignPrimitive};
use ecdsa::signature::{rand_core::CryptoRngCore, Signer};
use ecdsa::{Signature, SignatureSize, SigningKey};
use k256::Secp256k1;
use p256::NistP256;

pub struct Keypair<C>
where
    C: PrimeCurve + CurveArithmetic,
    Scalar<C>: Invert<Output = CtOption<Scalar<C>>> + SignPrimitive<C>,
    SignatureSize<C>: ArrayLength<u8>,
{
    signing_key: SigningKey<C>,
}

impl<C> Keypair<C>
where
    C: PrimeCurve + CurveArithmetic,
    Scalar<C>: Invert<Output = CtOption<Scalar<C>>> + SignPrimitive<C>,
    SignatureSize<C>: ArrayLength<u8>,
{
    pub fn create(rng: &mut impl CryptoRngCore) -> Self {
        Self {
            signing_key: SigningKey::<C>::random(rng),
        }
    }
    pub fn import(priv_key: &[u8]) -> Result<Self> {
        Ok(Self {
            signing_key: SigningKey::from_slice(priv_key)?,
        })
    }
}

impl<C> Keypair<C>
where
    C: PrimeCurve + CurveArithmetic,
    Scalar<C>: Invert<Output = CtOption<Scalar<C>>> + SignPrimitive<C>,
    SignatureSize<C>: ArrayLength<u8>,
    AffinePoint<C>: FromEncodedPoint<C> + ToEncodedPoint<C>,
    FieldBytesSize<C>: ModulusSize,
{
    fn compressed_public_key(&self) -> Box<[u8]> {
        self.signing_key
            .verifying_key()
            .to_encoded_point(true)
            .to_bytes()
    }
}

impl<C> Keypair<C>
where
    C: PrimeCurve + CurveArithmetic + DigestPrimitive,
    Scalar<C>: Invert<Output = CtOption<Scalar<C>>> + SignPrimitive<C>,
    SignatureSize<C>: ArrayLength<u8>,
{
    pub fn sign(&self, msg: &[u8]) -> Result<Vec<u8>> {
        let signature: Signature<_> = self.signing_key.try_sign(msg)?;
        Ok(signature
            .normalize_s()
            .unwrap_or(signature)
            .to_bytes()
            .to_vec())
    }
}

pub trait Did<C> {
    fn did(&self) -> String;
}

pub trait Export<C> {
    fn export(&self) -> Vec<u8>;
}

impl<C> Export<C> for Keypair<C>
where
    C: PrimeCurve + CurveArithmetic,
    Scalar<C>: Invert<Output = CtOption<Scalar<C>>> + SignPrimitive<C>,
    SignatureSize<C>: ArrayLength<u8>,
{
    fn export(&self) -> Vec<u8> {
        self.signing_key.to_bytes().to_vec()
    }
}

pub type P256Keypair = Keypair<NistP256>;

impl Did<NistP256> for P256Keypair {
    fn did(&self) -> String {
        prefix_did_key(&Algorithm::P256.format_mulikey_compressed(&self.compressed_public_key()))
    }
}

pub type Secp256k1Keypair = Keypair<Secp256k1>;

impl Did<Secp256k1> for Secp256k1Keypair {
    fn did(&self) -> String {
        prefix_did_key(
            &Algorithm::Secp256k1.format_mulikey_compressed(&self.compressed_public_key()),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::{P256Keypair, Secp256k1Keypair};
    use crate::did::{format_did_key, parse_did_key};
    use crate::verify::Verifier;
    use crate::Algorithm;
    use rand::rngs::ThreadRng;

    #[test]
    fn p256_did() {
        let keypair = P256Keypair::create(&mut ThreadRng::default());
        let did = {
            use super::Did;
            keypair.did()
        };
        let formatted = format_did_key(
            Algorithm::P256,
            &keypair.signing_key.verifying_key().to_sec1_bytes(),
        )
        .expect("formatting to did key should succeed");
        assert_eq!(did, formatted);

        let (alg, public_key) = parse_did_key(&did).expect("parsing did key should succeed");
        assert_eq!(alg, Algorithm::P256);
        assert_eq!(
            public_key,
            keypair
                .signing_key
                .verifying_key()
                .to_encoded_point(false)
                .as_bytes()
        );
    }

    #[test]
    fn secp256k1_did() {
        let keypair = Secp256k1Keypair::create(&mut ThreadRng::default());
        let did = {
            use super::Did;
            keypair.did()
        };
        let formatted = format_did_key(
            Algorithm::Secp256k1,
            &keypair.signing_key.verifying_key().to_sec1_bytes(),
        )
        .expect("formatting to did key should succeed");
        assert_eq!(did, formatted);

        let (alg, public_key) = parse_did_key(&did).expect("parsing did key should succeed");
        assert_eq!(alg, Algorithm::Secp256k1);
        assert_eq!(
            public_key,
            keypair
                .signing_key
                .verifying_key()
                .to_encoded_point(false)
                .as_bytes()
        );
    }

    #[test]
    fn p256_export() {
        let keypair = P256Keypair::create(&mut ThreadRng::default());
        let exported = {
            use super::Export;
            keypair.export()
        };
        let imported = P256Keypair::import(&exported).expect("importing keypair should succeed");
        {
            use super::Did;
            assert_eq!(keypair.did(), imported.did());
        }
    }

    #[test]
    fn secp256k1_export() {
        let keypair = Secp256k1Keypair::create(&mut ThreadRng::default());
        let exported = {
            use super::Export;
            keypair.export()
        };
        let imported =
            Secp256k1Keypair::import(&exported).expect("importing keypair should succeed");
        {
            use super::Did;
            assert_eq!(keypair.did(), imported.did());
        }
    }

    #[test]
    fn p256_verify() {
        let keypair = P256Keypair::create(&mut ThreadRng::default());
        let did = {
            use super::Did;
            keypair.did()
        };
        let (alg, public_key) = parse_did_key(&did).expect("parsing did key should succeed");
        assert_eq!(alg, Algorithm::P256);

        let verifier = Verifier::default();
        let msg = [1, 2, 3, 4, 5, 6, 7, 8];
        let signature = keypair.sign(&msg).expect("signing should succeed");
        let mut corrupted_signature = signature.clone();
        corrupted_signature[0] = corrupted_signature[0].wrapping_add(1);
        assert!(
            verifier.verify(alg, &public_key, &msg, &signature).is_ok(),
            "verifying signature should succeed"
        );
        assert!(
            verifier
                .verify(alg, &public_key, &msg[..7], &signature)
                .is_err(),
            "verifying signature should fail with incorrect message"
        );
        assert!(
            verifier
                .verify(alg, &public_key, &msg, &corrupted_signature)
                .is_err(),
            "verifying signature should fail with incorrect signature"
        );
        assert!(
            verifier
                .verify(Algorithm::Secp256k1, &public_key, &msg, &signature)
                .is_err(),
            "verifying signature should fail with incorrect algorithm"
        );
    }

    #[test]
    fn secp256k1_verify() {
        let keypair = Secp256k1Keypair::create(&mut ThreadRng::default());
        let did = {
            use super::Did;
            keypair.did()
        };
        let (alg, public_key) = parse_did_key(&did).expect("parsing did key should succeed");
        assert_eq!(alg, Algorithm::Secp256k1);

        let verifier = Verifier::default();
        let msg = [1, 2, 3, 4, 5, 6, 7, 8];
        let signature = keypair.sign(&msg).expect("signing should succeed");
        let mut corrupted_signature = signature.clone();
        corrupted_signature[0] = corrupted_signature[0].wrapping_add(1);
        assert!(
            verifier.verify(alg, &public_key, &msg, &signature).is_ok(),
            "verifying signature should succeed"
        );
        assert!(
            verifier
                .verify(alg, &public_key, &msg[..7], &signature)
                .is_err(),
            "verifying signature should fail with incorrect message"
        );
        assert!(
            verifier
                .verify(alg, &public_key, &msg, &corrupted_signature)
                .is_err(),
            "verifying signature should fail with incorrect signature"
        );
        assert!(
            verifier
                .verify(Algorithm::P256, &public_key, &msg, &signature)
                .is_err(),
            "verifying signature should fail with incorrect algorithm"
        );
    }
}
