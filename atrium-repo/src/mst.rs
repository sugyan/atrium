use std::convert::Infallible;

use ipld_core::{cid::Cid, ipld::Ipld};
use serde::Deserialize;

/// The location of an entry in a Merkle Search Tree.
#[derive(Debug)]
pub enum Located<E> {
    /// The tree entry corresponding to a key.
    Entry(E),
    /// The CID of the [`Node`] containing the sub-tree in which a key is located.
    InSubtree(Cid),
}

/// A node in a Merkle Search Tree.
#[derive(Debug)]
pub struct Node {
    left: Option<Cid>,
    entries: Vec<TreeEntry>,
}

impl Node {
    /// Parses an MST node from its DAG-CBOR encoding.
    pub fn parse(bytes: &[u8]) -> Result<Self, Error> {
        let node: NodeSchema = serde_ipld_dagcbor::from_slice(bytes)?;

        let entries = node
            .entries
            .into_iter()
            .scan(vec![], |prev_key, entry| {
                match TreeEntry::parse(entry, &prev_key) {
                    Ok(entry) => {
                        prev_key.clear();
                        prev_key.extend_from_slice(&entry.key);
                        Some(Ok(entry))
                    }
                    Err(e) => Some(Err(e)),
                }
            })
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Self {
            left: node.left,
            entries,
        })
    }

    /// Finds the location of the given key's value within this sub-tree.
    ///
    /// Returns `None` if the key does not exist within this sub-tree.
    pub fn get(&self, key: &[u8]) -> Option<Located<Cid>> {
        match self
            .entries
            .iter()
            .rev()
            .find(|entry| &entry.key[..] <= key)
        {
            Some(entry) => {
                if &entry.key[..] == key {
                    Some(Located::Entry(entry.value))
                } else {
                    entry.tree.map(Located::InSubtree)
                }
            }
            None => self.left.map(Located::InSubtree),
        }
    }

    /// Returns the locations of values for all keys within this sub-tree with the given
    /// prefix.
    pub fn entries_with_prefix<'a>(
        &'a self,
        prefix: &'a [u8],
    ) -> impl Iterator<Item = Located<(&[u8], Cid)>> + 'a {
        self.entries
            .first()
            .and_then(|entry| {
                if entry.key.starts_with(prefix) || prefix < &entry.key[..] {
                    self.left.map(Located::InSubtree)
                } else {
                    None
                }
            })
            .into_iter()
            .chain(self.entries.iter().flat_map(move |entry| {
                if entry.key.starts_with(prefix) {
                    [
                        Some(Located::Entry((&entry.key[..], entry.value))),
                        entry.tree.map(Located::InSubtree),
                    ]
                } else if prefix > &entry.key[..] {
                    [None, entry.tree.map(Located::InSubtree)]
                } else {
                    [None, None]
                }
                .into_iter()
                .flatten()
            }))
    }

    /// Returns the locations of values for all keys within this sub-tree with the given
    /// prefix, in reverse order.
    pub fn reversed_entries_with_prefix<'a>(
        &'a self,
        prefix: &'a [u8],
    ) -> impl Iterator<Item = Located<(&[u8], Cid)>> + 'a {
        self.entries
            .iter()
            .rev()
            .flat_map(move |entry| {
                if entry.key.starts_with(prefix) {
                    [
                        entry.tree.map(Located::InSubtree),
                        Some(Located::Entry((&entry.key[..], entry.value))),
                    ]
                } else if prefix > &entry.key[..] {
                    [entry.tree.map(Located::InSubtree), None]
                } else {
                    [None, None]
                }
                .into_iter()
                .flatten()
            })
            .chain(self.entries.first().and_then(|entry| {
                if entry.key.starts_with(prefix) || prefix < &entry.key[..] {
                    self.left.map(Located::InSubtree)
                } else {
                    None
                }
            }))
    }
}

#[derive(Debug)]
struct TreeEntry {
    key: Vec<u8>,
    value: Cid,
    tree: Option<Cid>,
}

impl TreeEntry {
    fn parse(entry: TreeEntrySchema, prev_key: &[u8]) -> Result<Self, Error> {
        let mut key_suffix = match entry.key_suffix {
            Ipld::Bytes(k) => Ok(k.clone()),
            _ => Err(Error::KeySuffixNotBytes),
        }?;

        let key = if entry.prefix_len == 0 {
            key_suffix
        } else if prev_key.len() < entry.prefix_len {
            return Err(Error::InvalidPrefixLen);
        } else {
            let mut key_bytes = prev_key[..entry.prefix_len].to_vec();
            key_bytes.append(&mut key_suffix);
            key_bytes
        };

        Ok(Self {
            key,
            value: entry.value,
            tree: entry.tree,
        })
    }
}

/// The [IPLD schema] for an MST node.
///
/// [IPLD schema]: https://atproto.com/specs/repository#mst-structure
#[derive(Deserialize)]
struct NodeSchema {
    /// ("left", CID link, nullable): link to sub-tree [`Node`] on a lower level and with
    /// all keys sorting before keys at this node.
    #[serde(rename = "l")]
    left: Option<Cid>,

    /// ("entries", array of objects, required): ordered list of [`TreeEntry`] objects.
    #[serde(rename = "e")]
    entries: Vec<TreeEntrySchema>,
}

#[derive(Deserialize)]
struct TreeEntrySchema {
    /// ("prefixlen", integer, required): count of bytes shared with previous [`TreeEntry`]
    /// in this [`Node`] (if any).
    #[serde(rename = "p")]
    prefix_len: usize,

    /// ("keysuffix", byte array, required): remainder of key for this [`TreeEntry`],
    /// after "prefixlen" have been removed.
    ///
    /// We deserialize this with the [`Ipld`] type instead of directly as a `Vec<u8>`,
    /// because serde maps the latter to CBOR Major Type 4 (array of data items) instead
    /// of Major Type 2 (byte string). Other crates exist that provide bytes-specific
    /// deserializers, but `Ipld` is already in our dependencies.
    #[serde(rename = "k")]
    key_suffix: Ipld,

    /// ("value", CID Link, required): link to the record data (CBOR) for this entry.
    #[serde(rename = "v")]
    value: Cid,

    /// ("tree", CID Link, nullable): link to a sub-tree [`Node`] at a lower level which
    /// has keys sorting after this [`TreeEntry`]'s key (to the "right"), but before the
    /// next [`TreeEntry`]'s key in this [`Node`] (if any).
    #[serde(rename = "t")]
    tree: Option<Cid>,
}

/// Errors that can occur while interacting with an MST.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Invalid prefix_len")]
    InvalidPrefixLen,
    #[error("key_suffix not a byte string")]
    KeySuffixNotBytes,
    #[error("serde_ipld_dagcbor decoding error: {0}")]
    Parse(#[from] serde_ipld_dagcbor::DecodeError<Infallible>),
}
