//! Definitions for AT Protocol's data models.
//! <https://atproto.com/specs/data-model>

use std::{cell::OnceCell, fmt, ops::Deref, str::FromStr};

use regex::Regex;

mod cid_link;
pub use cid_link::CidLink;

mod integer;
pub use integer::*;

pub mod string;

/// Trait for a collection of records that can be stored in a repository.
///
/// The records all have the same Lexicon schema.
pub trait Collection: fmt::Debug {
    /// The NSID for the Lexicon that defines the schema of records in this collection.
    const NSID: &'static str;

    /// This collection's record type.
    type Record: fmt::Debug + serde::de::DeserializeOwned + serde::Serialize;

    /// Returns the [`Nsid`] for the Lexicon that defines the schema of records in this
    /// collection.
    ///
    /// This is a convenience method that parses [`Self::NSID`].
    ///
    /// # Panics
    ///
    /// Panics if [`Self::NSID`] is not a valid NSID.
    ///
    /// [`Nsid`]: string::Nsid
    fn nsid() -> string::Nsid {
        Self::NSID
            .parse()
            .expect("Self::NSID should be a valid NSID")
    }

    /// Returns the repo path for a record in this collection with the given record key.
    ///
    /// Per the [Repo Data Structure v3] specification:
    /// > Repo paths currently have a fixed structure of `<collection>/<record-key>`. This
    /// > means a valid, normalized [`Nsid`], followed by a `/`, followed by a valid
    /// > [`RecordKey`].
    ///
    /// [Repo Data Structure v3]: https://atproto.com/specs/repository#repo-data-structure-v3
    /// [`Nsid`]: string::Nsid
    fn repo_path(rkey: &RecordKey) -> String {
        format!("{}/{}", Self::NSID, rkey.as_str())
    }
}

/// A record key (`rkey`) used to name and reference an individual record within the same
/// collection of an atproto repository.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct RecordKey(String);

impl RecordKey {
    /// Returns the record key as a string slice.
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl FromStr for RecordKey {
    type Err = &'static str;

    #[allow(
        clippy::borrow_interior_mutable_const,
        clippy::declare_interior_mutable_const
    )]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        const RE_RKEY: OnceCell<Regex> = OnceCell::new();

        if [".", ".."].contains(&s) {
            Err("Disallowed rkey")
        } else if !RE_RKEY
            .get_or_init(|| Regex::new(r"^[a-zA-Z0-9._~-]{1,512}$").unwrap())
            .is_match(s)
        {
            Err("Invalid rkey")
        } else {
            Ok(Self(s.into()))
        }
    }
}

impl<'de> serde::Deserialize<'de> for RecordKey {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::Error;
        let value = serde::Deserialize::deserialize(deserializer)?;
        Self::from_str(value).map_err(D::Error::custom)
    }
}

impl From<RecordKey> for String {
    fn from(value: RecordKey) -> Self {
        value.0
    }
}

impl AsRef<str> for RecordKey {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl Deref for RecordKey {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}

/// Definitions for Blob types.
/// Usually a map with `$type` is used, but deprecated legacy formats are also supported for parsing.
/// <https://atproto.com/specs/data-model#blob-type>
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(untagged)]
pub enum BlobRef {
    Typed(TypedBlobRef),
    Untyped(UnTypedBlobRef),
}

/// Current, typed blob reference.
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(tag = "$type", rename_all = "lowercase")]
pub enum TypedBlobRef {
    Blob(Blob),
}

