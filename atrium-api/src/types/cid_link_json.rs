use serde::{Deserialize, Serialize};

/// Representation of an IPLD Link.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct CidLink {
    #[serde(rename = "$link")]
    pub link: String,
}

impl TryFrom<&str> for CidLink {
    type Error = ();

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        Ok(Self {
            link: s.to_string(),
        })
    }
}
