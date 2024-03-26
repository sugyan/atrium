use ipld_core::cid::{Cid, Error};
use ipld_core::ipld::Ipld;
use serde::{Deserialize, Serialize};

/// Representation of an IPLD Link.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CidLink(pub Cid);

#[derive(Serialize, Deserialize)]
struct Link {
    #[serde(rename = "$link")]
    link: crate::types::string::Cid,
}

impl Serialize for CidLink {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        if serializer.is_human_readable() {
            Link {
                link: crate::types::string::Cid::new(self.0),
            }
            .serialize(serializer)
        } else {
            self.0.serialize(serializer)
        }
    }
}

impl<'de> Deserialize<'de> for CidLink {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let ipld = Ipld::deserialize(deserializer)?;
        match &ipld {
            Ipld::Link(cid) => {
                return Ok(Self(*cid));
            }
            Ipld::Map(map) => {
                if map.len() == 1 {
                    if let Some(Ipld::String(link)) = map.get("$link") {
                        return Ok(Self(
                            Cid::try_from(link.as_str()).map_err(serde::de::Error::custom)?,
                        ));
                    }
                }
            }
            _ => {}
        }
        Err(serde::de::Error::custom("Invalid cid-link"))
    }
}

impl TryFrom<&str> for CidLink {
    type Error = Error;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        Ok(Self(Cid::try_from(s)?))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_ipld_dagcbor::{from_slice, to_vec};
    use serde_json::{from_str, to_string};

    const CID_LINK_JSON: &str =
        r#"{"$link":"bafkreibme22gw2h7y2h7tg2fhqotaqjucnbc24deqo72b6mkl2egezxhvy"}"#;

    const CID_LINK_DAGCBOR: [u8; 41] = [
        0xd8, 0x2a, 0x58, 0x25, 0x00, 0x01, 0x55, 0x12, 0x20, 0x2c, 0x26, 0xb4, 0x6b, 0x68, 0xff,
        0xc6, 0x8f, 0xf9, 0x9b, 0x45, 0x3c, 0x1d, 0x30, 0x41, 0x34, 0x13, 0x42, 0x2d, 0x70, 0x64,
        0x83, 0xbf, 0xa0, 0xf9, 0x8a, 0x5e, 0x88, 0x62, 0x66, 0xe7, 0xae,
    ];

    fn cid() -> Cid {
        Cid::try_from("bafkreibme22gw2h7y2h7tg2fhqotaqjucnbc24deqo72b6mkl2egezxhvy").unwrap()
    }

    #[test]
    fn test_cid_link_serialize_json() {
        let cid_link = CidLink(cid());

        let serialized = to_string(&cid_link).expect("failed to serialize cid-link");
        assert_eq!(serialized, CID_LINK_JSON);
    }

    #[test]
    fn test_cid_link_serialize_dagcbor() {
        let cid_link = CidLink(cid());

        let serialized = to_vec(&cid_link).expect("failed to serialize cid-link");
        assert_eq!(serialized, CID_LINK_DAGCBOR);
    }

    #[test]
    fn test_cid_link_deserialize_json() {
        let deserialized =
            from_str::<CidLink>(CID_LINK_JSON).expect("failed to deserialize cid-link");

        assert_eq!(deserialized, CidLink(cid()));
    }

    #[test]
    fn test_cid_link_deserialize_dagcbor() {
        let deserialized =
            from_slice::<CidLink>(&CID_LINK_DAGCBOR).expect("failed to deserialize cid-link");

        assert_eq!(deserialized, CidLink(cid()));
    }

    #[test]
    fn test_cid_link_deserialize_any_json() {
        #[derive(Deserialize, Debug, PartialEq, Eq)]
        #[serde(untagged)]
        enum Enum {
            CidLink(CidLink),
        }

        let deserialized = from_str::<Enum>(CID_LINK_JSON).expect("failed to deserialize cid-link");
        assert_eq!(deserialized, Enum::CidLink(CidLink(cid())));
    }

    #[test]
    fn test_cid_link_deserialize_any_dagcbor() {
        #[derive(Deserialize, Debug, PartialEq, Eq)]
        #[serde(untagged)]
        enum Enum {
            CidLink(CidLink),
        }

        let deserialized =
            from_slice::<Enum>(&CID_LINK_DAGCBOR).expect("failed to deserialize cid-link");
        assert_eq!(deserialized, Enum::CidLink(CidLink(cid())));
    }

    #[test]
    fn test_cid_link_serde_json() {
        // let deserialized =
        //     from_str::<CidLink>(CID_LINK_JSON).expect("failed to deserialize cid-link");
        // let serialized = to_string(&deserialized).expect("failed to serialize cid-link");
        // assert_eq!(serialized, CID_LINK_JSON);

        let cid_link = CidLink(cid());
        let serialized = to_string(&cid_link).expect("failed to serialize cid-link");
        let deserialized =
            from_str::<CidLink>(&serialized).expect("failed to deserialize cid-link");
        assert_eq!(deserialized, cid_link);
    }

    #[test]
    fn test_cid_link_serde_dagcbor() {
        let deserialized =
            from_slice::<Cid>(&CID_LINK_DAGCBOR).expect("failed to deserialize cid-link");
        let serialized = to_vec(&deserialized).expect("failed to serialize cid-link");
        assert_eq!(serialized, CID_LINK_DAGCBOR);

        let cid_link = CidLink(cid());
        let serialized = to_vec(&cid_link).expect("failed to serialize cid-link");
        let deserialized =
            from_slice::<CidLink>(&serialized).expect("failed to deserialize cid-link");
        assert_eq!(deserialized, cid_link);
    }
}
