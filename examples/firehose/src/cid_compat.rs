use cid::{multihash::Multihash, Cid};

pub struct CidOld(cid_old::Cid);

impl From<cid_old::Cid> for CidOld {
    fn from(value: cid_old::Cid) -> Self {
        Self(value)
    }
}
impl TryFrom<CidOld> for Cid {
    type Error = cid::Error;
    fn try_from(value: CidOld) -> Result<Self, Self::Error> {
        let version = match value.0.version() {
            cid_old::Version::V0 => cid::Version::V0,
            cid_old::Version::V1 => cid::Version::V1,
        };

        let codec = value.0.codec();
        let hash = value.0.hash();
        let hash = Multihash::from_bytes(&hash.to_bytes())?;

        Self::new(version, codec, hash)
    }
}
