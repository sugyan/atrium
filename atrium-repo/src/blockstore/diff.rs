use std::collections::HashSet;

use ipld_core::cid::Cid;

use super::{AsyncBlockStoreRead, AsyncBlockStoreWrite, Error};

/// An extremely simple differencing blockstore layer. This tracks all CIDs that are created.
pub struct DiffBlockStore<S> {
    inner: S,
    blocks: HashSet<Cid>,
}

impl<S> DiffBlockStore<S> {
    pub fn wrap(inner: S) -> Self {
        Self { inner, blocks: HashSet::new() }
    }

    pub fn into_inner(self) -> S {
        self.inner
    }

    /// Return the CIDs of the blocks that have been written so far.
    pub fn blocks(&self) -> impl Iterator<Item = Cid> + '_ {
        self.blocks.iter().cloned()
    }
}

impl<S: AsyncBlockStoreRead> AsyncBlockStoreRead for DiffBlockStore<S> {
    async fn read_block_into(&mut self, cid: Cid, contents: &mut Vec<u8>) -> Result<(), Error> {
        self.inner.read_block_into(cid, contents).await
    }
}

impl<S: AsyncBlockStoreWrite> AsyncBlockStoreWrite for DiffBlockStore<S> {
    async fn write_block(&mut self, codec: u64, hash: u64, contents: &[u8]) -> Result<Cid, Error> {
        let cid = self.inner.write_block(codec, hash, contents).await?;
        self.blocks.insert(cid);
        Ok(cid)
    }
}
