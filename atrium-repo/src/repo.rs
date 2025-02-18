use std::collections::HashSet;

use atrium_api::types::{
    string::{Did, RecordKey, Tid},
    Collection, LimitedU32,
};
use futures::TryStreamExt;
use ipld_core::cid::Cid;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
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
        #[serde(with = "serde_bytes")]
        pub sig: Vec<u8>,
    }
}

async fn read_record<T: DeserializeOwned>(
    mut db: impl AsyncBlockStoreRead,
    cid: Cid,
) -> Result<T, Error> {
    assert_eq!(cid.codec(), crate::blockstore::DAG_CBOR);

    let data = db.read_block(cid).await?;
    let parsed: T = serde_ipld_dagcbor::from_reader(&data[..])?;
    Ok(parsed)
}

/// A [Commit] builder.
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
    pub async fn finalize(self, sig: Vec<u8>) -> Result<Cid, Error> {
        let s = schema::SignedCommit {
            did: self.inner.did.clone(),
            version: self.inner.version,
            data: self.inner.data,
            rev: self.inner.rev.clone(),
            prev: self.inner.prev,
            sig,
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
    pub async fn finalize(mut self, sig: Vec<u8>) -> Result<Repository<S>, Error> {
        // Write the commit into the database.
        let s = schema::SignedCommit {
            did: self.commit.did.clone(),
            version: self.commit.version,
            data: self.commit.data,
            rev: self.commit.rev.clone(),
            prev: self.commit.prev,
            sig,
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
        self.inner.sig.as_slice()
    }
}

/// An ATProtocol user repository.
///
/// This is a convenience data structure that is cheap to construct and intended
/// to be used in an on-demand manner rather than as a long-lived data structure.
///
/// For example, to open an existing repository and query a single record:
/// ```no_run
/// # use atrium_api::{app::bsky, types::string::RecordKey};
/// # use atrium_repo::{blockstore::MemoryBlockStore, Cid, Repository};
/// # let mut bs = MemoryBlockStore::new();
/// # let root = Cid::default();
/// # let rkey = RecordKey::new("2222222222222".to_string()).unwrap();
/// #
/// # async move {
/// // N.B: This will not verify the contents of the repository, so this should only
/// // be used with data from a trusted source.
/// let mut repo = Repository::open(&mut bs, root).await.unwrap();
/// let post = repo.get::<bsky::feed::Post>(rkey).await.unwrap();
///
/// drop(repo); // We're done using the repository at this point.
/// # };
/// ```
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

    /// Open the merkle search tree for the latest commit.
    ///
    /// This API is for advanced usage. Typically you will want to use the convenience
    /// APIs offered by this struct instead. Any modifications to the tree will _not_
    /// automatically be reflected by this `Repository`.
    pub fn tree(&mut self) -> mst::Tree<&mut R> {
        mst::Tree::open(&mut self.db, self.latest_commit.data)
    }

    /// Returns the specified record from the repository, or `None` if it does not exist.
    pub async fn get<C: Collection>(
        &mut self,
        rkey: RecordKey,
    ) -> Result<Option<C::Record>, Error> {
        let path = C::repo_path(&rkey);
        let mut mst = mst::Tree::open(&mut self.db, self.latest_commit.data);

        if let Some(cid) = mst.get(&path).await? {
            Ok(Some(read_record::<C::Record>(&mut self.db, cid).await?))
        } else {
            Ok(None)
        }
    }

    /// Returns the contents of the specified record from the repository, or `None` if it does not exist.
    pub async fn get_raw<T: DeserializeOwned>(&mut self, key: &str) -> Result<Option<T>, Error> {
        let mut mst = mst::Tree::open(&mut self.db, self.latest_commit.data);

        if let Some(cid) = mst.get(key).await? {
            Ok(Some(read_record::<T>(&mut self.db, cid).await?))
        } else {
            Ok(None)
        }
    }

    /// Returns the contents of a specified record from the repository, or `None` if it does not exist.
    ///
    /// Caution: This is a potentially expensive operation that will iterate through the entire MST.
    /// This is done for security reasons; a particular CID cannot be proven to originate from this
    /// repository if it is not present in the MST.
    ///
    /// Typically if you have a record's CID, you should also have its key (e.g. from a firehose commit).
    /// If you have the key, you should **always prefer to use [`Repository::get_raw`]** as it is both
    /// much faster and secure.
    ///
    /// If you're absolutely certain you want to look up a record by its CID and the repository comes
    /// from a trusted source, you can elide the enumeration by accessing the backing storage directly.
    pub async fn get_raw_cid<T: DeserializeOwned>(&mut self, cid: Cid) -> Result<Option<T>, Error> {
        let mut mst = mst::Tree::open(&mut self.db, self.latest_commit.data);

        let mut ocid = None;

        let mut it = Box::pin(mst.entries());
        while let Some((_rkey, rcid)) = it.try_next().await? {
            if rcid == cid {
                ocid = Some(rcid);
                break;
            }
        }

        // Drop the iterator so that we can access `self.db`.
        drop(it);

        if let Some(ocid) = ocid {
            Ok(Some(read_record::<T>(&mut self.db, ocid).await?))
        } else {
            Ok(None)
        }
    }

    /// Export a list of all CIDs in the repository.
    pub async fn export(&mut self) -> Result<impl Iterator<Item = Cid>, Error> {
        let mut mst = mst::Tree::open(&mut self.db, self.latest_commit.data);

        let mut r = vec![self.root];
        r.extend(mst.export().try_collect::<Vec<_>>().await?);
        Ok(r.into_iter())
    }

    /// Export all CIDs in the repository into a blockstore.
    pub async fn export_into(&mut self, mut bs: impl AsyncBlockStoreWrite) -> Result<(), Error> {
        let cids = self.export().await?.collect::<HashSet<_>>();

        for cid in cids {
            bs.write_block(cid.codec(), SHA2_256, self.db.read_block(cid).await?.as_slice())
                .await?;
        }

        Ok(())
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
        r.extend(mst.extract_path(key).await?);
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
    pub async fn add<C: Collection>(
        &mut self,
        rkey: RecordKey,
        record: C::Record,
    ) -> Result<(CommitBuilder<'_, S>, Cid), Error> {
        let path = C::repo_path(&rkey);
        self.add_raw(&path, record).await
    }

    /// Add a new raw record to this repository.
    pub async fn add_raw<'a, T: Serialize>(
        &'a mut self,
        key: &str,
        data: T,
    ) -> Result<(CommitBuilder<'a, S>, Cid), Error> {
        let data = serde_ipld_dagcbor::to_vec(&data).unwrap();
        let cid = self.db.write_block(DAG_CBOR, SHA2_256, &data).await?;

        let mut mst = mst::Tree::open(&mut self.db, self.latest_commit.data);
        mst.add(key, cid).await?;
        let root = mst.root();

        Ok((CommitBuilder::new(self, self.latest_commit.did.clone(), root), cid))
    }

    /// Update an existing record in the repository.
    pub async fn update<C: Collection>(
        &mut self,
        rkey: RecordKey,
        record: C::Record,
    ) -> Result<(CommitBuilder<'_, S>, Cid), Error> {
        let path = C::repo_path(&rkey);
        self.update_raw(&path, record).await
    }

    /// Update an existing record in the repository with raw data.
    pub async fn update_raw<'a, T: Serialize>(
        &'a mut self,
        key: &str,
        data: T,
    ) -> Result<(CommitBuilder<'a, S>, Cid), Error> {
        let data = serde_ipld_dagcbor::to_vec(&data).unwrap();
        let cid = self.db.write_block(DAG_CBOR, SHA2_256, &data).await?;

        let mut mst = mst::Tree::open(&mut self.db, self.latest_commit.data);
        mst.update(key, cid).await?;
        let root = mst.root();

        Ok((CommitBuilder::new(self, self.latest_commit.did.clone(), root), cid))
    }

    /// Delete an existing record in the repository.
    pub async fn delete<C: Collection>(
        &mut self,
        rkey: RecordKey,
    ) -> Result<CommitBuilder<'_, S>, Error> {
        let path = C::repo_path(&rkey);
        self.delete_raw(&path).await
    }

    /// Delete an existing record in the repository.
    pub async fn delete_raw<'a>(&'a mut self, key: &str) -> Result<CommitBuilder<'a, S>, Error> {
        let mut mst = mst::Tree::open(&mut self.db, self.latest_commit.data);
        mst.delete(key).await?;
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

    use crate::blockstore::MemoryBlockStore;
    use atrium_api::{app::bsky, types::string::Datetime};
    use atrium_crypto::{
        did::parse_did_key,
        keypair::{Did as _, P256Keypair},
        verify::Verifier,
        Algorithm,
    };

    use super::*;

    async fn create_repo<S: AsyncBlockStoreRead + AsyncBlockStoreWrite>(
        bs: S,
        did: Did,
        keypair: &P256Keypair,
    ) -> Repository<S> {
        let builder = Repository::create(bs, did).await.unwrap();

        // Sign the root commit.
        let sig = keypair.sign(&builder.hash()).unwrap();

        // Finalize the root commit and create the repository.
        builder.finalize(sig).await.unwrap()
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
        let (cb, _) = repo
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
        let _cid = cb.finalize(sig).await.unwrap();

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
        let (cb, _) = repo
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
        let cid = cb.finalize(sig).await.unwrap();

        let commit = repo.commit();

        let mut bs2 = MemoryBlockStore::new();
        repo.extract_into::<bsky::feed::Post>(rkey.clone(), &mut bs2).await.unwrap();

        assert!(bs2.read_block(cid).await.is_ok()); // Root commit object
        assert!(bs2.read_block(commit.data()).await.is_ok()); // MST root

        // Ensure the record can be fetched via a record lookup.
        let mut repo2 = Repository::open(&mut bs2, repo.root()).await.unwrap();
        assert!(repo2.get::<bsky::feed::Post>(rkey.clone()).await.is_ok());

        let cb = repo.delete::<bsky::feed::Post>(rkey.clone()).await.unwrap();
        let sig = keypair.sign(&cb.hash()).unwrap();
        let cid = cb.finalize(sig).await.unwrap();

        // Extract won't fail even if the actual record does not exist.
        // In this case, we are extracting a proof that the record does _not_ exist.
        let cids =
            repo.extract::<bsky::feed::Post>(rkey.clone()).await.unwrap().collect::<HashSet<_>>();

        assert!(cids.contains(&cid)); // Root commit object
        assert!(cids.contains(
            // MST root (known empty hash)
            &Cid::from_str("bafyreie5737gdxlw5i64vzichcalba3z2v5n6icifvx5xytvske7mr3hpm").unwrap()
        ))
    }

    #[tokio::test]
    async fn test_extract_complex() {
        let mut bs = MemoryBlockStore::new();

        // Create a signing key pair.
        let keypair = P256Keypair::create(&mut rand::thread_rng());

        // Build a new repository.
        let mut repo =
            create_repo(&mut bs, Did::new("did:web:pds.abc.com".to_string()).unwrap(), &keypair)
                .await;

        let mut records = Vec::new();

        for i in 0..10 {
            // Ensure we don't generate the same record key twice.
            let rkey = loop {
                let rkey = RecordKey::new(Tid::now(LimitedU32::MIN).to_string()).unwrap();
                if !records.contains(&rkey) {
                    break rkey;
                }
            };

            let (cb, _) = repo
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
                        text: format!("Hello world, post {i}"),
                    }
                    .into(),
                )
                .await
                .unwrap();

            let sig = keypair.sign(&cb.hash()).unwrap();
            cb.finalize(sig).await.unwrap();

            records.push(rkey.clone());
        }

        for record in records {
            let mut bs2 = MemoryBlockStore::new();
            repo.extract_into::<bsky::feed::Post>(record.clone(), &mut bs2).await.unwrap();

            assert!(bs2.contains(repo.root()));
            assert!(bs2.contains(repo.commit().data()));

            let mut repo2 = Repository::open(&mut bs2, repo.root()).await.unwrap();
            assert!(repo2.get::<bsky::feed::Post>(record.clone()).await.is_ok());
        }
    }
}
