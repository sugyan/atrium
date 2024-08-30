use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct Claims {
    #[serde(flatten)]
    pub registered: RegisteredClaims,
}

// https://datatracker.ietf.org/doc/html/rfc7519#section-4.1
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct RegisteredClaims {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub iss: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sub: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aud: Option<RegisteredClaimsAud>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exp: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nbf: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub iat: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub jti: Option<String>,
    // htm: String,
    // htu: String,
    // #[serde(skip_serializing_if = "Option::is_none")]
    // nonce: Option<String>,
}

impl From<RegisteredClaims> for Claims {
    fn from(registered: RegisteredClaims) -> Self {
        Self { registered }
    }
}

// https://datatracker.ietf.org/doc/html/rfc7519#section-4.1.3
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RegisteredClaimsAud {
    Single(String),
    #[allow(dead_code)]
    Multiple(Vec<String>),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialize_claims() {
        // empty
        {
            let claims = Claims::default();
            let json = serde_json::to_string(&claims).expect("failed to serialize claims");
            assert_eq!(json, "{}");
        }
        // single aud
        {
            let claims = Claims {
                registered: RegisteredClaims {
                    aud: Some(RegisteredClaimsAud::Single(String::from("client"))),
                    ..Default::default()
                },
            };
            let json = serde_json::to_string(&claims).expect("failed to serialize claims");
            assert_eq!(json, r#"{"aud":"client"}"#);
        }
        // multiple auds
        {
            let claims = Claims {
                registered: RegisteredClaims {
                    aud: Some(RegisteredClaimsAud::Multiple(vec![
                        String::from("client1"),
                        String::from("client2"),
                    ])),
                    ..Default::default()
                },
            };
            let json = serde_json::to_string(&claims).expect("failed to serialize claims");
            assert_eq!(json, r#"{"aud":["client1","client2"]}"#);
        }
    }
}
