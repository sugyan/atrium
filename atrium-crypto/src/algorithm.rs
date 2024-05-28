use multibase::Base;

/// Supported algorithms (elliptic curves) for atproto cryptography.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Algorithm {
    /// [`p256`] elliptic curve: aka "NIST P-256", aka `secp256r1` (note the `r`), aka `prime256v1`.
    P256,
    /// [`k256`] elliptic curve: aka "NIST K-256", aka `secp256k1` (note the `k`).
    Secp256k1,
}

impl Algorithm {
    const MULTICODE_PREFIX_P256: [u8; 2] = [0x80, 0x24];
    const MULTICODE_PREFIX_SECP256K1: [u8; 2] = [0xe7, 0x01];

    pub(crate) fn prefix(&self) -> [u8; 2] {
        match self {
            Self::P256 => Self::MULTICODE_PREFIX_P256,
            Self::Secp256k1 => Self::MULTICODE_PREFIX_SECP256K1,
        }
    }
    pub(crate) fn from_prefix(prefix: [u8; 2]) -> Option<Self> {
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
