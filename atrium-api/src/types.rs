//! Definitions for AT Protocol's data models.
//! <https://atproto.com/specs/data-model>

#[cfg(feature = "dag-cbor")]
mod cid_link_ipld;
#[cfg(not(feature = "dag-cbor"))]
mod cid_link_json;

#[cfg(feature = "dag-cbor")]
pub use cid_link_ipld::CidLink;
#[cfg(not(feature = "dag-cbor"))]
pub use cid_link_json::CidLink;

mod integer;
pub use integer::*;

pub mod string;

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

    #[cfg(feature = "dag-cbor")]
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
