use multibase::Base;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Algorithm {
    P256,
    Secp256k1,
}

impl Algorithm {
    const MULTICODE_PREFIX_P256: [u8; 2] = [0x80, 0x24];
    const MULTICODE_PREFIX_SECP256K1: [u8; 2] = [0xe7, 0x01];

    pub fn prefix(&self) -> [u8; 2] {
        match self {
            Self::P256 => Self::MULTICODE_PREFIX_P256,
            Self::Secp256k1 => Self::MULTICODE_PREFIX_SECP256K1,
        }
    }
    pub fn from_prefix(prefix: [u8; 2]) -> Option<Self> {
        match prefix {
            Self::MULTICODE_PREFIX_P256 => Some(Self::P256),
            Self::MULTICODE_PREFIX_SECP256K1 => Some(Self::Secp256k1),
            _ => None,
        }
    }
    pub(crate) fn format_mulikey_compressed(&self, key: &[u8]) -> String {
        let mut v = Vec::with_capacity(2 + key.len());
        v.extend_from_slice(&self.prefix());
        v.extend_from_slice(key);
        multibase::encode(Base::Base58Btc, v)
    }
}
