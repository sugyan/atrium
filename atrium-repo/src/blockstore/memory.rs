use std::collections::HashMap;

use ipld_core::cid::{multihash::Multihash, Cid};
use sha2::Digest;

use super::{AsyncBlockStoreRead, AsyncBlockStoreWrite, Error, SHA2_256};

/// Basic in-memory blockstore. This is primarily used for testing.
pub struct MemoryBlockStore {
    blocks: HashMap<Cid, Vec<u8>>,
}

impl Default for MemoryBlockStore {
    fn default() -> Self {
        Self::new()
    }
}

impl MemoryBlockStore {
    pub fn new() -> Self {
        Self { blocks: HashMap::new() }
    }
}

impl AsyncBlockStoreRead for MemoryBlockStore {
    async fn read_block_into(&mut self, cid: Cid, contents: &mut Vec<u8>) -> Result<(), Error> {
        contents.clear();
        contents.extend_from_slice(self.blocks.get(&cid).ok_or(Error::CidNotFound)?);
        Ok(())
    }
}

impl AsyncBlockStoreWrite for MemoryBlockStore {
    async fn write_block(&mut self, codec: u64, hash: u64, contents: &[u8]) -> Result<Cid, Error> {
        let digest = match hash {
            SHA2_256 => sha2::Sha256::digest(contents),
            _ => return Err(Error::UnsupportedHash(hash)),
        };
        let hash =
            Multihash::wrap(hash, digest.as_slice()).expect("internal error encoding multihash");
        let cid = Cid::new_v1(codec, hash);

        // Insert the block. We're explicitly ignoring the case where it's already present inside the hashmap.
        self.blocks.insert(cid, contents.to_vec());
        Ok(cid)
    }
}
