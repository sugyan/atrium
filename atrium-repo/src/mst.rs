use std::{cmp::Ordering, convert::Infallible, pin::pin, string::FromUtf8Error};

use async_stream::try_stream;
use futures::{Stream, StreamExt};
use ipld_core::{cid::Cid, ipld::Ipld};
use serde::{Deserialize, Serialize};
use sha2::Digest;

use crate::blockstore::{AsyncBlockStoreRead, AsyncBlockStoreWrite, DAG_CBOR};

mod schema {
    use super::*;

    /// The [IPLD schema] for an MST node.
    ///
    /// [IPLD schema]: https://atproto.com/specs/repository#mst-structure
    #[derive(Deserialize, Serialize, Clone, PartialEq)]
    pub struct Node {
        /// ("left", CID link, nullable): link to sub-tree [`Node`] on a lower level and with
        /// all keys sorting before keys at this node.
        #[serde(rename = "l")]
        pub left: Option<Cid>,

        /// ("entries", array of objects, required): ordered list of [`TreeEntry`] objects.
        #[serde(rename = "e")]
        pub entries: Vec<TreeEntry>,
    }

    #[derive(Deserialize, Serialize, Clone, PartialEq)]
    pub struct TreeEntry {
        /// ("prefixlen", integer, required): count of bytes shared with previous [`TreeEntry`]
        /// in this [`Node`] (if any).
        #[serde(rename = "p")]
        pub prefix_len: usize,

        /// ("keysuffix", byte array, required): remainder of key for this [`TreeEntry`],
        /// after "prefixlen" have been removed.
        ///
        /// We deserialize this with the [`Ipld`] type instead of directly as a `Vec<u8>`,
        /// because serde maps the latter to CBOR Major Type 4 (array of data items) instead
        /// of Major Type 2 (byte string). Other crates exist that provide bytes-specific
        /// deserializers, but `Ipld` is already in our dependencies.
        #[serde(rename = "k")]
        pub key_suffix: Ipld,

        /// ("value", CID Link, required): link to the record data (CBOR) for this entry.
        #[serde(rename = "v")]
        pub value: Cid,

        /// ("tree", CID Link, nullable): link to a sub-tree [`Node`] at a lower level which
        /// has keys sorting after this [`TreeEntry`]'s key (to the "right"), but before the
        /// next [`TreeEntry`]'s key in this [`Node`] (if any).
        #[serde(rename = "t")]
        pub tree: Option<Cid>,
    }
}

// https://users.rust-lang.org/t/how-to-find-common-prefix-of-two-byte-slices-effectively/25815/3
fn prefix(xs: &[u8], ys: &[u8]) -> usize {
    prefix_chunks::<128>(xs, ys)
}

fn prefix_chunks<const N: usize>(xs: &[u8], ys: &[u8]) -> usize {
    let off =
        std::iter::zip(xs.chunks_exact(N), ys.chunks_exact(N)).take_while(|(x, y)| x == y).count()
            * N;
    off + std::iter::zip(&xs[off..], &ys[off..]).take_while(|(x, y)| x == y).count()
}

/// Calculate the number of leading zeroes from the sha256 hash of a byte array
fn leading_zeroes(key: &[u8]) -> usize {
    let digest = sha2::Sha256::digest(key);
    let mut zeroes = 0;

    for byte in digest.iter() {
        zeroes += byte.leading_zeros() as usize;

        // If the byte is nonzero, then there cannot be any more leading zeroes.
        if *byte != 0 {
            break;
        }
    }

    zeroes
}

/// A merkle search tree data structure, backed by storage implementing
/// [AsyncBlockStoreRead] and [AsyncBlockStoreWrite].
///
/// There are two factors that determine the placement of nodes inside of
/// a merkle search tree:
/// - The number of leading zeroes in the SHA256 hash of the key
/// - The key's lexicographic position inside of a layer
///
/// Useful reading: https://interjectedfuture.com/crdts-turned-inside-out/
pub struct Tree<S> {
    storage: S,
    root: Cid,
}

impl<S: AsyncBlockStoreRead + AsyncBlockStoreWrite> Tree<S> {
    /// Create a new MST with an empty root node
    pub async fn create(mut storage: S) -> Result<Self, Error> {
        let node =
            serde_ipld_dagcbor::to_vec(&schema::Node { left: None, entries: vec![] }).unwrap();
        let cid = storage.write_block(DAG_CBOR, &node).await.unwrap();

        Ok(Self { storage, root: cid })
    }

