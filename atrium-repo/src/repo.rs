use std::collections::HashSet;

use atrium_api::types::{
    string::{Did, RecordKey, Tid},
    Collection, LimitedU32,
};
use ipld_core::{cid::Cid, ipld::Ipld};
use serde::{Deserialize, Serialize};
use sha2::Digest;

use crate::{
    blockstore::{AsyncBlockStoreRead, AsyncBlockStoreWrite, DAG_CBOR, SHA2_256},
    mst,
};

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

    let data = db.read_block(cid).await?;
    let parsed: C::Record = serde_ipld_dagcbor::from_reader(&data[..])?;
    Ok(parsed)
}

/// A [Commit] builder.
///
/// Example usage:
/// ```ignore
/// let commit = Commit::new(did, root)
///     // .rev("1234abcd");
/// let hash = commit.hash();
///
/// // Sign the SHA256 digest however you wish - either using `atrium-crypto` or
/// // (preferably) using a HSM from a cloud provider.
/// let signature = todo!();
///
/// let commit: Commit = commit.sign(signature);
/// ```
pub struct CommitBuilder<'r, S: AsyncBlockStoreWrite> {
    repo: &'r mut Repository<S>,
    inner: schema::Commit,
}

impl<'r, S: AsyncBlockStoreWrite> CommitBuilder<'r, S> {
    fn new(repo: &'r mut Repository<S>, did: Did, root: Cid) -> Self {
        CommitBuilder {
            inner: schema::Commit {
                did,
                version: 3,
                data: root,
                rev: Tid::now(LimitedU32::MIN),
                prev: None,
            },
            repo,
        }
    }

    /// Set the `prev` commit field, which contains a link to the previous commit
    pub fn prev(&mut self, prev: Cid) -> &mut Self {
        self.inner.prev = Some(prev);
        self
    }

    /// Set the `rev` commit field, which is a monotonically-increasing number that can
    /// be used to signify the order in which commits are made.
    pub fn rev(&mut self, time: Tid) -> &mut Self {
        self.inner.rev = time;
        self
    }

    /// Calculate the cryptographic hash of the commit.
    pub fn hash(&self) -> [u8; 32] {
        let commit = serde_ipld_dagcbor::to_vec(&self.inner).unwrap(); // This should (hopefully!) never fail
        sha2::Sha256::digest(commit).into()
    }

    /// Cryptographically sign the commit, ensuring it can never be mutated again.
    ///
    /// We assume that the provided cryptographic hash is valid. If the signature
    /// is invalid, the commit will be rejected when published to the network!
    pub async fn sign(self, sig: Vec<u8>) -> Result<Cid, Error> {
        let s = schema::SignedCommit {
            did: self.inner.did.clone(),
            version: self.inner.version,
            data: self.inner.data,
            rev: self.inner.rev.clone(),
            prev: self.inner.prev,
            sig: Ipld::Bytes(sig),
        };
        let b = serde_ipld_dagcbor::to_vec(&s).unwrap();
        let c = self.repo.db.write_block(DAG_CBOR, SHA2_256, &b).await?;

        self.repo.root = c;
        self.repo.latest_commit = s.clone();
        Ok(c)
    }
}

/// A [Repository] builder.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RepoBuilder<S: AsyncBlockStoreRead + AsyncBlockStoreWrite> {
    db: S,
    commit: schema::Commit,
}

impl<S: AsyncBlockStoreRead + AsyncBlockStoreWrite> RepoBuilder<S> {
    /// Get the cryptographic hash of the root commit.
    pub fn hash(&self) -> [u8; 32] {
        let commit = serde_ipld_dagcbor::to_vec(&self.commit).unwrap(); // This should (hopefully!) never fail
        sha2::Sha256::digest(commit).into()
    }

    /// Cryptographically sign the root commit, finalizing the initial version of this repository.
    pub async fn sign(mut self, sig: Vec<u8>) -> Result<Repository<S>, Error> {
        // Write the commit into the database.
        let s = schema::SignedCommit {
            did: self.commit.did.clone(),
            version: self.commit.version,
            data: self.commit.data,
            rev: self.commit.rev.clone(),
            prev: self.commit.prev,
            sig: Ipld::Bytes(sig),
        };
        let b = serde_ipld_dagcbor::to_vec(&s).unwrap();
        let c = self.db.write_block(DAG_CBOR, SHA2_256, &b).await?;

        Ok(Repository { db: self.db, root: c, latest_commit: s })
    }
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

