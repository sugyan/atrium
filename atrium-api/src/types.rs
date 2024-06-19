//! Definitions for AT Protocol's data models.
//! <https://atproto.com/specs/data-model>

use ipld_core::ipld::Ipld;
use std::fmt;
use std::ops::{Deref, DerefMut};

mod cid_link;
pub use cid_link::CidLink;

mod integer;
pub use integer::*;

pub mod string;
use string::RecordKey;

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

/// A generic object type.
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Object<T> {
    #[serde(flatten)]
    pub data: T,
    #[serde(flatten)]
    pub extra_data: Ipld,
}

impl<T> From<T> for Object<T> {
    fn from(data: T) -> Self {
        Self {
            data,
            extra_data: Ipld::Map(std::collections::BTreeMap::new()),
        }
    }
}

impl<T> Deref for Object<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<T> DerefMut for Object<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}

/// An "open" union type.
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(untagged)]
pub enum Union<T> {
    Refs(T),
    Unknown(UnknownData),
}

/// The data of variants represented by a map and include a `$type` field indicating the variant type.
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct UnknownData {
    #[serde(rename = "$type")]
    pub r#type: String,
    #[serde(flatten)]
    pub data: Ipld,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{from_str, to_string};
    use std::collections::BTreeMap;

    const CID_LINK_JSON: &str =
        r#"{"$link":"bafkreibme22gw2h7y2h7tg2fhqotaqjucnbc24deqo72b6mkl2egezxhvy"}"#;

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

    #[test]
    fn test_union() {
        #[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq, Eq)]
        #[serde(tag = "$type")]
        enum FooRefs {
            #[serde(rename = "example.com#bar")]
            Bar(Box<Bar>),
            #[serde(rename = "example.com#baz")]
            Baz(Box<Baz>),
        }

        #[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq, Eq)]
        struct Bar {
            bar: String,
        }

        #[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq, Eq)]
        struct Baz {
            baz: i32,
        }

        type Foo = Union<FooRefs>;

        let foo = serde_json::from_str::<Foo>(r#"{"$type":"example.com#bar","bar":"bar"}"#)
            .expect("failed to deserialize foo");
        assert_eq!(
            foo,
            Union::Refs(FooRefs::Bar(Box::new(Bar {
                bar: String::from("bar")
            })))
        );

        let foo = serde_json::from_str::<Foo>(r#"{"$type":"example.com#baz","baz":42}"#)
            .expect("failed to deserialize foo");
        assert_eq!(foo, Union::Refs(FooRefs::Baz(Box::new(Baz { baz: 42 }))));

        let foo = serde_json::from_str::<Foo>(r#"{"$type":"example.com#foo","foo":true}"#)
            .expect("failed to deserialize foo");
        assert_eq!(
            foo,
            Union::Unknown(UnknownData {
                r#type: String::from("example.com#foo"),
                data: Ipld::Map(BTreeMap::from_iter([(
                    String::from("foo"),
                    Ipld::Bool(true)
                )]))
            })
        );
    }
}