    pub async fn add(&mut self, key: &str, value: Cid) -> Result<(), Error> {
        // Compute the layer where this note should be added.
        let target_layer = leading_zeroes(key.as_bytes());

        // Now traverse to the node containing the target layer.
        let mut node_path = vec![];
        let mut node_cid = self.root.clone();

        let mut node = loop {
            let mut node = self.read_node(node_cid).await?;

            // Determine whether or not the desired node belongs in this layer.
            if let Some(layer) = node.layer() {
                match layer.cmp(&target_layer) {
                    Ordering::Equal => break node,
                    // The entire tree needs to be shifted downward.
                    Ordering::Less => {
                        // This should only happen for the root node.
                        assert_eq!(self.root, node_cid);
                        break Node { entries: vec![NodeEntry::Tree(node_cid)] };
                    }
                    // Search in a subtree.
                    Ordering::Greater => {
                        let partition = node.find_ge(key).unwrap();

                        // If left neighbor is a subtree, recurse through.
                        if let Some(subtree) = node.entries.get(partition - 1).unwrap().tree() {
                            node_path.push((node_cid, partition - 1, node.clone()));
                            node_cid = subtree.clone();
                        } else {
                            // N.B: The `node_cid` in the tree entry is a placeholder.
                            node.entries.insert(partition, NodeEntry::Tree(node_cid));
                            node_path.push((node_cid, partition, node.clone()));

                            // We need to insert a new subtree.
                            break Node { entries: vec![] };
                        }
                    }
                }
            } else if node_cid == self.root {
                // The node is an empty root node.
                break node;
            } else {
                // This can happen if we encounter an empty intermediate node.
                todo!()
            }
        };

        if let Some(partition) = node.find_ge(key) {
            // Check if the key is already present in the node.
            if let Some(NodeEntry::Leaf(e)) = node.entries.get(partition) {
                if e.key == key {
                    return Err(Error::KeyAlreadyExists);
                }
            }

            match node.entries.get(partition - 1) {
                Some(NodeEntry::Leaf(_)) => {
                    // Left neighbor is a leaf, so we can simply insert this leaf to its right.
                    node.entries.insert(
                        partition,
                        NodeEntry::Leaf(TreeEntry { key: key.to_string(), value }),
                    );
                }
                Some(NodeEntry::Tree(e)) => {
                    // Need to split the subtree into two based on the node's key.
                    let (left, right) = self.split_tree(e.clone(), key).await?;

                    // Insert the new node inbetween the two subtrees.
                    let right_subvec = node.entries.split_off(partition);

                    node.entries.pop();
                    node.entries.extend([
                        NodeEntry::Tree(left),
                        NodeEntry::Leaf(TreeEntry { key: key.to_string(), value }),
                    ]);
                    if let Some(right) = right {
                        node.entries.push(NodeEntry::Tree(right));
                    }
                    node.entries.extend(right_subvec.into_iter());
                }
                None => todo!(),
            }
        } else {
            // The node is empty! Just append the new key to this node's entries.
            node.entries.push(NodeEntry::Leaf(TreeEntry { key: key.to_string(), value }));
        }

        let mut cid = node.serialize_into(&mut self.storage).await?;

        // Now walk back up the node path chain and update parent entries to point to the new node's CID.
        for (_parent_cid, i, mut parent) in node_path.into_iter().rev() {
            parent.entries[i] = NodeEntry::Tree(cid);
            cid = parent.serialize_into(&mut self.storage).await?;
        }

        self.root = cid;
        Ok(())
    }

    pub async fn update(&mut self, key: &str, value: Cid) -> Result<(), Error> {
        todo!()
    }

    pub async fn delete(&mut self, key: &str) -> Result<(), Error> {
        todo!()
    }