/// An untyped blob reference.
/// Some records in the wild still contain this format, but should never write them.
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct UnTypedBlobRef {
    pub cid: String,
    pub mime_type: String,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Blob {
    pub r#ref: CidLink,
    pub mime_type: String,
    pub size: usize, // TODO
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{from_str, to_string};

    const CID_LINK_JSON: &str =
        r#"{"$link":"bafkreibme22gw2h7y2h7tg2fhqotaqjucnbc24deqo72b6mkl2egezxhvy"}"#;

    #[test]
    fn valid_rkey() {
        // From https://atproto.com/specs/record-key#examples
        for valid in &["3jui7kd54zh2y", "self", "example.com", "~1.2-3_", "dHJ1ZQ"] {
            assert!(
                from_str::<RecordKey>(&format!("\"{}\"", valid)).is_ok(),
                "valid rkey `{}` parsed as invalid",
                valid,
            );
        }
    }

    #[test]
    fn invalid_rkey() {
        // From https://atproto.com/specs/record-key#examples
        for invalid in &[
            "literal:self",
            "alpha/beta",
            ".",
            "..",
            "#extra",
            "@handle",
            "any space",
            "any+space",
            "number[3]",
            "number(3)",
            "\"quote\"",
            "pre:fix",
            "dHJ1ZQ==",
        ] {
            assert!(
                from_str::<RecordKey>(&format!("\"{}\"", invalid)).is_err(),
                "invalid rkey `{}` parsed as valid",
                invalid,
            );
        }
    }

    #[test]
    fn test_cid_link_serde_json() {
        let deserialized =
            from_str::<CidLink>(CID_LINK_JSON).expect("failed to deserialize cid-link");
        let serialized = to_string(&deserialized).expect("failed to serialize cid-link");
        assert_eq!(serialized, CID_LINK_JSON);
    }

    #[test]
    fn test_blob_ref_typed_deserialize_json() {
        let json = format!(
            r#"{{"$type":"blob","ref":{},"mimeType":"text/plain","size":0}}"#,
            CID_LINK_JSON
        );
        let deserialized = from_str::<BlobRef>(&json).expect("failed to deserialize blob-ref");
        assert_eq!(
            deserialized,
            BlobRef::Typed(TypedBlobRef::Blob(Blob {
                r#ref: CidLink::try_from(
                    "bafkreibme22gw2h7y2h7tg2fhqotaqjucnbc24deqo72b6mkl2egezxhvy"
                )
                .expect("failed to create cid-link"),
                mime_type: "text/plain".into(),
                size: 0
            }))
        );
    }

    #[test]
    fn test_blob_ref_untyped_deserialize_json() {
        let json = r#"{"cid":"bafkreibme22gw2h7y2h7tg2fhqotaqjucnbc24deqo72b6mkl2egezxhvy","mimeType":"text/plain"}"#;
        let deserialized = from_str::<BlobRef>(json).expect("failed to deserialize blob-ref");
        assert_eq!(
            deserialized,
            BlobRef::Untyped(UnTypedBlobRef {
                cid: "bafkreibme22gw2h7y2h7tg2fhqotaqjucnbc24deqo72b6mkl2egezxhvy".into(),
                mime_type: "text/plain".into(),
            })
        );
    }

    #[test]
    fn test_blob_ref_serialize_json() {
        let blob_ref = BlobRef::Typed(TypedBlobRef::Blob(Blob {
            r#ref: CidLink::try_from("bafkreibme22gw2h7y2h7tg2fhqotaqjucnbc24deqo72b6mkl2egezxhvy")
                .expect("failed to create cid-link"),
            mime_type: "text/plain".into(),
            size: 0,
        }));
        let serialized = to_string(&blob_ref).expect("failed to serialize blob-ref");
        assert_eq!(
            serialized,
            format!(
                r#"{{"$type":"blob","ref":{},"mimeType":"text/plain","size":0}}"#,
                CID_LINK_JSON
            )
        );
    }

    #[test]
    fn test_blob_ref_deserialize_dag_cbor() {
        // {"$type": "blob", "mimeType": "text/plain", "ref": bafkreibme22gw2h7y2h7tg2fhqotaqjucnbc24deqo72b6mkl2egezxhvy, "size": 0}
        let dag_cbor = [
            0xa4, 0x65, 0x24, 0x74, 0x79, 0x70, 0x65, 0x64, 0x62, 0x6c, 0x6f, 0x62, 0x63, 0x72,
            0x65, 0x66, 0xd8, 0x2a, 0x58, 0x25, 0x00, 0x01, 0x55, 0x12, 0x20, 0x2c, 0x26, 0xb4,
            0x6b, 0x68, 0xff, 0xc6, 0x8f, 0xf9, 0x9b, 0x45, 0x3c, 0x1d, 0x30, 0x41, 0x34, 0x13,
            0x42, 0x2d, 0x70, 0x64, 0x83, 0xbf, 0xa0, 0xf9, 0x8a, 0x5e, 0x88, 0x62, 0x66, 0xe7,
            0xae, 0x68, 0x6d, 0x69, 0x6d, 0x65, 0x54, 0x79, 0x70, 0x65, 0x6a, 0x74, 0x65, 0x78,
            0x74, 0x2f, 0x70, 0x6c, 0x61, 0x69, 0x6e, 0x64, 0x73, 0x69, 0x7a, 0x65, 0x00,
        ];
        let deserialized = serde_ipld_dagcbor::from_slice::<BlobRef>(dag_cbor.as_slice())
            .expect("failed to deserialize blob-ref");
        assert_eq!(
            deserialized,
            BlobRef::Typed(TypedBlobRef::Blob(Blob {
                r#ref: CidLink::try_from(
                    "bafkreibme22gw2h7y2h7tg2fhqotaqjucnbc24deqo72b6mkl2egezxhvy"
                )
                .expect("failed to create cid-link"),
                mime_type: "text/plain".into(),
                size: 0,
            }))
        );
    }
}
