use crate::jose::{
    create_signed_jwt,
    jws::RegisteredHeader,
    jwt::{Claims, PublicClaims, RegisteredClaims},
};
use atrium_common::store::{memory::MemoryStore, Store};
use atrium_xrpc::{
    http::{Request, Response},
    HttpClient,
};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use chrono::Utc;
use jose_jwa::{Algorithm, Signing};
use jose_jwk::{crypto, EcCurves, Jwk, Key};
use rand::{
    rngs::SmallRng,
    {RngCore, SeedableRng},
};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::sync::Arc;
use thiserror::Error;

const JWT_HEADER_TYP_DPOP: &str = "dpop+jwt";

#[derive(Deserialize)]
struct ErrorResponse {
    error: String,
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("crypto error: {0:?}")]
    JwkCrypto(crypto::Error),
    #[error("key does not match any alg supported by the server")]
    UnsupportedKey,
    #[error(transparent)]
    SerdeJson(#[from] serde_json::Error),
}

type Result<T> = core::result::Result<T, Error>;

pub struct DpopClient<T, S = MemoryStore<String, String>>
where
    S: Store<String, String>,
{
    inner: Arc<T>,
    pub(crate) key: Key,
    nonces: S,
    is_auth_server: bool,
}

impl<T> DpopClient<T> {
    pub fn new(
        key: Key,
        http_client: Arc<T>,
        is_auth_server: bool,
        supported_algs: &Option<Vec<String>>,
    ) -> Result<Self> {
        if let Some(algs) = supported_algs {
            let alg = String::from(match &key {
                Key::Ec(ec) => match &ec.crv {
                    EcCurves::P256 => "ES256",
                    _ => unimplemented!(),
                },
                _ => unimplemented!(),
            });
            if !algs.contains(&alg) {
                return Err(Error::UnsupportedKey);
            }
        }
        let nonces = MemoryStore::<String, String>::default();
        Ok(Self { inner: http_client, key, nonces, is_auth_server })
    }
}

impl<T, S> DpopClient<T, S>
where
    S: Store<String, String>,
{
    fn build_proof(
        &self,
        htm: String,
        htu: String,
        ath: Option<String>,
        nonce: Option<String>,
    ) -> Result<String> {
        match crypto::Key::try_from(&self.key).map_err(Error::JwkCrypto)? {
            crypto::Key::P256(crypto::Kind::Secret(secret_key)) => {
                let mut header = RegisteredHeader::from(Algorithm::Signing(Signing::Es256));
                header.typ = Some(JWT_HEADER_TYP_DPOP.into());
                header.jwk = Some(Jwk {
                    key: Key::from(&crypto::Key::from(secret_key.public_key())),
                    prm: Default::default(),
                });
                let claims = Claims {
                    registered: RegisteredClaims {
                        jti: Some(Self::generate_jti()),
                        iat: Some(Utc::now().timestamp()),
                        ..Default::default()
                    },
                    public: PublicClaims { htm: Some(htm), htu: Some(htu), ath, nonce },
                };
                Ok(create_signed_jwt(secret_key.into(), header.into(), claims)?)
            }
            _ => unimplemented!(),
        }
    }
    fn is_use_dpop_nonce_error(&self, response: &Response<Vec<u8>>) -> bool {
        // https://datatracker.ietf.org/doc/html/rfc9449#name-authorization-server-provid
        if self.is_auth_server {
            if response.status() == 400 {
                if let Ok(res) = serde_json::from_slice::<ErrorResponse>(response.body()) {
                    return res.error == "use_dpop_nonce";
                };
            }
        }
        // https://datatracker.ietf.org/doc/html/rfc6750#section-3
        // https://datatracker.ietf.org/doc/html/rfc9449#name-resource-server-provided-no
        else if response.status() == 401 {
            if let Some(www_auth) =
                response.headers().get("WWW-Authenticate").and_then(|v| v.to_str().ok())
            {
                return www_auth.starts_with("DPoP")
                    && www_auth.contains(r#"error="use_dpop_nonce""#);
            }
        }
        false
    }
    // https://datatracker.ietf.org/doc/html/rfc9449#section-4.2
    fn generate_jti() -> String {
        let mut rng = SmallRng::from_entropy();
        let mut bytes = [0u8; 12];
        rng.fill_bytes(&mut bytes);
        URL_SAFE_NO_PAD.encode(bytes)
    }
}

impl<T, S> HttpClient for DpopClient<T, S>
where
    T: HttpClient + Send + Sync + 'static,
    S: Store<String, String> + Send + Sync + 'static,
    S::Error: std::error::Error + Send + Sync + 'static,
{
    async fn send_http(
        &self,
        mut request: Request<Vec<u8>>,
    ) -> core::result::Result<Response<Vec<u8>>, Box<dyn std::error::Error + Send + Sync + 'static>>
    {
        let uri = request.uri();
        let nonce_key = uri.authority().unwrap().to_string();
        let htm = request.method().to_string();
        let htu = uri.to_string();
        // https://datatracker.ietf.org/doc/html/rfc9449#section-4.2
        let ath = request
            .headers()
            .get("Authorization")
            .filter(|v| v.to_str().map_or(false, |s| s.starts_with("DPoP ")))
            .map(|auth| URL_SAFE_NO_PAD.encode(Sha256::digest(&auth.as_bytes()[5..])));

        let init_nonce = self.nonces.get(&nonce_key).await?;
        let init_proof =
            self.build_proof(htm.clone(), htu.clone(), ath.clone(), init_nonce.clone())?;
        request.headers_mut().insert("DPoP", init_proof.parse()?);
        let response = self.inner.send_http(request.clone()).await?;

        let next_nonce =
            response.headers().get("DPoP-Nonce").and_then(|v| v.to_str().ok()).map(String::from);
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
        let next_proof = self.build_proof(htm, htu, ath, next_nonce)?;
        request.headers_mut().insert("DPoP", next_proof.parse()?);
        let response = self.inner.send_http(request).await?;
        Ok(response)
    }
}

impl<T> Clone for DpopClient<T> {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
            key: self.key.clone(),
            nonces: self.nonces.clone(),
            is_auth_server: self.is_auth_server,
        }
    }
}
