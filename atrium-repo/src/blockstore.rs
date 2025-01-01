use std::future::Future;

use ipld_core::cid::Cid;

mod car;
mod memory;

pub use car::{CarStore, Error as CarError};
pub use memory::MemoryBlockStore;

/// DAG-PB multicodec code
pub const DAG_PB: u64 = 0x70;
/// DAG-CBOR multicodec code
pub const DAG_CBOR: u64 = 0x71;
/// The SHA_256 multihash code
pub const SHA2_256: u64 = 0x12;

pub trait AsyncBlockStoreRead: Send {
    /// Read a single block from the block store into the provided buffer.
    fn read_block_into(
        &mut self,
        cid: &Cid,
        contents: &mut Vec<u8>,
    ) -> impl Future<Output = Result<(), Error>> + Send;

    /// Read a single block from the block store.
    fn read_block(&mut self, cid: &Cid) -> impl Future<Output = Result<Vec<u8>, Error>> + Send {
        async {
            let mut contents = Vec::new();
            self.read_block_into(cid, &mut contents).await?;
            Ok(contents)
        }
    }
}

pub trait AsyncBlockStoreWrite: Send {
    /// Write a single block into the block store.
    /// This will return the block's computed hash.
    fn write_block(
        &mut self,
        codec: u64,
        hash: u64,
        contents: &[u8],
    ) -> impl Future<Output = Result<Cid, Error>> + Send;

    /// Delete a single block from the block store.
    fn delete_block(&mut self, cid: &Cid) -> impl Future<Output = Result<(), Error>> + Send;
}

impl<T: AsyncBlockStoreRead> AsyncBlockStoreRead for &mut T {
    fn read_block_into(
        &mut self,
        cid: &Cid,
        contents: &mut Vec<u8>,
    ) -> impl Future<Output = Result<(), Error>> + Send {
        (**self).read_block_into(cid, contents)
    }
}

impl<T: AsyncBlockStoreWrite> AsyncBlockStoreWrite for &mut T {
    fn write_block(
        &mut self,
        codec: u64,
        hash: u64,
        contents: &[u8],
    ) -> impl Future<Output = Result<Cid, Error>> + Send {
        (**self).write_block(codec, hash, contents)
    }

    fn delete_block(&mut self, cid: &Cid) -> impl Future<Output = Result<(), Error>> + Send {
        (**self).delete_block(cid)
    }
}

/// Errors that can occur while interacting with a block store.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("CID does not exist in block store")]
    CidNotFound,
    #[error("unsupported hashing algorithm")]
    UnsupportedHash(u64),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}