    /// Recursively split a node based on a key.
    ///
    /// If the key is found within the subtree, this will return an error.
    async fn split_tree(&mut self, node: Cid, key: &str) -> Result<(Cid, Option<Cid>), Error> {
        let mut node_path = vec![];
        let mut node_cid = node;

        let (left, right) = loop {
            let mut node = self.read_node(node_cid).await?;

            if let Some(partition) = node.find_ge(key) {
                // Ensure that the key does not already exist.
                if let Some(NodeEntry::Leaf(e)) = node.entries.get(partition) {
                    if e.key == key {
                        return Err(Error::KeyAlreadyExists);
                    }
                }

                // Determine if the left neighbor is a subtree. If so, we need to recursively split that tree.
                match node.entries.get(partition - 1) {
                    Some(NodeEntry::Leaf(_e)) => {
                        // Left neighbor is a leaf, so we can split the current node into two and we are done.
                        let right = node.entries.split_off(partition);

                        break (
                            node,
                            if !right.is_empty() { Some(Node { entries: right }) } else { None },
                        );
                    }
                    Some(NodeEntry::Tree(e)) => {
                        node_path.push((node_cid, partition, node.clone()));
                        node_cid = e.clone();
                    }
                    // This should not happen; node.find_ge() should return `None` in this case.
                    None => panic!(),
                }
            } else {
                // The node is empty.
                todo!()
            }
        };

        // Now walk back up the path chain and split the parent entries.
        for (parent_cid, i, parent) in node_path.into_iter().rev() {
            todo!()
        }

        // Serialize the two new subtrees.
        let left = left.serialize_into(&mut self.storage).await?;
        let right = if let Some(right) = right {
            Some(right.serialize_into(&mut self.storage).await?)
        } else {
            None
        };

        Ok((left, right))
    }
}

impl<S: AsyncBlockStoreRead> Tree<S> {
    pub fn load(storage: S, root: Cid) -> Self {
        Self { storage, root }
    }

    /// Parses the data with the given CID as an MST node.
    async fn read_node(&mut self, cid: Cid) -> Result<Node, Error> {
        let node_bytes = self.storage.read_block(&cid).await?;
        let node = Node::parse(&node_bytes)?;
        Ok(node)
    }

    async fn resolve_subtree<K>(
        &mut self,
        link: Cid,
        prefix: &str,
        key_fn: impl Fn(&str, Cid) -> Result<K, Error>,
        stack: &mut Vec<Located<K>>,
    ) -> Result<(), Error> {
        let node = self.read_node(link).await?;

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
        let node = self.read_node(link).await?;

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

    /// Compute the depth of the merkle search tree
    pub async fn depth(&mut self) -> Result<usize, Error> {
        // The root of the tree should be the highest layer, so we can simply
        // count the leading number of zeroes of the sha256 hash of a key in the
        // root layer.
        let mut keys = pin!(self.keys());
        if let Some(Ok(e)) = keys.next().await {
            return Ok(leading_zeroes(e.as_bytes()));
        }

        Err(Error::EmptyTree)
    }

    /// Returns a stream of all keys in this tree.
    pub fn keys<'a>(&'a mut self) -> impl Stream<Item = Result<String, Error>> + 'a {
        // Start from the root of the tree.
        let mut stack = vec![Located::InSubtree(self.root)];

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

    /// Returns the specified record from the repository, or `None` if it does not exist.
    pub async fn get(&mut self, key: &str) -> Result<Option<Cid>, Error> {
        // Start from the root of the tree.
        let mut link = self.root;

        loop {
            let node = self.read_node(link).await?;
            match node.get(key) {
                None => return Ok(None),
                Some(Located::Entry(cid)) => return Ok(Some(cid)),
                Some(Located::InSubtree(cid)) => link = cid,
            }
        }
    }
}

/// The location of an entry in a Merkle Search Tree.
#[derive(Debug)]
pub enum Located<E> {
    /// The tree entry corresponding to a key.
    Entry(E),
    /// The CID of the [`Node`] containing the sub-tree in which a key is located.
    InSubtree(Cid),
}

#[derive(Debug, Clone)]
enum NodeEntry {
    /// A nested node.
    Tree(Cid),
    /// A tree entry.
    Leaf(TreeEntry),
}

impl NodeEntry {
    fn tree(&self) -> Option<&Cid> {
        match self {
            NodeEntry::Tree(cid) => Some(cid),
            _ => None,
        }
    }

    fn leaf(&self) -> Option<&TreeEntry> {
        match self {
            NodeEntry::Leaf(entry) => Some(entry),
            _ => None,
        }
    }
}

/// A node in a Merkle Search Tree.
#[derive(Debug, Clone)]
pub struct Node {
    /// The entries within this node.
    ///
    /// This list has the special property that no two `Tree` variants can be adjacent.
    entries: Vec<NodeEntry>,
}

