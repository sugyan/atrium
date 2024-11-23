//! Definitions for AT Protocol's data models.
//! <https://atproto.com/specs/data-model>

use crate::error::Error;
use ipld_core::ipld::Ipld;
use ipld_core::serde::to_ipld;
use serde::{de, ser};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
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
    type Record: fmt::Debug + de::DeserializeOwned + Serialize;

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
        Self::NSID.parse().expect("Self::NSID should be a valid NSID")
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
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(untagged)]
pub enum BlobRef {
    Typed(TypedBlobRef),
    Untyped(UnTypedBlobRef),
}

/// Current, typed blob reference.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(tag = "$type", rename_all = "lowercase")]
pub enum TypedBlobRef {
    Blob(Blob),
}

/// An untyped blob reference.
/// Some records in the wild still contain this format, but should never write them.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct UnTypedBlobRef {
    pub cid: String,
    pub mime_type: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Blob {
    pub r#ref: CidLink,
    pub mime_type: String,
    pub size: usize, // TODO
}

/// A generic object type.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Object<T> {
    #[serde(flatten)]
    pub data: T,
    #[serde(flatten)]
    pub extra_data: Ipld,
}

impl<T> From<T> for Object<T> {
    fn from(data: T) -> Self {
        Self { data, extra_data: Ipld::Map(std::collections::BTreeMap::new()) }
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
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(untagged)]
pub enum Union<T> {
    Refs(T),
    Unknown(UnknownData),
}

/// Data with an unknown schema in an open [`Union`].
///
/// The data of variants represented by a map and include a `$type` field indicating the variant type.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct UnknownData {
    #[serde(rename = "$type")]
    pub r#type: String,
    #[serde(flatten)]
    pub data: Ipld,
}

