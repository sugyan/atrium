use crate::common_web::did_doc::DidDocument;
use atrium_crypto::did::{format_did_key, format_did_key_str, parse_multikey};
use atrium_crypto::Algorithm;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Could not parse signing key from doc: {0:?}")]
    SigningKey(DidDocument),
    #[error("Could not parse handle from doc: {0:?}")]
    Handle(DidDocument),
    #[error("Could not parse pds from doc: {0:?}")]
    Pds(DidDocument),
    #[error(transparent)]
    Crypto(#[from] atrium_crypto::error::Error),
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct AtprotoData {
    pub did: String,
    pub signing_key: String,
    pub handle: String,
    pub pds: String,
}

pub fn ensure_atproto_data(did_doc: &DidDocument) -> Result<AtprotoData> {
    Ok(AtprotoData {
        did: did_doc.get_did(),
        signing_key: get_key(did_doc)?.ok_or(Error::SigningKey(did_doc.clone()))?,
        handle: did_doc.get_handle().ok_or(Error::Handle(did_doc.clone()))?,
        pds: did_doc
            .get_pds_endpoint()
            .ok_or(Error::Pds(did_doc.clone()))?,
    })
}

fn get_key(did_doc: &DidDocument) -> Result<Option<String>> {
    if let Some((r#type, public_key_multibase)) = did_doc.get_signing_key() {
        get_did_key_from_multibase(r#type, public_key_multibase)
    } else {
        Ok(None)
    }
}

fn get_did_key_from_multibase(
    r#type: String,
    public_key_multibase: String,
) -> Result<Option<String>> {
    Ok(match r#type.as_str() {
        "EcdsaSecp256r1VerificationKey2019" => {
            Some(format_did_key_str(Algorithm::P256, &public_key_multibase)?)
        }
        "EcdsaSecp256k1VerificationKey2019" => Some(format_did_key_str(
            Algorithm::Secp256k1,
            &public_key_multibase,
        )?),
        "Multikey" => {
            let (alg, key) = parse_multikey(&public_key_multibase)?;
            Some(format_did_key(alg, &key)?)
        }
        _ => None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common_web::did_doc::{Service, VerificationMethod};

    #[test]
    fn extract_from_legacy_format() {
        let did_doc = DidDocument {
            context: Some(vec![
                String::from("https://www.w3.org/ns/did/v1"),
                String::from("https://w3id.org/security/suites/secp256k1-2019/v1"),
            ]),
            id: String::from("did:plc:yk4dd2qkboz2yv6tpubpc6co"),
            also_known_as: Some(vec![String::from("at://dholms.xyz")]),
            verification_method: Some(vec![VerificationMethod {
                id: String::from("#atproto"),
                r#type: String::from("EcdsaSecp256k1VerificationKey2019"),
                controller: String::from("did:plc:yk4dd2qkboz2yv6tpubpc6co"),
                public_key_multibase: Some(String::from(
                    "zQYEBzXeuTM9UR3rfvNag6L3RNAs5pQZyYPsomTsgQhsxLdEgCrPTLgFna8yqCnxPpNT7DBk6Ym3dgPKNu86vt9GR",
                )),
            }]),
            service: Some(vec![Service {
                id: String::from("#atproto_pds"),
                r#type: String::from("AtprotoPersonalDataServer"),
                service_endpoint: String::from("https://bsky.social"),
            }]),
        };
        let atp_data = ensure_atproto_data(&did_doc)
            .expect("ensure_atproto_data should succeed with legacy DID format");
        assert_eq!(
            atp_data,
            AtprotoData {
                did: String::from("did:plc:yk4dd2qkboz2yv6tpubpc6co"),
                signing_key: String::from(
                    "did:key:zQ3shXjHeiBuRCKmM36cuYnm7YEMzhGnCmCyW92sRJ9pribSF"
                ),
                handle: String::from("dholms.xyz"),
                pds: String::from("https://bsky.social"),
            }
        );
    }

    #[test]
    fn extract_from_newer_format() {
        let did_doc = DidDocument {
            context: Some(vec![
                String::from("https://www.w3.org/ns/did/v1"),
                String::from("https://w3id.org/security/multikey/v1"),
                String::from("https://w3id.org/security/suites/secp256k1-2019/v1"),
            ]),
            id: String::from("did:plc:yk4dd2qkboz2yv6tpubpc6co"),
            also_known_as: Some(vec![String::from("at://dholms.xyz")]),
            verification_method: Some(vec![VerificationMethod {
                id: String::from("did:plc:yk4dd2qkboz2yv6tpubpc6co#atproto"),
                r#type: String::from("Multikey"),
                controller: String::from("did:plc:yk4dd2qkboz2yv6tpubpc6co"),
                public_key_multibase: Some(String::from(
                    "zQ3shXjHeiBuRCKmM36cuYnm7YEMzhGnCmCyW92sRJ9pribSF",
                )),
            }]),
            service: Some(vec![Service {
                id: String::from("#atproto_pds"),
                r#type: String::from("AtprotoPersonalDataServer"),
                service_endpoint: String::from("https://bsky.social"),
            }]),
        };
        let atp_data = ensure_atproto_data(&did_doc)
            .expect("ensure_atproto_data should succeed with legacy DID format");
        assert_eq!(
            atp_data,
            AtprotoData {
                did: String::from("did:plc:yk4dd2qkboz2yv6tpubpc6co"),
                signing_key: String::from(
                    "did:key:zQ3shXjHeiBuRCKmM36cuYnm7YEMzhGnCmCyW92sRJ9pribSF"
                ),
                handle: String::from("dholms.xyz"),
                pds: String::from("https://bsky.social"),
            }
        );
    }
}
