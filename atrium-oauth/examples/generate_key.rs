use elliptic_curve::pkcs8::EncodePrivateKey;
use elliptic_curve::SecretKey;
use jose_jwa::{Algorithm, Signing};
use jose_jwk::{Class, Jwk, JwkSet, Key, Parameters};
use p256::NistP256;
use rand::rngs::ThreadRng;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let secret_key = SecretKey::<NistP256>::random(&mut ThreadRng::default());
    let key = Key::from(&secret_key.public_key().into());
    let jwks = JwkSet {
        keys: vec![Jwk {
            key,
            prm: Parameters {
                alg: Some(Algorithm::Signing(Signing::Es256)),
                kid: Some(String::from("kid01")),
                cls: Some(Class::Signing),
                ..Default::default()
            },
        }],
    };
    println!("SECRET KEY:");
    println!("{}", secret_key.to_pkcs8_pem(Default::default())?.as_str());

    println!("JWKS:");
    println!("{}", serde_json::to_string_pretty(&jwks)?);
    Ok(())
}
