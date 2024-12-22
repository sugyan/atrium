use async_stream::try_stream;
use atrium_api::types::{
    string::{Did, Nsid, RecordKey},
    Collection,
};
use futures::Stream;
use ipld_core::cid::Cid;
use serde::{Deserialize, Serialize};
use tokio::io::{AsyncRead, AsyncSeek, BufReader};

use crate::{
    blockstore::AsyncBlockStoreRead,
    car::{self, IndexedReader},
    mst::{self, Located},
};

/// Commit data
///
/// Defined in: https://atproto.com/specs/repository
///
/// https://github.com/bluesky-social/atproto/blob/c34426fc55e8b9f28d9b1d64eab081985d1b47b5/packages/repo/src/types.ts#L12-L19
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
struct Commit {
    /// the account DID associated with the repo, in strictly normalized form (eg, lowercase as appropriate)
    pub did: Did,
    /// fixed value of 3 for this repo format version
    pub version: i64,
    /// pointer to the top of the repo contents tree structure (MST)
    pub data: Cid,
    /// revision of the repo, used as a logical clock. Must increase monotonically
    pub rev: String,
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
struct SignedCommit {
    /// the account DID associated with the repo, in strictly normalized form (eg, lowercase as appropriate)
    pub did: Did,
    /// fixed value of 3 for this repo format version
    pub version: i64,
    /// pointer to the top of the repo contents tree structure (MST)
    pub data: Cid,
    /// revision of the repo, used as a logical clock. Must increase monotonically
    pub rev: String,
    /// pointer (by hash) to a previous commit object for this repository
    pub prev: Option<Cid>,
    /// cryptographic signature of this commit, as raw bytes
    pub sig: Vec<u8>,
}

#[derive(Debug)]
pub struct Repository<R: AsyncBlockStoreRead> {
    db: R,
    latest_commit: Commit,
}

impl<R: AsyncBlockStoreRead> Repository<R> {
    pub async fn new(mut db: R, root: Cid) -> Result<Self, Error> {
        let commit_block = db.read_block(&root).await?;
        let latest_commit: Commit = serde_ipld_dagcbor::from_reader(&commit_block[..])?;

        Ok(Self { db, latest_commit })
    }

    /// Returns the DID for the repository's user.
    pub fn did(&self) -> &Did {
        &self.latest_commit.did
    }

    /// Parses the data with the given CID as an MST node.
    async fn read_mst_node(&mut self, cid: Cid) -> Result<mst::Node, Error> {
        let node_bytes = self.db.read_block(&cid).await?;
        let node = mst::Node::parse(&node_bytes)?;
        Ok(node)
    }

    /// Parses the data with the given CID as a record of the specified collection.
    async fn read_record<C: Collection>(&mut self, cid: Cid) -> Result<C::Record, Error> {
        let data = self.db.read_block(&cid).await?;
        let parsed: C::Record = serde_ipld_dagcbor::from_reader(&data[..])?;
        Ok(parsed)
    }

    async fn resolve_subtree<K>(
        &mut self,
        link: Cid,
        prefix: &str,
        key_fn: impl Fn(&str, Cid) -> Result<K, Error>,
        stack: &mut Vec<Located<K>>,
    ) -> Result<(), Error> {
        let node = self.read_mst_node(link).await?;

        // Read the entries from the node in reverse order; pushing each
        // entry onto the stack un-reverses their order.
        for entry in node.reversed_entries_with_prefix(prefix) {
            stack.push(match entry {
                Located::Entry((key, cid)) => Located::Entry(key_fn(key, cid)?),
                Located::InSubtree(cid) => Located::InSubtree(cid),
            });
        }

        Ok(())
    }

    async fn resolve_subtree_reversed<K>(
        &mut self,
        link: Cid,
        prefix: &str,
        key_fn: impl Fn(&str, Cid) -> Result<K, Error>,
        stack: &mut Vec<Located<K>>,
    ) -> Result<(), Error> {
        let node = self.read_mst_node(link).await?;

        // Read the entries from the node in forward order; pushing each
        // entry onto the stack reverses their order.
        for entry in node.entries_with_prefix(prefix) {
            stack.push(match entry {
                Located::Entry((key, cid)) => Located::Entry(key_fn(key, cid)?),
                Located::InSubtree(cid) => Located::InSubtree(cid),
            });
        }

        Ok(())
    }

    /// Returns a stream of all keys in this repository.
    pub fn keys<'a>(&'a mut self) -> impl Stream<Item = Result<String, Error>> + 'a {
        // Start from the root of the tree.
        let mut stack = vec![Located::InSubtree(self.latest_commit.data)];