    /// Return the revision of the commit.
    pub fn rev(&self) -> Tid {
        self.inner.rev.clone()
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

        sha2::Sha256::digest(commit).into()
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
pub struct Repository<S> {
    db: S,
    root: Cid,
    latest_commit: schema::SignedCommit,
}

impl<R: AsyncBlockStoreRead> Repository<R> {
    /// Open a pre-existing instance of a user repository. This is a cheap operation
    /// that simply reads out the root commit from a repository (_without_ verifying
    /// its signature!)
    pub async fn open(mut db: R, root: Cid) -> Result<Self, Error> {
        let commit_block = db.read_block(root).await?;
        let latest_commit: schema::SignedCommit =
            serde_ipld_dagcbor::from_reader(&commit_block[..])?;

        Ok(Self { db, root, latest_commit })
    }

    /// Returns the current root cid.
    pub fn root(&self) -> Cid {
        self.root
    }

    /// Returns the latest commit in the repository.
    pub fn commit(&self) -> Commit {
        Commit { inner: self.latest_commit.clone() }
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
    pub async fn get<C: Collection>(
        &mut self,
        rkey: RecordKey,
    ) -> Result<Option<C::Record>, Error> {
        let path = C::repo_path(&rkey);
        let mut mst = mst::Tree::open(&mut self.db, self.latest_commit.data);

        if let Some(cid) = mst.get(&path).await? {
            Ok(Some(read_record::<C>(&mut self.db, cid).await?))
        } else {
            Ok(None)
        }
    }

    /// Returns the contents of the specified record from the repository, or `None` if it does not exist.
    pub async fn get_raw(&mut self, key: &str) -> Result<Option<Vec<u8>>, Error> {
        let mut mst = mst::Tree::open(&mut self.db, self.latest_commit.data);

        if let Some(cid) = mst.get(&key).await? {
            Ok(Some(self.db.read_block(cid).await?))
        } else {
            Ok(None)
        }
    }

    /// Extract the CIDs associated with a particular record.
    ///
    /// If the record does not exist in this repository, the CIDs returned will point to the
    /// node in the merkle search tree that would've contained the record.
    ///
    /// This can be used to collect the blocks needed to broadcast a record out on the firehose,
    /// for example.
    pub async fn extract<C: Collection>(
        &mut self,
        rkey: RecordKey,
    ) -> Result<impl Iterator<Item = Cid>, Error> {
        let path = C::repo_path(&rkey);
        self.extract_raw(&path).await
    }

    /// Extract the CIDs associated with a particular record into a blockstore.
    pub async fn extract_into<C: Collection>(
        &mut self,
        rkey: RecordKey,
        bs: impl AsyncBlockStoreWrite,
    ) -> Result<(), Error> {
        let path = C::repo_path(&rkey);
        self.extract_raw_into(&path, bs).await
    }

    /// Extract the CIDs associated with a particular record.
    pub async fn extract_raw(&mut self, key: &str) -> Result<impl Iterator<Item = Cid>, Error> {
        let mut mst = mst::Tree::open(&mut self.db, self.latest_commit.data);

        let mut r = vec![self.root];
        r.extend(mst.extract_path(&key).await?);
        Ok(r.into_iter())
    }

    /// Extract the CIDs associated with a particular record into a blockstore.
    pub async fn extract_raw_into(
        &mut self,
        key: &str,
        mut bs: impl AsyncBlockStoreWrite,
    ) -> Result<(), Error> {
        let cids = self.extract_raw(key).await?.collect::<HashSet<_>>();

        for cid in cids {
            bs.write_block(cid.codec(), SHA2_256, self.db.read_block(cid).await?.as_slice())
                .await?;
        }

        Ok(())
    }
}

impl<S: AsyncBlockStoreRead + AsyncBlockStoreWrite> Repository<S> {
    /// Build a new user repository.
    pub async fn create(mut db: S, did: Did) -> Result<RepoBuilder<S>, Error> {
        let tree = mst::Tree::create(&mut db).await?;
        let root = tree.root();

        Ok(RepoBuilder {
            db,
            commit: schema::Commit {
                did,
                version: 3,
                data: root,
                rev: Tid::now(LimitedU32::MIN),
                prev: None,
            },
        })
    }

    /// Add a new record to this repository.
    pub async fn add<'a, C: Collection>(
        &'a mut self,
        rkey: RecordKey,
        record: C::Record,
    ) -> Result<CommitBuilder<'a, S>, Error> {
        let path = C::repo_path(&rkey);
        self.add_raw(&path, record).await
    }

    /// Add a new raw record to this repository.
    pub async fn add_raw<'a, T: Serialize>(
        &'a mut self,
        key: &str,
        data: T,
    ) -> Result<CommitBuilder<'a, S>, Error> {
        let data = serde_ipld_dagcbor::to_vec(&data).unwrap();
        let cid = self.db.write_block(DAG_CBOR, SHA2_256, &data).await?;

