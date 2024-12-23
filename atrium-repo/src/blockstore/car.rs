use std::{collections::HashMap, convert::Infallible};

use futures::{AsyncReadExt as _, AsyncSeekExt as _};
use ipld_core::cid::{multihash::Multihash, Cid, Version};
use serde::Deserialize;
use sha2::Digest;
use tokio::io::{
    AsyncRead, AsyncReadExt as _, AsyncSeek, AsyncSeekExt as _, AsyncWrite, AsyncWriteExt as _,
    SeekFrom,
};
use tokio_util::compat::TokioAsyncReadCompatExt;
use unsigned_varint::io::ReadError;

use crate::blockstore::{self, AsyncBlockStoreRead, SHA2_256};

use super::AsyncBlockStoreWrite;

#[derive(Debug, Deserialize)]
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
    pub async fn new(mut storage: R) -> Result<Self, Error> {
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
                    buffer.clear();
                    buffer.resize(len as usize, 0);
                    storage.read_exact(buffer.as_mut_slice()).await?;

                    let digest = sha2::Sha256::digest(buffer.as_slice());
                    let expected = Multihash::wrap(cid.hash().code(), digest.as_slice())
                        .map_err(Error::Multihash)?;
                    let expected = Cid::new_v1(cid.codec(), expected);

                    if expected != cid {
                        return Err(Error::InvalidHash);
                    }

                    index.insert(cid, (offset, len as usize));
                }
                Err(ReadError::Io(e)) if e.kind() == std::io::ErrorKind::UnexpectedEof => break,
                Err(e) => Err(e)?,
            }
        }

        Ok(Self { storage, header, index })
    }

    pub fn header(&self) -> &V1Header {
        &self.header
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
    async fn write_block(&mut self, codec: u64, contents: &[u8]) -> Result<Cid, blockstore::Error> {
        let digest = sha2::Sha256::digest(contents);
        let hash = Multihash::wrap(SHA2_256, digest.as_slice())
            .expect("internal error encoding multihash");
        let cid = Cid::new_v1(codec, hash);

        let mut fc = vec![];
        cid.write_bytes(&mut fc).expect("internal error writing CID");
        fc.extend_from_slice(contents);

        let mut buf = unsigned_varint::encode::u64_buffer();
        unsigned_varint::encode::u64(fc.len() as u64, &mut buf);

        self.storage.write_all(&buf).await?;
        let offs = self.storage.stream_position().await?;
        self.storage.write_all(&fc).await?;

        // Update the index with the new block.
        self.index.insert(cid, (offs, contents.len()));

        Ok(cid)
    }

    async fn delete_block(&mut self, _cid: &Cid) -> Result<(), blockstore::Error> {
        todo!()
    }
}

/// Errors that can occur while interacting with a CAR.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Invalid CID: {0}")]
    Cid(#[from] ipld_core::cid::Error),
    #[error("CID does not exist in CAR")]
    CidNotFound,
    #[error("Invalid hash")]
    InvalidHash,
    #[error("Invalid explicit CID v0")]
    InvalidCidV0,
    #[error("Invalid varint: {0}")]
    InvalidVarint(#[from] unsigned_varint::io::ReadError),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Invalid Multihash: {0}")]
    Multihash(#[from] ipld_core::cid::multihash::Error),
    #[error("serde_ipld_dagcbor decoding error: {0}")]
    Parse(#[from] serde_ipld_dagcbor::DecodeError<Infallible>),
}