        try_stream! {
            while let Some(located) = stack.pop() {
                match located {
                    Located::Entry(key) => yield key,
                    Located::InSubtree(link) => {
                        self.resolve_subtree(
                            link,
                            "",
                            |key, _| Ok(key.to_string()),
                            &mut stack,
                        )
                        .await?
                    }
                }
            }
        }
    }

    /// Returns a stream of the records contained in the given collection.
    pub fn get_collection<'a, C: Collection + 'a>(
        &'a mut self,
    ) -> impl Stream<Item = Result<(RecordKey, C::Record), Error>> + 'a {
        let prefix = fmt_prefix(C::nsid());

        // Start from the root of the tree.
        let mut stack = vec![Located::InSubtree(self.latest_commit.data)];

        try_stream! {
            while let Some(located) = stack.pop() {
                match located {
                    Located::Entry((rkey, cid)) => yield (rkey, self.read_record::<C>(cid).await?),
                    Located::InSubtree(link) => {
                        self.resolve_subtree(
                            link,
                            &prefix,
                            |key, cid| Ok((parse_recordkey(&key[prefix.len()..])?, cid)),
                            &mut stack,
                        )
                        .await?
                    }
                }
            }
        }
    }

    /// Returns a stream of the records contained in the given collection, in reverse
    /// order.
    pub fn get_collection_reversed<'a, C: Collection + 'a>(
        &'a mut self,
    ) -> impl Stream<Item = Result<(RecordKey, C::Record), Error>> + 'a {
        let prefix = fmt_prefix(C::nsid());

        // Start from the root of the tree.
        let mut stack = vec![Located::InSubtree(self.latest_commit.data)];

        try_stream! {
            while let Some(located) = stack.pop() {
                match located {
                    Located::Entry((rkey, cid)) => yield (rkey, self.read_record::<C>(cid).await?),
                    Located::InSubtree(link) => {
                        self.resolve_subtree_reversed(
                            link,
                            &prefix,
                            |key, cid| Ok((parse_recordkey(&key[prefix.len()..])?, cid)),
                            &mut stack,
                        )
                        .await?
                    }
                }
            }
        }
    }

    /// Returns the specified record from the repository, or `None` if it does not exist.
    pub async fn get<C: Collection>(
        &mut self,
        rkey: &RecordKey,
    ) -> Result<Option<C::Record>, Error> {
        let mut mst = mst::Tree::open(&mut self.db, self.latest_commit.data);
        let key = C::repo_path(rkey);

        if let Some(cid) = mst.get(&key).await? {
            Ok(Some(self.read_record::<C>(cid).await?))
        } else {
            Ok(None)
        }
    }
}

#[inline(always)]
fn fmt_prefix(nsid: Nsid) -> String {
    let mut prefix: String = nsid.into();
    prefix.push('/');
    prefix
}

fn parse_recordkey(key: &str) -> Result<RecordKey, Error> {
    key.parse::<RecordKey>().map_err(Error::InvalidRecordKey)
}

/// Errors that can occur while interacting with a repository.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("CAR error: {0}")]
    Car(#[from] car::Error),
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
    use std::pin::pin;

    use atrium_api::types::Object;
    use futures::StreamExt;

    use super::*;

    /// Loads a repository from the given CAR file.
    async fn load(
        bytes: &[u8],
    ) -> Result<Repository<IndexedReader<std::io::Cursor<&[u8]>>>, Error> {
        let db = IndexedReader::new(std::io::Cursor::new(bytes)).await?;
        let root = db.header().roots[0];

        Repository::new(db, root).await
    }

    #[tokio::test]
    async fn test_commit() {
        const DATA: &[u8] = include_bytes!("../test_fixtures/commit");

        // Read out the commit record.
        let commit: Object<atrium_api::com::atproto::sync::subscribe_repos::Commit> =
            serde_ipld_dagcbor::from_reader(&DATA[..]).unwrap();

        println!("{:?}", commit.ops);

        let mut repo = load(commit.blocks.as_slice()).await.unwrap();
        let keys = pin!(repo.keys()).collect::<Vec<_>>().await;

        println!("{:?}", keys);

        let record = repo
            .get::<atrium_api::app::bsky::feed::Like>(
                &RecordKey::new(commit.ops[0].path.split('/').last().unwrap().to_string()).unwrap(),
            )
            .await
            .unwrap()
            .unwrap();

        println!("{:?}", record);
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
