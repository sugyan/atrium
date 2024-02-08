#![feature(test)]

extern crate test;
use serde::Deserialize;
use test::Bencher;

const JSON_STR: &str = r#"{
    "$link": "bafkreibme22gw2h7y2h7tg2fhqotaqjucnbc24deqo72b6mkl2egezxhvy"
}"#;

const CBOR_CID: [u8; 41] = [
    0xd8, 0x2a, 0x58, 0x25, 0x00, 0x01, 0x55, 0x12, 0x20, 0x2c, 0x26, 0xb4, 0x6b, 0x68, 0xff, 0xc6,
    0x8f, 0xf9, 0x9b, 0x45, 0x3c, 0x1d, 0x30, 0x41, 0x34, 0x13, 0x42, 0x2d, 0x70, 0x64, 0x83, 0xbf,
    0xa0, 0xf9, 0x8a, 0x5e, 0x88, 0x62, 0x66, 0xe7, 0xae,
];

mod via_ipld {
    use super::*;
    use libipld_core::ipld::Ipld;

    #[derive(PartialEq, Eq, Debug)]
    pub struct CidLink(pub cid::Cid);

    impl<'de> Deserialize<'de> for CidLink {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            let ipld = Ipld::deserialize(deserializer)?;
            match &ipld {
                Ipld::Link(cid) => {
                    return Ok(Self(*cid));
                }
                Ipld::Map(map) => {
                    if map.len() == 1 {
                        if let Some(Ipld::String(link)) = map.get("$link") {
                            return Ok(Self(
                                cid::Cid::try_from(link.as_str())
                                    .map_err(serde::de::Error::custom)?,
                            ));
                        }
                    }
                }
                _ => {}
            }
            Err(serde::de::Error::custom("Invalid cid-link"))
        }
    }
}

mod untagged_1 {
    use super::*;

    #[derive(Deserialize, PartialEq, Eq, Debug)]
    #[serde(untagged)]
    pub enum CidLink {
        Raw(cid::Cid),
        Object {
            #[serde(rename = "$link")]
            link: String,
        },
    }
}

mod untagged_2 {
    use super::*;

    #[derive(Deserialize, PartialEq, Eq, Debug)]
    #[serde(untagged)]
    pub enum CidLink {
        Object {
            #[serde(rename = "$link")]
            link: String,
        },
        Raw(cid::Cid),
    }
}

mod only_json {
    use super::*;

    #[derive(Deserialize, PartialEq, Eq, Debug)]

    pub struct CidLink {
        #[serde(rename = "$link")]
        pub link: String,
    }
}

mod only_cbor {
    use super::*;

    #[derive(Deserialize, PartialEq, Eq, Debug)]

    pub struct CidLink(pub cid::Cid);
}

fn cid() -> cid::Cid {
    serde_ipld_dagcbor::from_slice::<cid::Cid>(&CBOR_CID).expect("failed to deserialize cid")
}

#[bench]
fn bench_cbor_untagged_1(b: &mut Bencher) {
    let expected = untagged_1::CidLink::Raw(cid());

    b.iter(|| {
        let result = serde_ipld_dagcbor::from_slice::<untagged_1::CidLink>(&CBOR_CID)
            .expect("failed to deserialize cid_link");
        assert_eq!(result, expected);
    });
}

#[bench]
fn bench_json_untagged_1(b: &mut Bencher) {
    let expected = untagged_1::CidLink::Object {
        link: cid().to_string(),
    };

    b.iter(|| {
        let result = serde_json::from_str::<untagged_1::CidLink>(JSON_STR)
            .expect("failed to deserialize cid_link");
        assert_eq!(result, expected);
    });
}

#[bench]
fn bench_cbor_untagged_2(b: &mut Bencher) {
    let expected = untagged_2::CidLink::Raw(cid());

    b.iter(|| {
        let result = serde_ipld_dagcbor::from_slice::<untagged_2::CidLink>(&CBOR_CID)
            .expect("failed to deserialize cid_link");
        assert_eq!(result, expected);
    });
}

#[bench]
fn bench_json_untagged_2(b: &mut Bencher) {
    let expected = untagged_2::CidLink::Object {
        link: cid().to_string(),
    };

    b.iter(|| {
        let result = serde_json::from_str::<untagged_2::CidLink>(JSON_STR)
            .expect("failed to deserialize cid_link");
        assert_eq!(result, expected);
    });
}

#[bench]
fn bench_cbor_via_ipld(b: &mut Bencher) {
    let expected = via_ipld::CidLink(cid());

    b.iter(|| {
        let result = serde_ipld_dagcbor::from_slice::<via_ipld::CidLink>(&CBOR_CID)
            .expect("failed to deserialize cid_link");
        assert_eq!(result, expected);
    });
}

#[bench]
fn bench_json_via_ipld(b: &mut Bencher) {
    let expected = via_ipld::CidLink(cid());

    b.iter(|| {
        let result = serde_json::from_str::<via_ipld::CidLink>(JSON_STR)
            .expect("failed to deserialize cid_link");
        assert_eq!(result, expected);
    });
}

#[bench]
fn bench_json_only(b: &mut Bencher) {
    let expected = only_json::CidLink {
        link: cid().to_string(),
    };

    b.iter(|| {
        let result = serde_json::from_str::<only_json::CidLink>(JSON_STR)
            .expect("failed to deserialize cid_link");
        assert_eq!(result, expected);
    });
}

#[bench]
fn bench_cbor_only(b: &mut Bencher) {
    let expected = only_cbor::CidLink(cid());

    b.iter(|| {
        let result = serde_ipld_dagcbor::from_slice::<only_cbor::CidLink>(&CBOR_CID)
            .expect("failed to deserialize cid_link");
        assert_eq!(result, expected);
    });
}