impl Node {
    /// Parses an MST node from its DAG-CBOR encoding.
    pub fn parse(bytes: &[u8]) -> Result<Self, Error> {
        let node: schema::Node = serde_ipld_dagcbor::from_slice(bytes)?;

        let mut entries = vec![];
        if let Some(left) = &node.left {
            entries.push(NodeEntry::Tree(left.clone()));
        }

        let mut prev_key = vec![];
        for entry in &node.entries {
            let parsed_entry = TreeEntry::parse(entry.clone(), &prev_key)?;
            prev_key = parsed_entry.key.as_bytes().to_vec();

            entries.push(NodeEntry::Leaf(parsed_entry));

            // Nested subtrees are located to the right of the entry.
            if let Some(tree) = &entry.tree {
                entries.push(NodeEntry::Tree(tree.clone()));
            }
        }

        Ok(Self { entries })
    }

    pub async fn serialize_into(&self, mut bs: impl AsyncBlockStoreWrite) -> Result<Cid, Error> {
        let mut node = schema::Node { left: None, entries: vec![] };

        // Special case: if the first entry is a tree, that gets inserted into the node directly.
        let ents = match self.entries.first() {
            Some(NodeEntry::Tree(cid)) => {
                node.left = Some(cid.clone());
                &self.entries[1..]
            }
            _ => &self.entries,
        };

        let mut prev_key = vec![];
        let mut i = 0usize;
        while i != ents.len() {
            // Skip this window if the first entry is not a leaf.
            let leaf = if let Some(leaf) = ents.get(i).and_then(NodeEntry::leaf) {
                leaf
            } else {
                i += 1;
                continue;
            };
            let tree = ents.get(i + 1).and_then(NodeEntry::tree);

            let prefix = prefix(&prev_key, &leaf.key.as_bytes());

            node.entries.push(schema::TreeEntry {
                prefix_len: prefix,
                key_suffix: Ipld::Bytes(leaf.key[prefix..].as_bytes().to_vec()),
                value: leaf.value.clone(),
                tree: tree.cloned(),
            });

            prev_key = leaf.key.as_bytes().to_vec();
            i += 1;
        }

        let bytes = serde_ipld_dagcbor::to_vec(&node).unwrap();
        Ok(bs.write_block(DAG_CBOR, &bytes).await?)
    }

    fn leaves(&self) -> impl Iterator<Item = &TreeEntry> {
        self.entries.iter().filter_map(|entry| match entry {
            NodeEntry::Leaf(entry) => Some(entry),
            _ => None,
        })
    }

    /// Find the index of the first leaf node that has a key greater than or equal to the provided key.
    ///
    /// This may return an index that is equal to the length of `self.entries` (or in other words, OOB).
    /// If the node has no leaves, this will return `None`.
    fn find_ge(&self, key: &str) -> Option<usize> {
        let mut e = self.entries.iter().enumerate().filter_map(|(i, e)| e.leaf().map(|e| (i, e)));

        if let Some((i, _e)) = e.find(|(_i, e)| e.key.as_str() >= key) {
            Some(i)
        } else {
            if self.entries.len() != 0 {
                Some(self.entries.len())
            } else {
                None
            }
        }
    }

    /// Computes the node's layer, or returns `None` if this node has no leaves.
    fn layer(&self) -> Option<usize> {
        if let Some(e) = self.leaves().next() {
            Some(leading_zeroes(&e.key.as_bytes()))
        } else {
            None
        }
    }

    /// Finds the location of the given key's value within this sub-tree.
    ///
    /// Returns `None` if the key does not exist within this sub-tree.
    pub fn get(&self, key: &str) -> Option<Located<Cid>> {
        let i = self.find_ge(key)?;

        if let Some(NodeEntry::Leaf(e)) = self.entries.get(i) {
            if e.key == key {
                return Some(Located::Entry(e.value.clone()));
            }
        }

        if let Some(NodeEntry::Tree(cid)) = self.entries.get(i - 1) {
            Some(Located::InSubtree(cid.clone()))
        } else {
            None
        }
    }

