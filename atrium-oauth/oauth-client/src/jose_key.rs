use elliptic_curve::{JwkEcKey, SecretKey};
use rand::rngs::ThreadRng;

pub fn generate(allowed_algos: &[String]) -> Option<JwkEcKey> {
    for alg in allowed_algos {
        match alg.as_str() {
            "ES256K" => {
                return Some(JwkEcKey::from(&SecretKey::<k256::Secp256k1>::random(
                    &mut ThreadRng::default(),
                )));
            }
            "ES256" => {
                return Some(JwkEcKey::from(&SecretKey::<p256::NistP256>::random(
                    &mut ThreadRng::default(),
                )));
            }
            _ => {}
        }
    }
    None
}
