use super::get_http_client;
use crate::store::memory::MemorySimpleStore;
use crate::store::SimpleStore;
use crate::utils::get_random_values;
use atrium_xrpc::http::{Request, Response};
use atrium_xrpc::HttpClient;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use ecdsa::hazmat::{DigestPrimitive, SignPrimitive};
use ecdsa::{signature::SignerMut, Signature, SigningKey};
use ecdsa::{PrimeCurve, SignatureSize};
use elliptic_curve::generic_array::ArrayLength;
use elliptic_curve::ops::Invert;
use elliptic_curve::sec1::{FromEncodedPoint, ModulusSize, ToEncodedPoint};
use elliptic_curve::subtle::CtOption;
use elliptic_curve::{
    AffinePoint, Curve, CurveArithmetic, FieldBytesSize, JwkEcKey, JwkParameters, Scalar, SecretKey,
};
use rand::rngs::ThreadRng;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]

pub enum Error {
    #[error("unsupported curve: {0}")]
    UnsupportedCurve(String),
    #[error("key does not match any alg supported by the server")]
    UnsupportedKey,
    #[error(transparent)]
    EC(#[from] elliptic_curve::Error),
    #[error(transparent)]
    SerdeJson(#[from] serde_json::Error),
    #[error(transparent)]
    SystemTime(#[from] std::time::SystemTimeError),
}

type Result<T> = core::result::Result<T, Error>;

#[derive(Serialize)]
enum JwtHeaderType {
    #[serde(rename = "dpop+jwt")]
    DpopJwt,
}

#[derive(Serialize)]
struct JwtHeader {
    alg: String,
    typ: JwtHeaderType,
    jwk: JwkEcKey,
}

#[derive(Serialize)]
struct JwtClaims {
    iss: String,
    iat: u64,
    jti: String,
    htm: String,
    htu: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    nonce: Option<String>,
}

pub struct DpopClient<S = MemorySimpleStore<String, String>>
where
    S: SimpleStore<String, String>,
{
    key: JwkEcKey,
    iss: String,
    nonces: S,
}

impl DpopClient {
    pub fn new(key: JwkEcKey, iss: String, supported_algs: Option<Vec<String>>) -> Result<Self> {
        if let Some(algs) = supported_algs {
            let alg = String::from(match key.crv() {
                k256::Secp256k1::CRV => "ES256K",
                p256::NistP256::CRV => "ES256",
                _ => return Err(Error::UnsupportedCurve(key.crv().to_string())),
            });
            if !algs.contains(&alg) {
                return Err(Error::UnsupportedKey);
            }
        }
        let nonces = MemorySimpleStore::<String, String>::default();
        Ok(Self { key, iss, nonces })
    }
    fn build_proof(&self, htm: String, htu: String, nonce: Option<String>) -> Result<String> {
        Ok(match self.key.crv() {
            k256::Secp256k1::CRV => {
                self.create_jwk::<k256::Secp256k1>(htm, htu, String::from("ES256K"), nonce)?
            }
            p256::NistP256::CRV => {
                self.create_jwk::<p256::NistP256>(htm, htu, String::from("ES256K"), nonce)?
            }
            _ => return Err(Error::UnsupportedCurve(self.key.crv().to_string())),
        })
    }
    fn create_jwk<C>(
        &self,
        htm: String,
        htu: String,
        alg: String,
        nonce: Option<String>,
    ) -> Result<String>
    where
        C: Curve + JwkParameters + PrimeCurve + CurveArithmetic + DigestPrimitive,
        AffinePoint<C>: FromEncodedPoint<C> + ToEncodedPoint<C>,
        FieldBytesSize<C>: ModulusSize,
        Scalar<C>: Invert<Output = CtOption<Scalar<C>>> + SignPrimitive<C>,
        SignatureSize<C>: ArrayLength<u8>,
    {
        let key = SecretKey::<C>::from_jwk(&self.key)?;
        let iat = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs();
        let header = JwtHeader {
            alg,
            typ: JwtHeaderType::DpopJwt,
            jwk: key.public_key().to_jwk(),
        };
        let payload = JwtClaims {
            iss: self.iss.clone(),
            iat,
            jti: URL_SAFE_NO_PAD.encode(get_random_values(&mut ThreadRng::default())),
            htm,
            htu,
            nonce,
        };
        let header = URL_SAFE_NO_PAD.encode(serde_json::to_vec(&header)?);
        let payload = URL_SAFE_NO_PAD.encode(serde_json::to_vec(&payload)?);
        let mut signing_key = SigningKey::<C>::from(key);
        let signature: Signature<_> = signing_key.sign(format!("{header}.{payload}").as_bytes());
        Ok(format!(
            "{header}.{payload}.{}",
            URL_SAFE_NO_PAD.encode(signature.to_bytes())
        ))
    }
    fn is_use_dpop_nonce_error(&self, response: &Response<Vec<u8>>) -> bool {
        // is auth server?
        if response.status() == 400 {
            #[derive(Deserialize)]
            struct ErrorResponse {
                error: String,
            }
            if let Ok(res) = serde_json::from_slice::<ErrorResponse>(response.body()) {
                return res.error == "use_dpop_nonce";
            };
        }
        // is resource server?

        false
    }
}

#[cfg_attr(target_arch = "wasm32", async_trait::async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait::async_trait)]
impl HttpClient for DpopClient {
    async fn send_http(
        &self,
        mut request: Request<Vec<u8>>,
    ) -> core::result::Result<Response<Vec<u8>>, Box<dyn std::error::Error + Send + Sync + 'static>>
    {
        let uri = request.uri();
        let nonce_key = uri.authority().unwrap().to_string();
        let htm = request.method().to_string();
        let htu = uri.to_string();

        let init_nonce = self.nonces.get(&nonce_key).await?;
        let init_proof = self.build_proof(htm.clone(), htu.clone(), init_nonce.clone())?;
        println!("init proof: {init_proof}");
        request.headers_mut().insert("DPoP", init_proof.parse()?);
        let response = get_http_client().send_http(request.clone()).await?;

        let next_nonce = response
            .headers()
            .get("DPoP-Nonce")
            .and_then(|v| v.to_str().ok())
            .map(String::from);
        match &next_nonce {
            Some(s) if next_nonce != init_nonce => {
                // Store the fresh nonce for future requests
                self.nonces.set(nonce_key, s.clone()).await?;
            }
            _ => {
                // No nonce was returned or it is the same as the one we sent. No need to
                // update the nonce store, or retry the request.
                return Ok(response);
            }
        }

        if !self.is_use_dpop_nonce_error(&response) {
            return Ok(response);
        }
        let next_proof = self.build_proof(htm, htu, next_nonce)?;
        println!("next proof: {next_proof}");
        request.headers_mut().insert("DPoP", next_proof.parse()?);
        let response = get_http_client().send_http(request).await?;
        Ok(response)
    }
}