    /// Returns the locations of values for all keys within this sub-tree with the given
    /// prefix.
    pub fn entries_with_prefix<'a>(
        &'a self,
        prefix: &str,
    ) -> impl DoubleEndedIterator<Item = Located<(&'a str, Cid)>> + 'a {
        let mut list = Vec::new();

        let index = if let Some(i) = self.find_ge(prefix) {
            i
        } else {
            // Special case: The tree is empty.
            return list.into_iter();
        };

        if let Some(NodeEntry::Tree(cid)) = self.entries.get(index - 1) {
            list.push(Located::InSubtree(cid.clone()));
        }

        // FIXME: Verify this logic.
        if let Some(e) = self.entries.get(index..) {
            for e in e.chunks(2) {
                if let NodeEntry::Leaf(t) = &e[0] {
                    if t.key.starts_with(prefix) {
                        list.push(Located::Entry((&t.key[..], t.value.clone())));

                        if let Some(NodeEntry::Tree(cid)) = e.get(1) {
                            list.push(Located::InSubtree(cid.clone()));
                        }
                    } else if prefix > t.key.as_str() {
                        if let Some(NodeEntry::Tree(cid)) = e.get(1) {
                            list.push(Located::InSubtree(cid.clone()));
                        }
                    }
                }
            }
        }

        list.into_iter()
    }

    /// Returns the locations of values for all keys within this sub-tree with the given
    /// prefix, in reverse order.
    pub fn reversed_entries_with_prefix<'a>(
        &'a self,
        prefix: &'a str,
    ) -> impl Iterator<Item = Located<(&'a str, Cid)>> + 'a {
        self.entries_with_prefix(prefix).rev()
    }
}

#[derive(Debug, Clone)]
struct TreeEntry {
    key: String,
    value: Cid,
}

impl TreeEntry {
    fn parse(entry: schema::TreeEntry, prev_key: &[u8]) -> Result<Self, Error> {
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

        let key = String::from_utf8(key).map_err(|e| e.utf8_error())?;

        Ok(Self { key, value: entry.value })
    }
}

