use atrium_api::types::{
    string::{Did, Tid},
    Collection,
};
use ipld_core::{cid::Cid, ipld::Ipld};
use serde::{Deserialize, Serialize};
use sha2::Digest;

use crate::{blockstore::AsyncBlockStoreRead, mst};

mod schema {
    use super::*;

    /// Commit data
    ///
    /// Defined in: https://atproto.com/specs/repository
    ///
    /// https://github.com/bluesky-social/atproto/blob/c34426fc55e8b9f28d9b1d64eab081985d1b47b5/packages/repo/src/types.ts#L12-L19
    #[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
    pub struct Commit {
        /// the account DID associated with the repo, in strictly normalized form (eg, lowercase as appropriate)
        pub did: Did,
        /// fixed value of 3 for this repo format version
        pub version: i64,
        /// pointer to the top of the repo contents tree structure (MST)
        pub data: Cid,
        /// revision of the repo, used as a logical clock. Must increase monotonically
        pub rev: Tid,
        /// pointer (by hash) to a previous commit object for this repository
        pub prev: Option<Cid>,
    }

    /// Signed commit data. This is the exact same as a [Commit], but with a
    /// `sig` field appended.
    ///
    /// Defined in: https://atproto.com/specs/repository
    ///
    /// https://github.com/bluesky-social/atproto/blob/c34426fc55e8b9f28d9b1d64eab081985d1b47b5/packages/repo/src/types.ts#L22-L29
    #[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
    pub struct SignedCommit {
        /// the account DID associated with the repo, in strictly normalized form (eg, lowercase as appropriate)
        pub did: Did,
        /// fixed value of 3 for this repo format version
        pub version: i64,
        /// pointer to the top of the repo contents tree structure (MST)
        pub data: Cid,
        /// revision of the repo, used as a logical clock. Must increase monotonically
        pub rev: Tid,
        /// pointer (by hash) to a previous commit object for this repository
        pub prev: Option<Cid>,
        /// cryptographic signature of this commit, as raw bytes
        pub sig: Ipld,
    }
}

async fn read_record<C: Collection>(
    mut db: impl AsyncBlockStoreRead,
    cid: Cid,
) -> Result<C::Record, Error> {
    assert_eq!(cid.codec(), crate::blockstore::DAG_CBOR);

    let data = db.read_block(&cid).await?;
    let parsed: C::Record = serde_ipld_dagcbor::from_reader(&data[..])?;
    Ok(parsed)
}

/// A reference to a particular commit to a repository.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Commit {
    inner: schema::SignedCommit,
}

impl Commit {
    /// Returns a pointer to the top of the merkle search tree.
    pub fn data(&self) -> Cid {
        self.inner.data
    }

    /// Calculate the SHA-256 hash of the commit object. Used for signature verification.
    pub fn hash(&self) -> [u8; 32] {
        let commit = serde_ipld_dagcbor::to_vec(&schema::Commit {
            did: self.inner.did.clone(),
            version: self.inner.version,
            data: self.inner.data,
            rev: self.inner.rev.clone(),
            prev: self.inner.prev,
        })
        .unwrap(); // This should (hopefully!) never fail

        sha2::Sha256::digest(&commit).into()
    }

    /// Return the commit object's cryptographic signature.
    pub fn sig(&self) -> &[u8] {
        match self.inner.sig {
            Ipld::Bytes(ref bytes) => bytes,
            _ => panic!("signature field did not consist of bytes"),
        }
    }
}

/// An ATProtocol user repository.
///
/// Reference: https://atproto.com/specs/repository
#[derive(Debug)]
pub struct Repository<R: AsyncBlockStoreRead> {
    db: R,
    latest_commit: schema::SignedCommit,
}

impl<R: AsyncBlockStoreRead> Repository<R> {
    /// Open a pre-existing instance of a user repository. This is a cheap operation
    /// that simply reads out the root commit from a repository (_without_ verifying
    /// its signature!)
    pub async fn open(mut db: R, root: Cid) -> Result<Self, Error> {
        let commit_block = db.read_block(&root).await?;
        let latest_commit: schema::SignedCommit =
            serde_ipld_dagcbor::from_reader(&commit_block[..])?;

        Ok(Self { db, latest_commit })
    }

    /// Returns the latest commit in the repository.
    pub fn commit(&self) -> Commit {
        Commit { inner: self.latest_commit.clone() }
    }

    /// Returns the DID for the repository's user.
    pub fn did(&self) -> &Did {
        &self.latest_commit.did
    }

    /// Returns the specified record from the repository, or `None` if it does not exist.
    ///
    /// ---
    /// Special note: You probably noticed there's no "get record by CID" helper. This is by design.
    ///
    /// Fetching records directly via their CID is insecure because this lookup bypasses the MST
    /// (merkle search tree). Without using the MST, you cannot be sure that a particular CID was
    /// authored by the owner of the repository.
    ///
    /// If you acknowledge the risks and want to access records via CID anyway, you will have to
    /// do so by directly accessing the repository's backing storage.
    pub async fn get<C: Collection>(&mut self, rkey: &str) -> Result<Option<C::Record>, Error> {
        let mut mst = mst::Tree::open(&mut self.db, self.latest_commit.data);

        if let Some(cid) = mst.get(&rkey).await? {
            Ok(Some(read_record::<C>(&mut self.db, cid).await?))
        } else {
            Ok(None)
        }
    }
}

/// Errors that can occur while interacting with a repository.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Invalid key: {0}")]
    InvalidKey(#[from] std::str::Utf8Error),
    #[error("Invalid RecordKey: {0}")]
    InvalidRecordKey(&'static str),
    #[error("Blockstore error: {0}")]
    BlockStore(#[from] crate::blockstore::Error),
    #[error("MST error: {0}")]
    Mst(#[from] mst::Error),
    #[error("serde_ipld_dagcbor decoding error: {0}")]
    Parse(#[from] serde_ipld_dagcbor::DecodeError<std::io::Error>),
}

#[cfg(test)]
mod test {
    use crate::blockstore::CarStore;
    use atrium_api::types::Object;

    use super::*;

    /// Loads a repository from the given CAR file.
    async fn load(
        bytes: &[u8],
    ) -> Result<Repository<CarStore<std::io::Cursor<&[u8]>>>, Box<dyn std::error::Error>> {
        let db = CarStore::open(std::io::Cursor::new(bytes)).await?;
        let root = db.roots().next().unwrap();

        Repository::open(db, root).await.map_err(Into::into)
    }

    #[tokio::test]
    async fn test_commit() {
        const DATA: &[u8] = include_bytes!("../test_fixtures/commit");

        // Read out the commit record.
        let commit: Object<atrium_api::com::atproto::sync::subscribe_repos::Commit> =
            serde_ipld_dagcbor::from_reader(&DATA[..]).unwrap();

        println!("{:?}", commit.ops);

        let mut repo = load(commit.blocks.as_slice()).await.unwrap();
    }

    #[tokio::test]
    async fn test_invalid_commit() {
        const DATA: &[u8] = include_bytes!("../test_fixtures/commit_invalid");

        // Read out the commit record.
        let commit: Object<atrium_api::com::atproto::sync::subscribe_repos::Commit> =
            serde_ipld_dagcbor::from_reader(&DATA[..]).unwrap();

        println!("{:?}", commit.ops);

        load(commit.blocks.as_slice()).await.unwrap_err();
    }
}
