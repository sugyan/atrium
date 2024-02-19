//! Lexicon [string formats].
//!
//! [string formats]: https://atproto.com/specs/lexicon#string-formats

use std::{cell::OnceCell, ops::Deref, str::FromStr};

use regex::Regex;
use serde::{de::Error, Deserialize, Deserializer, Serialize};

/// Common trait implementations for Lexicon string formats that are newtype wrappers
/// around `String`.
macro_rules! string_newtype {
    ($name:ident) => {
        impl FromStr for $name {
            type Err = &'static str;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                Self::new(s.into())
            }
        }

        impl<'de> Deserialize<'de> for $name {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: Deserializer<'de>,
            {
                let value = Deserialize::deserialize(deserializer)?;
                Self::new(value).map_err(D::Error::custom)
            }
        }

        impl Into<String> for $name {
            fn into(self) -> String {
                self.0
            }
        }

        impl AsRef<str> for $name {
            fn as_ref(&self) -> &str {
                self.as_str()
            }
        }

        impl Deref for $name {
            type Target = str;

            fn deref(&self) -> &Self::Target {
                self.as_str()
            }
        }
    };
}

/// An AT Protocol identifier.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(untagged)]
pub enum AtIdentifier {
    Did(Did),
    Handle(Handle),
}

impl From<Did> for AtIdentifier {
    fn from(did: Did) -> Self {
        AtIdentifier::Did(did)
    }
}

impl From<Handle> for AtIdentifier {
    fn from(handle: Handle) -> Self {
        AtIdentifier::Handle(handle)
    }
}

impl FromStr for AtIdentifier {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Ok(did) = s.parse() {
            Ok(AtIdentifier::Did(did))
        } else {
            s.parse().map(AtIdentifier::Handle)
        }
    }
}

impl Into<String> for AtIdentifier {
    fn into(self) -> String {
        match self {
            AtIdentifier::Did(did) => did.into(),
            AtIdentifier::Handle(handle) => handle.into(),
        }
    }
}

impl AsRef<str> for AtIdentifier {
    fn as_ref(&self) -> &str {
        match self {
            AtIdentifier::Did(did) => did.as_ref(),
            AtIdentifier::Handle(handle) => handle.as_ref(),
        }
    }
}

/// A generic [DID Identifier].
///
/// [DID Identifier]: https://atproto.com/specs/did
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(transparent)]
pub struct Did(String);
string_newtype!(Did);

impl Did {
    /// Parses a `Did` from the given string.
    pub fn new(did: String) -> Result<Self, &'static str> {
        const RE_DID: OnceCell<Regex> = OnceCell::new();

        // https://atproto.com/specs/did#at-protocol-did-identifier-syntax
        if did.len() > 2048 {
            Err("DID too long")
        } else if !RE_DID
            .get_or_init(|| Regex::new(r"^did:[a-z]+:[a-zA-Z0-9._:%-]*[a-zA-Z0-9._-]$").unwrap())
            .is_match(&did)
        {
            Err("Invalid DID")
        } else {
            Ok(Self(did))
        }
    }

    /// Returns the DID method.
    pub fn method(&self) -> &str {
        &self.0[..4 + self.0[4..].find(':').unwrap()]
    }

    /// Returns the DID as a string slice.
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

/// A [Handle Identifier].
///
/// [Handle Identifier]: https://atproto.com/specs/handle
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(transparent)]
pub struct Handle(String);
string_newtype!(Handle);

impl Handle {
    /// Parses a `Handle` from the given string.
    pub fn new(handle: String) -> Result<Self, &'static str> {
        const RE_HANDLE: OnceCell<Regex> = OnceCell::new();

        // https://atproto.com/specs/handle#handle-identifier-syntax
        if handle.len() > 253 {
            Err("Handle too long")
        } else if !RE_HANDLE
            .get_or_init(|| Regex::new(r"^([a-zA-Z0-9]([a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?\.)+[a-zA-Z]([a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?$").unwrap())
            .is_match(&handle)
        {
            Err("Invalid handle")
        } else {
            Ok(Self(handle))
        }
    }

    /// Returns the handle as a string slice.
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

#[cfg(test)]
mod tests {
    use serde_json::from_str;

    use super::*;

    #[test]
    fn valid_did() {
        // From https://atproto.com/specs/did#examples
        for valid in &[
            "did:plc:z72i7hdynmk6r22z27h6tvur",
            "did:web:blueskyweb.xyz",
            "did:method:val:two",
            "did:m:v",
            "did:method::::val",
            "did:method:-:_:.",
            "did:key:zQ3shZc2QzApp2oymGvQbzP8eKheVshBHbU4ZYjeXqwSKEn6N",
        ] {
            assert!(
                from_str::<Did>(&format!("\"{}\"", valid)).is_ok(),
                "valid DID `{}` parsed as invalid",
                valid,
            );
        }
    }

    #[test]
    fn invalid_did() {
        // From https://atproto.com/specs/did#examples
        for invalid in &[
            "did:METHOD:val",
            "did:m123:val",
            "DID:method:val",
            "did:method:",
            "did:method:val/two",
            "did:method:val?two",
            "did:method:val#two",
        ] {
            assert!(
                from_str::<Did>(&format!("\"{}\"", invalid)).is_err(),
                "invalid DID `{}` parsed as valid",
                invalid,
            );
        }
    }

    #[test]
    fn did_method() {
        // From https://atproto.com/specs/did#examples
        for (method, did) in &[
            ("did:plc", "did:plc:z72i7hdynmk6r22z27h6tvur"),
            ("did:web", "did:web:blueskyweb.xyz"),
            ("did:method", "did:method:val:two"),
            ("did:m", "did:m:v"),
            ("did:method", "did:method::::val"),
            ("did:method", "did:method:-:_:."),
            (
                "did:key",
                "did:key:zQ3shZc2QzApp2oymGvQbzP8eKheVshBHbU4ZYjeXqwSKEn6N",
            ),
        ] {
            assert_eq!(Did::new(did.to_string()).unwrap().method(), *method);
        }
    }

    #[test]
    fn valid_handle() {
        // From https://atproto.com/specs/handle#identifier-examples
        for valid in &[
            "jay.bsky.social",
            "8.cn",
            "name.t--t", // not a real TLD, but syntax ok
            "XX.LCS.MIT.EDU",
            "a.co",
            "xn--notarealidn.com",
            "xn--fiqa61au8b7zsevnm8ak20mc4a87e.xn--fiqs8s",
            "xn--ls8h.test",
            "example.t", // not a real TLD, but syntax ok
            // Valid syntax, but must always fail resolution due to other restrictions:
            "2gzyxa5ihm7nsggfxnu52rck2vv4rvmdlkiu3zzui5du4xyclen53wid.onion",
            "laptop.local",
            "blah.arpa",
        ] {
            assert!(
                from_str::<Handle>(&format!("\"{}\"", valid)).is_ok(),
                "valid handle `{}` parsed as invalid",
                valid,
            );
        }
    }

    #[test]
    fn invalid_handle() {
        // From https://atproto.com/specs/handle#identifier-examples
        for invalid in &[
            "jo@hn.test",
            "💩.test",
            "john..test",
            "xn--bcher-.tld",
            "john.0",
            "cn.8",
            "www.masełkowski.pl.com",
            "org",
            "name.org.",
        ] {
            assert!(
                from_str::<Handle>(&format!("\"{}\"", invalid)).is_err(),
                "invalid handle `{}` parsed as valid",
                invalid,
            );
        }
    }
}