/// Errors that can occur while interacting with an MST.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Invalid prefix_len")]
    InvalidPrefixLen,
    #[error("key_suffix not a byte string")]
    KeySuffixNotBytes,
    #[error("the tree is empty")]
    EmptyTree,
    #[error("the key is already present in the tree")]
    KeyAlreadyExists,
    #[error("Invalid key: {0}")]
    InvalidKey(#[from] std::str::Utf8Error),
    #[error("blockstore error: {0}")]
    BlockStore(#[from] crate::blockstore::Error),
    #[error("serde_ipld_dagcbor decoding error: {0}")]
    Parse(#[from] serde_ipld_dagcbor::DecodeError<Infallible>),
}

#[cfg(test)]
mod test {
    use std::str::FromStr;

    use ipld_core::cid::multihash::Multihash;

    use crate::blockstore::{MemoryBlockStore, SHA2_256};

    use super::*;

    /// b"bafyreie5cvv4h45feadgeuwhbcutmh6t2ceseocckahdoe6uat64zmz454"
    fn value_cid() -> Cid {
        Cid::new_v1(
            DAG_CBOR,
            match Multihash::wrap(
                SHA2_256,
                &[
                    157, 21, 107, 195, 243, 165, 32, 6, 98, 82, 199, 8, 169, 54, 31, 211, 208, 137,
                    34, 56, 66, 80, 14, 55, 19, 212, 4, 253, 204, 179, 60, 239,
                ],
            ) {
                Ok(h) => h,
                Err(_e) => panic!(),
            },
        )
    }

    #[test]
    fn test_prefix() {
        assert_eq!(
            prefix(b"com.example.record/3jqfcqzm3fo2j", b"com.example.record/3jqfcqzm3fo2j"),
            32
        );
        assert_eq!(
            prefix(b"com.example.record/3jqfcqzm3fo2j", b"com.example.record/7jqfcqzm3fo2j"),
            19
        );
    }

    #[test]
    fn test_clz() {
        assert_eq!(leading_zeroes(b""), 0);
        assert_eq!(leading_zeroes(b"app.bsky.feed.like/3lb6gx6opi727"), 0);
    }

    #[test]
    fn node_find_ge() {
        let node = Node { entries: vec![] };
        assert_eq!(node.find_ge("com.example.record/3jqfcqzm3fp2j"), None);

        let node = Node {
            entries: vec![NodeEntry::Leaf(TreeEntry {
                key: "com.example.record/3jqfcqzm3fs2j".to_string(), // '3..s'
                value: value_cid(),
            })],
        };

        assert_eq!(node.find_ge("com.example.record/3jqfcqzm3fp2j"), Some(0)); // '3..p'
        assert_eq!(node.find_ge("com.example.record/3jqfcqzm3fs2j"), Some(0)); // '3..s'
        assert_eq!(node.find_ge("com.example.record/3jqfcqzm3ft2j"), Some(1)); // '3..t'
        assert_eq!(node.find_ge("com.example.record/3jqfcqzm4fc2j"), Some(1)); // '4..c'
    }

    #[tokio::test]
    async fn mst_create() {
        let bs = MemoryBlockStore::new();
        let tree = Tree::create(bs).await.unwrap();

        assert_eq!(
            tree.root,
            Cid::from_str("bafyreie5737gdxlw5i64vzichcalba3z2v5n6icifvx5xytvske7mr3hpm").unwrap()
        );
    }

    #[tokio::test]
    async fn mst_create_trivial() {
        let bs = MemoryBlockStore::new();
        let mut tree = Tree::create(bs).await.unwrap();

        tree.add("com.example.record/3jqfcqzm3fo2j", value_cid()).await.unwrap();

        assert_eq!(
            tree.root,
            Cid::from_str("bafyreibj4lsc3aqnrvphp5xmrnfoorvru4wynt6lwidqbm2623a6tatzdu").unwrap()
        );
    }

    #[tokio::test]
    async fn mst_create_singlelayer2() {
        let bs = MemoryBlockStore::new();
        let mut tree = Tree::create(bs).await.unwrap();

        tree.add("com.example.record/3jqfcqzm3fx2j", value_cid()).await.unwrap();

        assert_eq!(
            tree.root,
            Cid::from_str("bafyreih7wfei65pxzhauoibu3ls7jgmkju4bspy4t2ha2qdjnzqvoy33ai").unwrap()
        );
    }

    #[tokio::test]
    async fn mst_create_simple() {
        let bs = MemoryBlockStore::new();
        let mut tree = Tree::create(bs).await.unwrap();

        tree.add("com.example.record/3jqfcqzm3fp2j", value_cid()).await.unwrap(); // level 0
        tree.add("com.example.record/3jqfcqzm3fr2j", value_cid()).await.unwrap(); // level 0
        tree.add("com.example.record/3jqfcqzm3fs2j", value_cid()).await.unwrap(); // level 1
        tree.add("com.example.record/3jqfcqzm3ft2j", value_cid()).await.unwrap(); // level 0
        tree.add("com.example.record/3jqfcqzm4fc2j", value_cid()).await.unwrap(); // level 0

        assert_eq!(
            tree.root,
            Cid::from_str("bafyreicmahysq4n6wfuxo522m6dpiy7z7qzym3dzs756t5n7nfdgccwq7m").unwrap()
        );
    }

    #[tokio::test]
    async fn mst_trim_top() {
        let bs = MemoryBlockStore::new();
        let mut tree = Tree::create(bs).await.unwrap();

        tree.add("com.example.record/3jqfcqzm3fn2j", value_cid()).await.unwrap(); // level 0
        tree.add("com.example.record/3jqfcqzm3fo2j", value_cid()).await.unwrap(); // level 0
        tree.add("com.example.record/3jqfcqzm3fp2j", value_cid()).await.unwrap(); // level 0
        tree.add("com.example.record/3jqfcqzm3fs2j", value_cid()).await.unwrap(); // level 1
        tree.add("com.example.record/3jqfcqzm3ft2j", value_cid()).await.unwrap(); // level 0
        tree.add("com.example.record/3jqfcqzm3fu2j", value_cid()).await.unwrap(); // level 0

        assert_eq!(
            tree.root,
            Cid::from_str("bafyreifnqrwbk6ffmyaz5qtujqrzf5qmxf7cbxvgzktl4e3gabuxbtatv4").unwrap()
        );

        tree.delete("com.example.record/3jqfcqzm3fs2j").await.unwrap(); // level 1

        assert_eq!(
            tree.root,
            Cid::from_str("bafyreie4kjuxbwkhzg2i5dljaswcroeih4dgiqq6pazcmunwt2byd725vi").unwrap()
        );
    }

    #[tokio::test]
    async fn mst_insert() {
        let bs = MemoryBlockStore::new();
        let mut tree = Tree::create(bs).await.unwrap();

        tree.add("com.example.record/3jqfcqzm3fo2j", Cid::default()).await.unwrap();
    }
}
