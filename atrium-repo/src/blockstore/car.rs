use std::{collections::HashMap, convert::Infallible};

use futures::{AsyncReadExt as _, AsyncSeekExt as _};
use ipld_core::cid::{multihash::Multihash, Cid, Version};
use serde::{Deserialize, Serialize};
use sha2::Digest;
use tokio::io::{
    AsyncRead, AsyncReadExt as _, AsyncSeek, AsyncSeekExt as _, AsyncWrite, AsyncWriteExt as _,
    SeekFrom,
};
use tokio_util::compat::TokioAsyncReadCompatExt;
use unsigned_varint::io::ReadError;

use crate::blockstore::{self, AsyncBlockStoreRead, SHA2_256};

use super::AsyncBlockStoreWrite;

#[derive(Debug, Serialize, Deserialize)]
pub struct V1Header {
    pub version: u64,
    pub roots: Vec<Cid>,
}

async fn read_cid<R: futures::AsyncRead + futures::AsyncSeek + Unpin>(
    mut reader: R,
) -> Result<Cid, Error> {
    let version = unsigned_varint::aio::read_u64(&mut reader).await?;
    let codec = unsigned_varint::aio::read_u64(&mut reader).await?;

    // CIDv0 has the fixed `0x12 0x20` prefix
    if [version, codec] == [0x12, 0x20] {
        let mut digest = [0u8; 32];
        reader.read_exact(&mut digest).await?;
        let mh = Multihash::wrap(version, &digest).expect("Digest is always 32 bytes.");
        return Ok(Cid::new_v0(mh)?);
    }

    let version = Version::try_from(version)?;
    match version {
        Version::V0 => Err(Error::InvalidCidV0),
        Version::V1 => {
            let start = reader.stream_position().await?;
            let _code = unsigned_varint::aio::read_u64(&mut reader).await?;
            let size = unsigned_varint::aio::read_u64(&mut reader).await?;
            let len = (reader.stream_position().await? - start) + size;

            let mut mh_bytes = vec![0; len as usize];
            reader.seek(SeekFrom::Start(start)).await?;
            reader.read_exact(&mut mh_bytes).await?;

            let mh = Multihash::from_bytes(&mh_bytes)?;
            Ok(Cid::new(version, codec, mh)?)
        }
    }
}

/// An indexed reader/writer for CAR files.
#[derive(Debug)]
pub struct CarStore<S: AsyncRead + AsyncSeek> {
    storage: S,
    header: V1Header,
    index: HashMap<Cid, (u64, usize)>,
}

impl<R: AsyncRead + AsyncSeek + Unpin> CarStore<R> {
    /// Open a pre-existing CAR file.
    pub async fn open(mut storage: R) -> Result<Self, Error> {
        // Read the header.
        let header_len = unsigned_varint::aio::read_usize((&mut storage).compat()).await?;
        let mut header_bytes = vec![0; header_len];
        storage.read_exact(&mut header_bytes).await?;
        let header: V1Header = serde_ipld_dagcbor::from_slice(&header_bytes)?;

        let mut buffer = Vec::new();

        // Build the index.
        let mut index = HashMap::new();
        loop {
            match unsigned_varint::aio::read_u64((&mut storage).compat()).await {
                Ok(data_len) => {
                    let start = storage.stream_position().await?;
                    let cid = read_cid((&mut storage).compat()).await?;
                    let offset = storage.stream_position().await?;
                    let len = data_len - (offset - start);
                    // reader.seek(SeekFrom::Start(offset + len)).await?;

                    // Validate this block's multihash.
                    buffer.resize(len as usize, 0);
                    storage.read_exact(buffer.as_mut_slice()).await?;

                    let digest = match cid.hash().code() {
                        SHA2_256 => Some(sha2::Sha256::digest(buffer.as_slice())),
                        // FIXME: We should probably warn that we couldn't verify the block.
                        _ => None,
                    };

                    if let Some(digest) = digest {
                        let expected = Multihash::wrap(cid.hash().code(), digest.as_slice())
                            .map_err(Error::Multihash)?;
                        let expected = Cid::new_v1(cid.codec(), expected);

                        if expected != cid {
                            return Err(Error::InvalidHash);
                        }
                    }

                    index.insert(cid, (offset, len as usize));
                }
                Err(ReadError::Io(e)) if e.kind() == std::io::ErrorKind::UnexpectedEof => break,
                Err(e) => Err(e)?,
            }
        }

        Ok(Self { storage, header, index })
    }

    pub fn roots(&self) -> impl Iterator<Item = Cid> {
        self.header.roots.clone().into_iter()
    }
}

impl<S: AsyncRead + AsyncWrite + AsyncSeek + Send + Unpin> CarStore<S> {
    pub async fn create(mut storage: S) -> Result<Self, Error> {
        // HACK: Create a header with a single root entry (most commonly desired).
        // We do this here because it's hard to delete/insert bytes into a pre-existing file.
        let header = V1Header { version: 1, roots: vec![Cid::default()] };

        let header_bytes = serde_ipld_dagcbor::to_vec(&header).unwrap();
        let mut buf = unsigned_varint::encode::usize_buffer();
        let buf = unsigned_varint::encode::usize(header_bytes.len(), &mut buf);
        storage.write_all(buf).await?;
        storage.write_all(&header_bytes).await?;

        Ok(Self { storage, header, index: HashMap::new() })
    }

