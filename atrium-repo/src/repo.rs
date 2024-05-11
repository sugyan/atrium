use async_stream::try_stream;
use atrium_api::types::{
    string::{Did, Nsid, RecordKey},
    Collection,
};
use futures::Stream;
use ipld_core::cid::Cid;
use serde::Deserialize;
use tokio::io::{AsyncRead, AsyncSeek, BufReader};

use crate::{
    car::{self, IndexedReader},
    mst::{self, Located},
};

#[derive(Debug, Deserialize)]
struct Commit {
    did: Did,
    version: u32,
    data: Cid,
    rev: String,
    prev: Option<Cid>,
}

#[derive(Debug)]
pub struct Repository<R: AsyncRead + AsyncSeek> {
    db: IndexedReader<BufReader<R>>,
    latest_commit: Commit,
}

impl<R: AsyncRead + AsyncSeek + Unpin + Send> Repository<R> {
    /// Loads a repository from the given reader.
    pub async fn load(reader: R) -> Result<Self, Error> {
        let mut db = IndexedReader::new(BufReader::new(reader)).await?;
        let root = db.header().roots[0];

        let commit_block = db.get_block(&root).await?;
        let latest_commit: Commit = serde_ipld_dagcbor::from_reader(&commit_block[..])?;

        Ok(Self { db, latest_commit })
    }

    /// Returns the DID for the repository's user.
    pub fn did(&self) -> &Did {
        &self.latest_commit.did
    }

    /// Parses the data with the given CID as an MST node.
    async fn read_mst_node(&mut self, cid: Cid) -> Result<mst::Node, Error> {
        let node_bytes = self.db.get_block(&cid).await?;
        let node = mst::Node::parse(&node_bytes)?;
        Ok(node)
    }

    /// Parses the data with the given CID as a record of the specified collection.
    async fn read_record<C: Collection>(&mut self, cid: Cid) -> Result<C::Record, Error> {
        let data = self.db.get_block(&cid).await?;
        let parsed: C::Record = serde_ipld_dagcbor::from_reader(&data[..])?;
        Ok(parsed)
    }

    async fn resolve_subtree<K>(
        &mut self,
        link: Cid,
        prefix: &[u8],
        key_fn: impl Fn(&[u8], Cid) -> Result<K, Error>,
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
        prefix: &[u8],
        key_fn: impl Fn(&[u8], Cid) -> Result<K, Error>,
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
                            &[],
                            |key, _| Ok(std::str::from_utf8(key)?.to_string()),
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
                            prefix.as_bytes(),
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
                            prefix.as_bytes(),
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
        let key = C::repo_path(rkey);

        // Start from the root of the tree.
        let mut link = self.latest_commit.data;

        loop {
            let node = self.read_mst_node(link).await?;
            match node.get(key.as_bytes()) {
                None => return Ok(None),
                Some(Located::Entry(cid)) => return Ok(Some(self.read_record::<C>(cid).await?)),
                Some(Located::InSubtree(cid)) => link = cid,
            }
        }
    }
}

#[inline(always)]
fn fmt_prefix(nsid: Nsid) -> String {
    let mut prefix: String = nsid.into();
    prefix.push('/');
    prefix
}

fn parse_recordkey(key: &[u8]) -> Result<RecordKey, Error> {
    std::str::from_utf8(key)?
        .parse::<RecordKey>()
        .map_err(Error::InvalidRecordKey)
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
    #[error("MST error: {0}")]
    Mst(#[from] mst::Error),
    #[error("serde_ipld_dagcbor decoding error: {0}")]
    Parse(#[from] serde_ipld_dagcbor::DecodeError<std::io::Error>),
}
