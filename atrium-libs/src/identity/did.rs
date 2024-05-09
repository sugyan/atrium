use crate::common_web::did_doc::DidDocument;
#[cfg(feature = "crypto")]
mod atproto_data;
pub mod did_resolver;
mod error;
mod plc_resolver;
mod web_resolver;

use self::error::{Error, Result};
use async_trait::async_trait;

#[async_trait]
pub trait Fetch {
    async fn fetch(
        url: &str,
        timeout: Option<u64>,
    ) -> std::result::Result<Option<Vec<u8>>, Box<dyn std::error::Error + Send + Sync + 'static>>;
}

#[async_trait]
pub trait Resolve {
    async fn resolve_no_check(&self, did: &str) -> Result<Option<Vec<u8>>>;

    async fn resolve_no_cache(&self, did: &str) -> Result<Option<DidDocument>> {
        if let Some(got) = self.resolve_no_check(did).await? {
            Ok(serde_json::from_slice(&got)?)
        } else {
            Ok(None)
        }
    }
    async fn resolve(&self, did: &str, force_refresh: bool) -> Result<Option<DidDocument>> {
        // TODO: from cache
        if let Some(got) = self.resolve_no_cache(did).await? {
            // TODO: store in cache
            Ok(Some(got))
        } else {
            // TODO: clear cache
            Ok(None)
        }
    }
    async fn ensure_resolve(&self, did: &str, force_refresh: bool) -> Result<DidDocument> {
        self.resolve(did, force_refresh)
            .await?
            .ok_or_else(|| Error::DidNotFound(did.to_string()))
    }
}

pub fn validate_did_doc(did: &str, value: impl TryInto<DidDocument>) -> Result<DidDocument> {
    if let Ok(did_doc) = value.try_into() {
        if did_doc.get_did() == did {
            return Ok(did_doc);
        }
    }
    Err(Error::PoorlyFormattedDidDocument)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_bad_did_doc() {
        let err = validate_did_doc("did:plc:yk4dd2qkboz2yv6tpubpc6co", r##"
          {
            "ideep": "did:plc:yk4dd2qkboz2yv6tpubpc6co",
            "blah": [
              "https://dholms.xyz"
            ],
            "zoot": [
              {
                "id": "#elsewhere",
                "type": "EcdsaSecp256k1VerificationKey2019",
                "controller": "did:plc:yk4dd2qkboz2yv6tpubpc6co",
                "publicKeyMultibase": "zQYEBzXeuTM9UR3rfvNag6L3RNAs5pQZyYPsomTsgQhsxLdEgCrPTLgFna8yqCnxPpNT7DBk6Ym3dgPKNu86vt9GR"
              }
            ],
            "yarg": [ ]
          }
          "##,
        ).expect_err("validation should fail with bad DID document");
        assert!(matches!(err, Error::PoorlyFormattedDidDocument));
    }

    #[test]
    fn validate_legacy_format() {
        let did_doc = validate_did_doc(
            "did:plc:yk4dd2qkboz2yv6tpubpc6co",
            r##"
            {
              "@context": [
                "https://www.w3.org/ns/did/v1",
                "https://w3id.org/security/suites/secp256k1-2019/v1"
              ],
              "id": "did:plc:yk4dd2qkboz2yv6tpubpc6co",
              "alsoKnownAs": [
                "at://dholms.xyz"
              ],
              "verificationMethod": [
                {
                  "id": "#atproto",
                  "type": "EcdsaSecp256k1VerificationKey2019",
                  "controller": "did:plc:yk4dd2qkboz2yv6tpubpc6co",
                  "publicKeyMultibase": "zQYEBzXeuTM9UR3rfvNag6L3RNAs5pQZyYPsomTsgQhsxLdEgCrPTLgFna8yqCnxPpNT7DBk6Ym3dgPKNu86vt9GR"
                }
              ],
              "service": [
                {
                  "id": "#atproto_pds",
                  "type": "AtprotoPersonalDataServer",
                  "serviceEndpoint": "https://bsky.social"
                }
              ]
            }
        "##,
        )
        .expect("validation should succeed with legacy DID format");
        assert_eq!(did_doc.get_did(), "did:plc:yk4dd2qkboz2yv6tpubpc6co");
    }

    #[test]
    fn validate_newer_format() {
        let did_doc = validate_did_doc(
            "did:plc:yk4dd2qkboz2yv6tpubpc6co",
            r##"
            {
              "@context": [
                "https://www.w3.org/ns/did/v1",
                "https://w3id.org/security/multikey/v1",
                "https://w3id.org/security/suites/secp256k1-2019/v1"
              ],
              "id": "did:plc:yk4dd2qkboz2yv6tpubpc6co",
              "alsoKnownAs": [
                "at://dholms.xyz"
              ],
              "verificationMethod": [
                {
                  "id": "did:plc:yk4dd2qkboz2yv6tpubpc6co#atproto",
                  "type": "Multikey",
                  "controller": "did:plc:yk4dd2qkboz2yv6tpubpc6co",
                  "publicKeyMultibase": "zQ3shXjHeiBuRCKmM36cuYnm7YEMzhGnCmCyW92sRJ9pribSF"
                }
              ],
              "service": [
                {
                  "id": "#atproto_pds",
                  "type": "AtprotoPersonalDataServer",
                  "serviceEndpoint": "https://bsky.social"
                }
              ]
            }
        "##,
        )
        .expect("validation should succeed with newer Multikey DID format");
        assert_eq!(did_doc.get_did(), "did:plc:yk4dd2qkboz2yv6tpubpc6co");
    }
}