    pub async fn set_root(&mut self, root: Cid) -> Result<(), Error> {
        // HACK: The root array must be the same length in order to avoid shifting the file's contents.
        self.header.roots = vec![root];

        let header_bytes = serde_ipld_dagcbor::to_vec(&self.header).unwrap();
        let mut buf = unsigned_varint::encode::usize_buffer();
        let buf = unsigned_varint::encode::usize(header_bytes.len(), &mut buf);
        self.storage.seek(SeekFrom::Start(0)).await?;
        self.storage.write_all(buf).await?;
        self.storage.write_all(&header_bytes).await?;

        Ok(())
    }
}

impl<R: AsyncRead + AsyncSeek + Send + Unpin> AsyncBlockStoreRead for CarStore<R> {
    async fn read_block_into(
        &mut self,
        cid: &Cid,
        contents: &mut Vec<u8>,
    ) -> Result<(), blockstore::Error> {
        contents.clear();

        let (offset, len) = self.index.get(cid).ok_or_else(|| blockstore::Error::CidNotFound)?;
        contents.resize(*len, 0);

        self.storage.seek(SeekFrom::Start(*offset)).await?;
        self.storage.read_exact(contents).await?;

        Ok(())
    }
}

impl<R: AsyncRead + AsyncWrite + AsyncSeek + Send + Unpin> AsyncBlockStoreWrite for CarStore<R> {
    async fn write_block(
        &mut self,
        codec: u64,
        hash: u64,
        contents: &[u8],
    ) -> Result<Cid, blockstore::Error> {
        let digest = match hash {
            SHA2_256 => sha2::Sha256::digest(contents),
            _ => return Err(blockstore::Error::UnsupportedHash(hash)),
        };
        let hash =
            Multihash::wrap(hash, digest.as_slice()).expect("internal error encoding multihash");
        let cid = Cid::new_v1(codec, hash);

        // Only write the record if the CAR file does not already contain it.
        if let std::collections::hash_map::Entry::Vacant(e) = self.index.entry(cid) {
            let mut fc = vec![];
            cid.write_bytes(&mut fc).expect("internal error writing CID");

            let mut buf = unsigned_varint::encode::u64_buffer();
            let buf = unsigned_varint::encode::u64((fc.len() + contents.len()) as u64, &mut buf);

            self.storage.write_all(buf).await?;
            self.storage.write_all(&fc).await?;
            let offs = self.storage.stream_position().await?;
            self.storage.write_all(&contents).await?;

            // Update the index with the new block.
            e.insert((offs, contents.len()));
        }

        Ok(cid)
    }
}

/// Errors that can occur while interacting with a CAR.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("invalid CID: {0}")]
    Cid(#[from] ipld_core::cid::Error),
    #[error("CID does not exist in CAR")]
    CidNotFound,
    #[error("file hash does not match computed hash for block")]
    InvalidHash,
    #[error("invalid explicit CID v0")]
    InvalidCidV0,
    #[error("invalid varint: {0}")]
    InvalidVarint(#[from] unsigned_varint::io::ReadError),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("invalid Multihash: {0}")]
    Multihash(#[from] ipld_core::cid::multihash::Error),
    #[error("serde_ipld_dagcbor decoding error: {0}")]
    Parse(#[from] serde_ipld_dagcbor::DecodeError<Infallible>),
}

#[cfg(test)]
mod test {
    use std::io::Cursor;

    use crate::blockstore::DAG_CBOR;

    use super::*;

    #[tokio::test]
    async fn basic_rw() {
        const STR: &[u8] = b"the quick brown fox jumps over the lazy dog";

        let mut mem = Vec::new();
        let mut bs = CarStore::create(Cursor::new(&mut mem)).await.unwrap();

        let cid = bs.write_block(DAG_CBOR, SHA2_256, &STR).await.unwrap();
        assert_eq!(bs.read_block(&cid).await.unwrap(), STR);

        let mut bs = CarStore::open(Cursor::new(&mut mem)).await.unwrap();
        assert_eq!(bs.read_block(&cid).await.unwrap(), STR);
    }

    #[tokio::test]
    async fn basic_rw_2blocks() {
        const STR1: &[u8] = b"the quick brown fox jumps over the lazy dog";
        const STR2: &[u8] = b"the lazy fox jumps over the quick brown dog";

        let mut mem = Vec::new();
        let mut bs = CarStore::create(Cursor::new(&mut mem)).await.unwrap();

        let cid1 = bs.write_block(DAG_CBOR, SHA2_256, &STR1).await.unwrap();
        let cid2 = bs.write_block(DAG_CBOR, SHA2_256, &STR2).await.unwrap();
        assert_eq!(bs.read_block(&cid1).await.unwrap(), STR1);
        assert_eq!(bs.read_block(&cid2).await.unwrap(), STR2);

        let mut bs = CarStore::open(Cursor::new(&mut mem)).await.unwrap();
        assert_eq!(bs.read_block(&cid1).await.unwrap(), STR1);
        assert_eq!(bs.read_block(&cid2).await.unwrap(), STR2);
    }

    #[tokio::test]
    async fn basic_root() {
        const STR: &[u8] = b"the quick brown fox jumps over the lazy dog";

        let mut mem = Vec::new();
        let mut bs = CarStore::create(Cursor::new(&mut mem)).await.unwrap();

        let cid = bs.write_block(DAG_CBOR, SHA2_256, &STR).await.unwrap();
        assert_eq!(bs.read_block(&cid).await.unwrap(), STR);
        bs.set_root(cid).await.unwrap();

        let mut bs = CarStore::open(Cursor::new(&mut mem)).await.unwrap();
        assert_eq!(bs.roots().next().unwrap(), cid);
        assert_eq!(bs.read_block(&cid).await.unwrap(), STR);
    }
}