/// Arbitrary data with no specific validation and no type-specific fields.
///
/// Corresponds to [the `unknown` field type].
///
/// [the `unknown` field type]: https://atproto.com/specs/lexicon#unknown
///
/// By using the [`TryFromUnknown`] trait, it is possible to convert to any type
/// that implements [`DeserializeOwned`](serde::de::DeserializeOwned).
///
/// ```
/// use atrium_api::types::{TryFromUnknown, Unknown};
///
/// #[derive(Debug, serde::Deserialize)]
/// struct Foo {
///     bar: i32,
/// }
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let value: Unknown = serde_json::from_str(r#"{"bar": 42}"#)?;
/// println!("{value:?}"); // Object({"bar": DataModel(42)})
///
/// let foo = Foo::try_from_unknown(value)?;
/// println!("{foo:?}"); // Foo { bar: 42 }
/// #     Ok(())
/// # }
/// ```
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(untagged)]
pub enum Unknown {
    Object(BTreeMap<String, DataModel>),
    Null,
    Other(DataModel),
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(try_from = "Ipld")]
pub struct DataModel(#[serde(serialize_with = "serialize_data_model")] Ipld);

fn serialize_data_model<S>(ipld: &Ipld, serializer: S) -> Result<S::Ok, S::Error>
where
    S: ser::Serializer,
{
    match ipld {
        Ipld::Float(_) => Err(ser::Error::custom("float values are not allowed in ATProtocol")),
        Ipld::List(list) => {
            if list.iter().any(|value| matches!(value, Ipld::Float(_))) {
                Err(ser::Error::custom("float values are not allowed in ATProtocol"))
            } else {
                list.iter().cloned().map(DataModel).collect::<Vec<_>>().serialize(serializer)
            }
        }
        Ipld::Map(map) => {
            if map.values().any(|value| matches!(value, Ipld::Float(_))) {
                Err(ser::Error::custom("float values are not allowed in ATProtocol"))
            } else {
                map.iter()
                    .map(|(k, v)| (k, DataModel(v.clone())))
                    .collect::<BTreeMap<_, _>>()
                    .serialize(serializer)
            }
        }
        Ipld::Link(link) => CidLink(*link).serialize(serializer),
        _ => ipld.serialize(serializer),
    }
}

impl Deref for DataModel {
    type Target = Ipld;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for DataModel {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl TryFrom<Ipld> for DataModel {
    type Error = Error;

    fn try_from(value: Ipld) -> Result<Self, Self::Error> {
        // Enforce the ATProto data model.
        // https://atproto.com/specs/data-model
        match value {
            Ipld::Float(_) => Err(Error::NotAllowed),
            Ipld::List(list) => {
                if list.iter().any(|value| matches!(value, Ipld::Float(_))) {
                    Err(Error::NotAllowed)
                } else {
                    Ok(DataModel(Ipld::List(list)))
                }
            }
            Ipld::Map(map) => {
                if map.values().any(|value| matches!(value, Ipld::Float(_))) {
                    Err(Error::NotAllowed)
                } else {
                    Ok(DataModel(Ipld::Map(map)))
                }
            }
            data => Ok(DataModel(data)),
        }
    }
}

/// Trait for types that can be deserialized from an [`Unknown`] value.
pub trait TryFromUnknown: Sized {
    type Error;

    fn try_from_unknown(value: Unknown) -> Result<Self, Self::Error>;
}

impl<T> TryFromUnknown for T
where
    T: de::DeserializeOwned,
{
    type Error = Error;

    fn try_from_unknown(value: Unknown) -> Result<Self, Self::Error> {
        // TODO: Fix this
        // In the current latest `ipld-core` 0.4.1, deserialize to structs containing untagged/internal tagged does not work correctly when `Ipld::Integer` is included.
        // https://github.com/ipld/rust-ipld-core/issues/19
        // (It should be possible to convert as follows)
        // ```
        // Ok(match value {
        //     Unknown::Object(map) => {
        //         T::deserialize(Ipld::Map(map.into_iter().map(|(k, v)| (k, v.0)).collect()))?
        //     }
        //     Unknown::Null => T::deserialize(Ipld::Null)?,
        //     Unknown::Other(data) => T::deserialize(data.0)?,
        // })
        // ```
        //
        // For the time being, until this problem is resolved, use the workaround of serializing once to a json string and then deserializing it.
        let json = serde_json::to_vec(&value).unwrap();
        Ok(serde_json::from_slice(&json).unwrap())
    }
}

/// Trait for types that can be serialized into an [`Unknown`] value.
pub trait TryIntoUnknown {
    type Error;

    fn try_into_unknown(self) -> Result<Unknown, Self::Error>;
}

impl<T> TryIntoUnknown for T
where
    T: Serialize,
{
    type Error = Error;

    fn try_into_unknown(self) -> Result<Unknown, Self::Error> {
        Ok(Unknown::Other(to_ipld(self)?.try_into()?))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ipld_core::cid::Cid;
    use serde_json::{from_str, to_string};

    const CID_LINK_JSON: &str =
        r#"{"$link":"bafkreibme22gw2h7y2h7tg2fhqotaqjucnbc24deqo72b6mkl2egezxhvy"}"#;

    #[test]
    fn cid_link_serde_json() {
        let deserialized =
            from_str::<CidLink>(CID_LINK_JSON).expect("failed to deserialize cid-link");
        let serialized = to_string(&deserialized).expect("failed to serialize cid-link");
        assert_eq!(serialized, CID_LINK_JSON);
    }

    #[test]
    fn blob_ref_typed_deserialize_json() {
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
    fn blob_ref_untyped_deserialize_json() {
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
    fn blob_ref_serialize_json() {
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
    fn blob_ref_deserialize_dag_cbor() {
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
    fn data_model() {
        assert!(DataModel::try_from(Ipld::Null).is_ok());
        assert!(DataModel::try_from(Ipld::Bool(true)).is_ok());
        assert!(DataModel::try_from(Ipld::Integer(1)).is_ok());
        assert!(DataModel::try_from(Ipld::Float(1.5)).is_err(), "float value should fail");
        assert!(DataModel::try_from(Ipld::String("s".into())).is_ok());
        assert!(DataModel::try_from(Ipld::Bytes(vec![0x01])).is_ok());
        assert!(DataModel::try_from(Ipld::List(vec![Ipld::Bool(true)])).is_ok());
        assert!(
            DataModel::try_from(Ipld::List(vec![Ipld::Bool(true), Ipld::Float(1.5)])).is_err(),
            "list with float value should fail"
        );
        assert!(DataModel::try_from(Ipld::Map(BTreeMap::from_iter([(
            String::from("k"),
            Ipld::Bool(true)
        )])))
        .is_ok());
        assert!(
            DataModel::try_from(Ipld::Map(BTreeMap::from_iter([(
                String::from("k"),
                Ipld::Float(1.5)
            )])))
            .is_err(),
            "map with float value should fail"
        );
        assert!(DataModel::try_from(Ipld::Link(
            Cid::try_from("bafkreibme22gw2h7y2h7tg2fhqotaqjucnbc24deqo72b6mkl2egezxhvy")
                .expect("failed to create cid")
        ))
        .is_ok());
    }

    #[test]
    fn union() {
        #[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
        #[serde(tag = "$type")]
        enum FooRefs {
            #[serde(rename = "example.com#bar")]
            Bar(Box<Bar>),
            #[serde(rename = "example.com#baz")]
            Baz(Box<Baz>),
        }

        #[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
        struct Bar {
            bar: String,
        }

        #[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
        struct Baz {
            baz: i32,
        }

        type Foo = Union<FooRefs>;

        let foo = serde_json::from_str::<Foo>(r#"{"$type":"example.com#bar","bar":"bar"}"#)
            .expect("failed to deserialize foo");
        assert_eq!(foo, Union::Refs(FooRefs::Bar(Box::new(Bar { bar: String::from("bar") }))));

        let foo = serde_json::from_str::<Foo>(r#"{"$type":"example.com#baz","baz":42}"#)
            .expect("failed to deserialize foo");
        assert_eq!(foo, Union::Refs(FooRefs::Baz(Box::new(Baz { baz: 42 }))));

        let foo = serde_json::from_str::<Foo>(r#"{"$type":"example.com#foo","foo":true}"#)
            .expect("failed to deserialize foo");
        assert_eq!(
            foo,
            Union::Unknown(UnknownData {
                r#type: String::from("example.com#foo"),
                data: Ipld::Map(BTreeMap::from_iter([(String::from("foo"), Ipld::Bool(true))]))
            })
        );
    }

    #[test]
    fn unknown_serialize() {
        #[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
        struct Foo {
            foo: Unknown,
        }

        let foo = Foo {
            foo: Unknown::Object(BTreeMap::from_iter([(
                String::from("bar"),
                DataModel(Ipld::String(String::from("bar"))),
            )])),
        };
        let serialized = to_string(&foo).expect("failed to serialize foo");
        assert_eq!(serialized, r#"{"foo":{"bar":"bar"}}"#);
    }

    #[test]
    fn unknown_deserialize() {
        #[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
        struct Foo {
            foo: Unknown,
        }

        // valid: data object
        {
            let json = r#"{
                "foo": {
                    "$type": "example.com#foo",
                    "bar": "bar"
                }
            }"#;
            let deserialized = from_str::<Foo>(json).expect("failed to deserialize foo");
            assert_eq!(
                deserialized,
                Foo {
                    foo: Unknown::Object(BTreeMap::from_iter([
                        (String::from("bar"), DataModel(Ipld::String(String::from("bar")))),
                        (
                            String::from("$type"),
                            DataModel(Ipld::String(String::from("example.com#foo")))
                        )
                    ]))
                }
            );
        }
        // valid(?): empty object
        {
            let json = r#"{
                "foo": {}
            }"#;
            let deserialized = from_str::<Foo>(json).expect("failed to deserialize foo");
            assert_eq!(deserialized, Foo { foo: Unknown::Object(BTreeMap::default()) });
        }
        // valid(?): object with no `$type`
        {
            let json = r#"{
                "foo": {
                    "bar": "bar"
                }
            }"#;
            let deserialized = from_str::<Foo>(json).expect("failed to deserialize foo");
            assert_eq!(
                deserialized,
                Foo {
                    foo: Unknown::Object(BTreeMap::from_iter([(
                        String::from("bar"),
                        DataModel(Ipld::String(String::from("bar")))
                    )]))
                }
            );
        }
        // valid(?): null
        {
            let json = r#"{
                "foo": null
            }"#;
            let deserialized = from_str::<Foo>(json).expect("failed to deserialize foo");
            assert_eq!(deserialized, Foo { foo: Unknown::Null });
        }
        // valid(?): primitive types
        {
            let json = r#"{
                "foo": 42
            }"#;
            let deserialized = from_str::<Foo>(json).expect("failed to deserialize foo");
            assert_eq!(deserialized, Foo { foo: Unknown::Other(DataModel(Ipld::Integer(42))) });
        }
        // invalid: float (not allowed)
        {
            let json = r#"{
                "foo": 42.195
            }"#;
            assert!(from_str::<Foo>(json).is_err());
        }
    }

    #[test]
    fn unknown_try_from() {
        #[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
        #[serde(tag = "$type")]
        enum Foo {
            #[serde(rename = "example.com#bar")]
            Bar(Box<Bar>),
            #[serde(rename = "example.com#baz")]
            Baz(Box<Baz>),
        }

        #[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
        struct Bar {
            bar: String,
        }

        #[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
        struct Baz {
            baz: i32,
        }

        {
            let unknown = Unknown::Object(BTreeMap::from_iter([
                (String::from("$type"), DataModel(Ipld::String(String::from("example.com#bar")))),
                (String::from("bar"), DataModel(Ipld::String(String::from("barbar")))),
            ]));
            let bar = Bar::try_from_unknown(unknown.clone()).expect("failed to convert to Bar");
            assert_eq!(bar, Bar { bar: String::from("barbar") });
            let barbaz = Foo::try_from_unknown(unknown).expect("failed to convert to Bar");
            assert_eq!(barbaz, Foo::Bar(Box::new(Bar { bar: String::from("barbar") })));
        }
        {
            let unknown = Unknown::Object(BTreeMap::from_iter([
                (String::from("$type"), DataModel(Ipld::String(String::from("example.com#baz")))),
                (String::from("baz"), DataModel(Ipld::Integer(42))),
            ]));
            let baz = Baz::try_from_unknown(unknown.clone()).expect("failed to convert to Baz");
            assert_eq!(baz, Baz { baz: 42 });
            let barbaz = Foo::try_from_unknown(unknown).expect("failed to convert to Bar");
            assert_eq!(barbaz, Foo::Baz(Box::new(Baz { baz: 42 })));
        }
    }

    #[test]
    fn serialize_unknown_from_cid_link() {
        // cid link
        {
            let cid_link =
                CidLink::try_from("bafkreibme22gw2h7y2h7tg2fhqotaqjucnbc24deqo72b6mkl2egezxhvy")
                    .expect("failed to create cid-link");
            let unknown = cid_link.try_into_unknown().expect("failed to convert to unknown");
            assert_eq!(
                serde_json::to_string(&unknown).expect("failed to serialize unknown"),
                r#"{"$link":"bafkreibme22gw2h7y2h7tg2fhqotaqjucnbc24deqo72b6mkl2egezxhvy"}"#
            );
        }
        // blob ref (includes cid link)
        {
            let cid_link =
                CidLink::try_from("bafkreibme22gw2h7y2h7tg2fhqotaqjucnbc24deqo72b6mkl2egezxhvy")
                    .expect("failed to create cid-link");
            let blob_ref = BlobRef::Typed(TypedBlobRef::Blob(Blob {
                r#ref: cid_link,
                mime_type: "text/plain".into(),
                size: 0,
            }));
            let unknown = blob_ref.try_into_unknown().expect("failed to convert to unknown");
            let serialized = serde_json::to_string(&unknown).expect("failed to serialize unknown");
            assert!(
                serialized.contains("bafkreibme22gw2h7y2h7tg2fhqotaqjucnbc24deqo72b6mkl2egezxhvy"),
                "serialized unknown should contain cid string: {serialized}"
            );
        }
    }
}