        let mut mst = mst::Tree::open(&mut self.db, self.latest_commit.data);
        mst.add(&key, cid).await?;
        let root = mst.root();

        Ok(CommitBuilder::new(self, self.latest_commit.did.clone(), root))
    }

    /// Update an existing record in the repository.
    pub async fn update<'a, C: Collection>(
        &'a mut self,
        rkey: RecordKey,
        record: C::Record,
    ) -> Result<CommitBuilder<'a, S>, Error> {
        let path = C::repo_path(&rkey);
        self.update_raw(&path, record).await
    }

    /// Update an existing record in the repository with raw data.
    pub async fn update_raw<'a, T: Serialize>(
        &'a mut self,
        key: &str,
        data: T,
    ) -> Result<CommitBuilder<'a, S>, Error> {
        let data = serde_ipld_dagcbor::to_vec(&data).unwrap();
        let cid = self.db.write_block(DAG_CBOR, SHA2_256, &data).await?;

        let mut mst = mst::Tree::open(&mut self.db, self.latest_commit.data);
        mst.update(&key, cid).await?;
        let root = mst.root();

        Ok(CommitBuilder::new(self, self.latest_commit.did.clone(), root))
    }

    /// Delete an existing record in the repository.
    pub async fn delete<'a, C: Collection>(
        &'a mut self,
        rkey: RecordKey,
    ) -> Result<CommitBuilder<'a, S>, Error> {
        let path = C::repo_path(&rkey);
        self.delete_raw(&path).await
    }

    /// Delete an existing record in the repository.
    pub async fn delete_raw<'a>(&'a mut self, key: &str) -> Result<CommitBuilder<'a, S>, Error> {
        let mut mst = mst::Tree::open(&mut self.db, self.latest_commit.data);
        mst.delete(&key).await?;
        let root = mst.root();

        Ok(CommitBuilder::new(self, self.latest_commit.did.clone(), root))
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
    use std::str::FromStr;

    use crate::blockstore::{CarStore, MemoryBlockStore};
    use atrium_api::{
        app::bsky,
        types::{string::Datetime, Object},
    };
    use atrium_crypto::{
        did::parse_did_key,
        keypair::{Did as _, P256Keypair},
        verify::Verifier,
        Algorithm,
    };

    use super::*;

    /// Loads a repository from the given CAR file.
    async fn load(
        bytes: &[u8],
    ) -> Result<Repository<CarStore<std::io::Cursor<&[u8]>>>, Box<dyn std::error::Error>> {
        let db = CarStore::open(std::io::Cursor::new(bytes)).await?;
        let root = db.roots().next().unwrap();

        Repository::open(db, root).await.map_err(Into::into)
    }

    async fn create_repo<S: AsyncBlockStoreRead + AsyncBlockStoreWrite>(
        bs: S,
        did: Did,
        keypair: &P256Keypair,
    ) -> Repository<S> {
        let builder = Repository::create(bs, did).await.unwrap();

        // Sign the root commit.
        let sig = keypair.sign(&builder.hash()).unwrap();

        // Finalize the root commit and create the repository.
        builder.sign(sig).await.unwrap()
    }

    #[tokio::test]
    async fn test_commit() {
        const DATA: &[u8] = include_bytes!("../test_fixtures/commit");

        // Read out the commit record.
        let commit: Object<atrium_api::com::atproto::sync::subscribe_repos::Commit> =
            serde_ipld_dagcbor::from_reader(DATA).unwrap();

        println!("{:?}", commit.ops);

        let _repo = load(commit.blocks.as_slice()).await.unwrap();
    }

    #[tokio::test]
    async fn test_invalid_commit() {
        const DATA: &[u8] = include_bytes!("../test_fixtures/commit_invalid");

        // Read out the commit record.
        let commit: Object<atrium_api::com::atproto::sync::subscribe_repos::Commit> =
            serde_ipld_dagcbor::from_reader(DATA).unwrap();

        println!("{:?}", commit.ops);

        load(commit.blocks.as_slice()).await.unwrap_err();
    }

    #[tokio::test]
    async fn test_create_repo() {
        let mut bs = MemoryBlockStore::new();

        // Create a signing key pair.
        let keypair = P256Keypair::create(&mut rand::thread_rng());
        let dkey = keypair.did();

        // Build a new repository.
        let mut repo =
            create_repo(&mut bs, Did::new("did:web:pds.abc.com".to_string()).unwrap(), &keypair)
                .await;

        let commit = repo.commit();

        // Ensure that we can verify the root commit.
        let (_, pub_key) = parse_did_key(&dkey).unwrap();
        Verifier::default()
            .verify(Algorithm::P256, &pub_key, &commit.hash(), commit.sig())
            .unwrap();

        // Ensure the root commit points to the known empty MST CID.
        assert_eq!(
            commit.data(),
            Cid::from_str("bafyreie5737gdxlw5i64vzichcalba3z2v5n6icifvx5xytvske7mr3hpm").unwrap()
        );

        // Commit a record.
        let cb = repo
            .add::<bsky::feed::Post>(
                RecordKey::new(Tid::now(LimitedU32::MIN).to_string()).unwrap(),
                bsky::feed::post::RecordData {
                    created_at: Datetime::now(),
                    embed: None,
                    entities: None,
                    facets: None,
                    labels: None,
                    langs: None,
                    reply: None,
                    tags: None,
                    text: "Hello world".to_string(),
                }
                .into(),
            )
            .await
            .unwrap();

        let sig = keypair.sign(&cb.hash()).unwrap();
        let _cid = cb.sign(sig).await.unwrap();

        // Verify the new commit.
        let commit = repo.commit();

        Verifier::default()
            .verify(Algorithm::P256, &pub_key, &commit.hash(), commit.sig())
            .unwrap();
    }

    #[tokio::test]
    async fn test_extract() {
        let mut bs = MemoryBlockStore::new();

        // Create a signing key pair.
        let keypair = P256Keypair::create(&mut rand::thread_rng());

        // Build a new repository.
        let mut repo =
            create_repo(&mut bs, Did::new("did:web:pds.abc.com".to_string()).unwrap(), &keypair)
                .await;

        let rkey = RecordKey::new("2222222222222".to_string()).unwrap();
        let cb = repo
            .add::<bsky::feed::Post>(
                rkey.clone(),
                bsky::feed::post::RecordData {
                    created_at: Datetime::from_str("2025-02-01T00:00:00.000Z").unwrap(),
                    embed: None,
                    entities: None,
                    facets: None,
                    labels: None,
                    langs: None,
                    reply: None,
                    tags: None,
                    text: "Hello world".to_string(),
                }
                .into(),
            )
            .await
            .unwrap();

        let sig = keypair.sign(&cb.hash()).unwrap();
        let cid = cb.sign(sig).await.unwrap();

        let commit = repo.commit();

        let cids =
            repo.extract::<bsky::feed::Post>(rkey.clone()).await.unwrap().collect::<HashSet<_>>();

        assert!(cids.contains(&repo.root())); // Root commit object
        assert!(cids.contains(&commit.data())); // MST root
        assert!(cids.contains(&cid)); // Record data
    }
}
