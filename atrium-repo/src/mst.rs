use std::{cmp::Ordering, collections::HashSet, convert::Infallible};

use async_stream::try_stream;
use futures::Stream;
use ipld_core::{
    cid::{multihash::Multihash, Cid},
    ipld::Ipld,
};
use serde::{Deserialize, Serialize};
use sha2::Digest;

use crate::blockstore::{AsyncBlockStoreRead, AsyncBlockStoreWrite, DAG_CBOR, SHA2_256};

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

/// Merkle search tree helper algorithms.
mod algos {
    use super::*;

    pub enum TraverseAction<R, M> {
        /// Continue traversal into the specified `Cid`.
        Continue((Cid, M)),
        /// Stop traversal and return `R`.
        Stop(R),
    }

    /// Traverse a merkle search tree.
    ///
    /// This executes the closure provided in `f` and takes the action
    /// returned by the closure.
    /// This also keeps track of "seen" nodes, and if a node is seen twice, traversal
    /// is immediately halted and an error is returned.
    pub async fn traverse<R, M>(
        mut bs: impl AsyncBlockStoreRead,
        root: Cid,
        mut f: impl FnMut(Node, Cid) -> Result<TraverseAction<R, M>, Error>,
    ) -> Result<(Vec<(Node, M)>, R), Error> {
        let mut node_cid = root;
        let mut node_path = vec![];
        let mut seen = HashSet::new();

        loop {
            let node = Node::read_from(&mut bs, node_cid).await?;
            if !seen.insert(node_cid) {
                // This CID was already seen. There is a cycle in the graph.
                panic!();
            }

            match f(node.clone(), node_cid)? {
                TraverseAction::Continue((cid, meta)) => {
                    node_path.push((node, meta));
                    node_cid = cid;
                }
                TraverseAction::Stop(r) => {
                    return Ok((node_path, r));
                }
            }
        }
    }

    /// Traverse through the tree, finding the node that contains a key.
    pub fn traverse_find<'a>(
        key: &'a str,
    ) -> impl FnMut(Node, Cid) -> Result<TraverseAction<(Node, usize), usize>, Error> + 'a {
        move |node, _cid| -> Result<_, Error> {
            if let Some(index) = node.find_ge(key) {
                if let Some(NodeEntry::Leaf(e)) = node.entries.get(index) {
                    if e.key == key {
                        return Ok(TraverseAction::Stop((node, index)));
                    }
                }

                // Check if the left neighbor is a tree, and if so, recurse into it.
                if let Some(index) = index.checked_sub(1) {
                    if let Some(subtree) = node.entries.get(index).unwrap().tree() {
                        return Ok(TraverseAction::Continue((subtree.clone(), index)));
                    } else {
                        return Err(Error::KeyNotFound);
                    }
                } else {
                    // There is no left neighbor. The key is not present.
                    return Err(Error::KeyNotFound);
                }
            } else {
                // We've recursed into an empty node, so the key is not present in the tree.
                return Err(Error::KeyNotFound);
            }
        }
    }

    /// Traverse through the tree, finding the first node that consists of more than just a single
    /// nested tree entry.
    pub fn traverse_prune() -> impl FnMut(Node, Cid) -> Result<TraverseAction<Cid, usize>, Error> {
        move |node, cid| -> Result<_, Error> {
            if node.entries.len() == 1 {
                if let Some(NodeEntry::Tree(cid)) = node.entries.first() {
                    return Ok(TraverseAction::Continue((cid.clone(), 0)));
                }
            }

            return Ok(TraverseAction::Stop(cid));
        }
    }

    /// Recursively merge two subtrees into one.
    pub async fn merge_subtrees(
        mut bs: impl AsyncBlockStoreRead + AsyncBlockStoreWrite,
        mut lc: Cid,
        mut rc: Cid,
    ) -> Result<Cid, Error> {
        let mut node_path = vec![];

        let (ln, rn) = loop {
            // Traverse down both the left and right trees until we reach the first leaf node on either side.
            let ln = Node::read_from(&mut bs, lc).await?;
            let rn = Node::read_from(&mut bs, rc).await?;

            if let (Some(NodeEntry::Tree(l)), Some(NodeEntry::Tree(r))) =
                (ln.entries.last(), rn.entries.first())
            {
                node_path.push((ln.clone(), rn.clone()));

                lc = l.clone();
                rc = r.clone();
            } else {
                break (ln, rn);
            }
        };

        // Merge the two nodes.
        let node = Node { entries: ln.entries.into_iter().chain(rn.entries).collect() };
        let mut cid = node.serialize_into(&mut bs).await?;

        // Now go back up the node path chain and update parent entries.
        for (ln, rn) in node_path.into_iter().rev() {
            let node = Node {
                entries: ln.entries[..ln.entries.len() - 1]
                    .iter()
                    .cloned()
                    .chain([NodeEntry::Tree(cid)])
                    .chain(rn.entries[1..].iter().cloned())
                    .collect(),
            };

            cid = node.serialize_into(&mut bs).await?;
        }

        Ok(cid)
    }

    /// Recursively split a node based on a key.
    ///
    /// If the key is found within the subtree, this will return an error.
    pub async fn split_subtree(
        mut bs: impl AsyncBlockStoreRead + AsyncBlockStoreWrite,
        node: Cid,
        key: &str,
    ) -> Result<(Option<Cid>, Option<Cid>), Error> {
        let (node_path, (mut left, mut right)) = traverse(&mut bs, node, |mut node, _cid| {
            if let Some(partition) = node.find_ge(key) {
                // Ensure that the key does not already exist.
                if let Some(NodeEntry::Leaf(e)) = node.entries.get(partition) {
                    if e.key == key {
                        return Err(Error::KeyAlreadyExists);
                    }
                }

                // Determine if the left neighbor is a subtree. If so, we need to recursively split that tree.
                if let Some(partition) = partition.checked_sub(1) {
                    match node.entries.get(partition) {
                        Some(NodeEntry::Leaf(_e)) => {
                            // Left neighbor is a leaf, so we can split the current node into two and we are done.
                            let right = node.entries.split_off(partition + 1);

                            return Ok(TraverseAction::Stop((
                                Some(node),
                                (!right.is_empty()).then(|| Node { entries: right }),
                            )));
                        }
                        Some(NodeEntry::Tree(e)) => {
                            return Ok(TraverseAction::Continue((e.clone(), partition)));
                        }
                        // This should not happen; node.find_ge() should return `None` in this case.
                        None => panic!(),
                    }
                } else {
                    return Ok(TraverseAction::Stop((None, Some(node))));
                }
            } else {
                todo!()
            }
        })
        .await?;

        // If the node was split into two, walk back up the path chain and split all parents.
        for (mut parent, i) in node_path.into_iter().rev() {
            // Remove the tree entry at the partition point.
            parent.entries.remove(i);
            let (e_left, e_right) = parent.entries.split_at(i);

            if let Some(left) = left.as_mut() {
                let left_cid = left.serialize_into(&mut bs).await?;
                *left = Node {
                    entries: e_left.iter().cloned().chain([NodeEntry::Tree(left_cid)]).collect(),
                };
            }

            if let Some(right) = right.as_mut() {
                let right_cid = right.serialize_into(&mut bs).await?;
                *right = Node {
                    entries: [NodeEntry::Tree(right_cid)]
                        .into_iter()
                        .chain(e_right.iter().cloned())
                        .collect::<Vec<_>>(),
                };
            }
        }

        // Serialize the two new subtrees.
        let left =
            if let Some(left) = left { Some(left.serialize_into(&mut bs).await?) } else { None };
        let right =
            if let Some(right) = right { Some(right.serialize_into(&mut bs).await?) } else { None };

        Ok((left, right))
    }
}

// https://users.rust-lang.org/t/how-to-find-common-prefix-of-two-byte-slices-effectively/25815/3
fn prefix(xs: &[u8], ys: &[u8]) -> usize {
    prefix_chunks::<128>(xs, ys)
}

fn prefix_chunks<const N: usize>(xs: &[u8], ys: &[u8]) -> usize {
    // N.B: We take exact chunks here to entice the compiler to autovectorize this loop.
    let off =
        std::iter::zip(xs.chunks_exact(N), ys.chunks_exact(N)).take_while(|(x, y)| x == y).count()
            * N;
    off + std::iter::zip(&xs[off..], &ys[off..]).take_while(|(x, y)| x == y).count()
}

/// Calculate the number of leading zeroes from the sha256 hash of a byte array
///
/// Reference: https://github.com/bluesky-social/atproto/blob/13636ba963225407f63c20253b983a92dcfe1bfa/packages/repo/src/mst/util.ts#L8-L23
fn leading_zeroes(key: &[u8]) -> usize {
    let digest = sha2::Sha256::digest(key);
    let mut zeroes = 0;

    for byte in digest.iter() {
        /* 64 */
        if *byte < 0b0100_0000 {
            zeroes += 1;
        }
        /* 16 */
        if *byte < 0b0001_0000 {
            zeroes += 1;
        }
        /* 4 */
        if *byte < 0b0000_0100 {
            zeroes += 1;
        }

        if *byte == 0 {
            zeroes += 1;
        } else {
            // If the byte is nonzero, then there cannot be any more leading zeroes.
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
/// # Reference
/// * Official documentation: https://atproto.com/guides/data-repos
/// * Useful reading: https://interjectedfuture.com/crdts-turned-inside-out/
pub struct Tree<S> {
    storage: S,
    root: Cid,
}

impl<S: AsyncBlockStoreRead + AsyncBlockStoreWrite> Tree<S> {
    /// Create a new MST with an empty root node
    pub async fn create(mut storage: S) -> Result<Self, Error> {
        let node = Node { entries: vec![] };
        let cid = node.serialize_into(&mut storage).await?;

        Ok(Self { storage, root: cid })
    }

    pub async fn add(&mut self, key: &str, value: Cid) -> Result<(), Error> {
        // Compute the layer where this note should be added.
        let target_layer = leading_zeroes(key.as_bytes());

        // Now traverse to the node containing the target layer.
        let mut node_path = vec![];
        let mut node_cid = self.root.clone();

        // There are three cases we need to handle:
        // 1) The target layer is above the tree (and our entire tree needs to be pushed down).
        // 2) The target layer is contained within the tree (and we will traverse to find it).
        // 3) The tree is currently empty (trivial).
        let mut node = match self.depth(None).await {
            Ok(layer) => {
                match layer.cmp(&target_layer) {
                    // The new key can be inserted into the root node.
                    Ordering::Equal => Node::read_from(&mut self.storage, node_cid).await?,
                    // The entire tree needs to be shifted down.
                    Ordering::Less => {
                        let mut layer = layer + 1;

                        loop {
                            let node = Node { entries: vec![NodeEntry::Tree(node_cid)] };

                            if layer < target_layer {
                                node_cid = node.serialize_into(&mut self.storage).await?;
                                layer += 1;
                            } else {
                                break node;
                            }
                        }
                    }
                    // Search in a subtree (most common).
                    Ordering::Greater => {
                        let mut layer = layer;

                        // Traverse to the lowest possible layer in the tree.
                        let (path, (mut node, partition)) =
                            algos::traverse(&mut self.storage, node_cid, |node, _cid| {
                                if layer == target_layer {
                                    Ok(algos::TraverseAction::Stop((node, 0)))
                                } else {
                                    let partition = node.find_ge(key).unwrap();

                                    // If left neighbor is a subtree, recurse through.
                                    if let Some(partition) = partition.checked_sub(1) {
                                        if let Some(subtree) =
                                            node.entries.get(partition).unwrap().tree()
                                        {
                                            layer -= 1;
                                            return Ok(algos::TraverseAction::Continue((
                                                subtree.clone(),
                                                partition,
                                            )));
                                        }
                                    }

                                    return Ok(algos::TraverseAction::Stop((node, partition)));
                                }
                            })
                            .await?;

                        node_path = path;
                        if layer == target_layer {
                            // A pre-existing node was found on the same layer.
                            node
                        } else {
                            // Insert a new dummy tree entry and push the last node onto the node path.
                            node.entries.insert(partition, NodeEntry::Tree(Cid::default()));
                            node_path.push((node, partition));
                            layer -= 1;

                            // Insert empty nodes until we reach the target layer.
                            while layer != target_layer {
                                let node = Node { entries: vec![NodeEntry::Tree(Cid::default())] };

                                node_path.push((node.clone(), 0));
                                layer -= 1;
                            }

                            // Insert the new leaf node.
                            Node { entries: vec![] }
                        }
                    }
                }
            }
            Err(Error::EmptyTree) => {
                // The tree is currently empty.
                Node { entries: vec![] }
            }
            Err(e) => return Err(e),
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
                    let (left, right) =
                        algos::split_subtree(&mut self.storage, e.clone(), key).await?;

                    // Insert the new node inbetween the two subtrees.
                    let right_subvec = node.entries.split_off(partition);

                    node.entries.pop();
                    if let Some(left) = left {
                        node.entries.push(NodeEntry::Tree(left));
                    }
                    node.entries
                        .extend([NodeEntry::Leaf(TreeEntry { key: key.to_string(), value })]);
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
        for (mut parent, i) in node_path.into_iter().rev() {
            parent.entries[i] = NodeEntry::Tree(cid);
            cid = parent.serialize_into(&mut self.storage).await?;
        }

        self.root = cid;
        Ok(())
    }

    /// Update an existing key with a new value.
    pub async fn update(&mut self, key: &str, value: Cid) -> Result<(), Error> {
        let (node_path, (mut node, index)) =
            algos::traverse(&mut self.storage, self.root, algos::traverse_find(key)).await?;

        // Update the value.
        node.entries[index] = NodeEntry::Leaf(TreeEntry { key: key.to_string(), value });

        let mut cid = node.serialize_into(&mut self.storage).await?;

        // Now walk up the node path chain and update parent entries to point to the new node's CID.
        for (mut parent, i) in node_path.into_iter().rev() {
            parent.entries[i] = NodeEntry::Tree(cid);
            cid = parent.serialize_into(&mut self.storage).await?;
        }

        self.root = cid;
        Ok(())
    }

    /// Delete a key from the tree.
    pub async fn delete(&mut self, key: &str) -> Result<(), Error> {
        let (node_path, (mut node, index)) =
            algos::traverse(&mut self.storage, self.root, algos::traverse_find(key)).await?;

        // Remove the key.
        node.entries.remove(index);

        if let Some(index) = index.checked_sub(1) {
            // Check to see if the left and right neighbors are both trees. If so, merge them.
            if let (Some(NodeEntry::Tree(lc)), Some(NodeEntry::Tree(rc))) =
                (node.entries.get(index), node.entries.get(index + 1))
            {
                let cid = algos::merge_subtrees(&mut self.storage, lc.clone(), rc.clone()).await?;
                node.entries[index] = NodeEntry::Tree(cid);
                node.entries.remove(index + 1);

                // If the resulting node consists of a single tree entry, elide it.
                if node.entries.as_slice() == &[NodeEntry::Tree(cid)] {
                    node = Node::read_from(&mut self.storage, cid).await?;
                }
            }
        }

        let mut cid = node.serialize_into(&mut self.storage).await?;

        // Now walk back up the node path chain and update parent entries to point to the new node's CID.
        for (mut parent, i) in node_path.into_iter().rev() {
            parent.entries[i] = NodeEntry::Tree(cid);
            cid = parent.serialize_into(&mut self.storage).await?;
        }

        self.root = cid;
        self.prune().await?;
        Ok(())
    }

    /// Prune entries that contain a single nested tree entry from the root.
    async fn prune(&mut self) -> Result<(), Error> {
        let (_node_path, cid) =
            algos::traverse(&mut self.storage, self.root, algos::traverse_prune()).await?;

        self.root = cid;
        Ok(())
    }
}

impl<S: AsyncBlockStoreRead> Tree<S> {
    /// Open a pre-existing merkle search tree.
    ///
    /// This is a very cheap operation that does not actually load the MST
    /// or check its validity. You should only use this with data from a trusted
    /// source.
    pub fn open(storage: S, root: Cid) -> Self {
        Self { storage, root }
    }

    /// Return the CID of the root node.
    pub fn root(&self) -> Cid {
        self.root
    }

    async fn resolve_subtree<K>(
        &mut self,
        link: Cid,
        prefix: &str,
        key_fn: impl Fn(&str, Cid) -> Result<K, Error>,
        stack: &mut Vec<Located<K>>,
    ) -> Result<(), Error> {
        let node = Node::read_from(&mut self.storage, link).await?;

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
        let node = Node::read_from(&mut self.storage, link).await?;

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

    /// Compute the depth of the merkle search tree from either the specified node or the root
    pub async fn depth(&mut self, node: Option<Cid>) -> Result<usize, Error> {
        // Recursively iterate through the tree until we encounter a leaf node, and then
        // use that to calculate the depth of the entire tree.
        let mut subtrees = vec![(node.unwrap_or(self.root), 0usize)];

        loop {
            if let Some((subtree, depth)) = subtrees.pop() {
                let node = Node::read_from(&mut self.storage, subtree).await?;
                if let Some(layer) = node.layer() {
                    return Ok(depth + layer);
                }

                subtrees.extend(node.trees().cloned().zip(std::iter::repeat(depth + 1)));
            } else {
                return Err(Error::EmptyTree);
            }
        }
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
        match algos::traverse(&mut self.storage, self.root, algos::traverse_find(key)).await {
            // FIXME: The `unwrap` call here isn't preferable, but it is guaranteed to succeed.
            Ok((_node_path, (node, index))) => Ok(Some(node.entries[index].leaf().unwrap().value)),
            Err(Error::KeyNotFound) => Ok(None),
            Err(e) => Err(e),
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

#[derive(Debug, Clone, PartialEq)]
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

    /// Read and parse a node from block storage
    pub async fn read_from(mut bs: impl AsyncBlockStoreRead, cid: Cid) -> Result<Self, Error> {
        let bytes = bs.read_block(&cid).await?;
        Self::parse(&bytes)
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

    /// Return an iterator of the subtrees contained within this node
    fn trees(&self) -> impl Iterator<Item = &Cid> {
        self.entries.iter().filter_map(|entry| match entry {
            NodeEntry::Tree(entry) => Some(entry),
            _ => None,
        })
    }

    /// Return an iterator of the leaves contained within this node
    fn leaves(&self) -> impl Iterator<Item = &TreeEntry> {
        self.entries.iter().filter_map(|entry| match entry {
            NodeEntry::Leaf(entry) => Some(entry),
            _ => None,
        })
    }

    /// Computes the node's layer, or returns `None` if this node has no leaves.
    fn layer(&self) -> Option<usize> {
        if let Some(e) = self.leaves().next() {
            Some(leading_zeroes(&e.key.as_bytes()))
        } else {
            None
        }
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

        if let Some(index) = index.checked_sub(1) {
            if let Some(NodeEntry::Tree(cid)) = self.entries.get(index) {
                list.push(Located::InSubtree(cid.clone()));
            }
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

#[derive(Debug, Clone, PartialEq)]
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
    #[error("the key is not present in the tree")]
    KeyNotFound,
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

    /// Returns a dummy value Cid used for testing.
    ///
    /// b"bafyreie5cvv4h45feadgeuwhbcutmh6t2ceseocckahdoe6uat64zmz454"
    fn value_cid() -> Cid {
        Cid::new_v1(
            DAG_CBOR,
            match Multihash::wrap(
                SHA2_256,
                &[
                    0x9d, 0x15, 0x6b, 0xc3, 0xf3, 0xa5, 0x20, 0x06, 0x62, 0x52, 0xc7, 0x08, 0xa9,
                    0x36, 0x1f, 0xd3, 0xd0, 0x89, 0x22, 0x38, 0x42, 0x50, 0x0e, 0x37, 0x13, 0xd4,
                    0x04, 0xfd, 0xcc, 0xb3, 0x3c, 0xef,
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
        assert_eq!(leading_zeroes(b"com.example.record/3jqfcqzm3fn2j"), 0); // level 0
        assert_eq!(leading_zeroes(b"com.example.record/3jqfcqzm3fo2j"), 0); // level 0
        assert_eq!(leading_zeroes(b"com.example.record/3jqfcqzm3fp2j"), 0); // level 0
        assert_eq!(leading_zeroes(b"com.example.record/3jqfcqzm3fs2j"), 1); // level 1
        assert_eq!(leading_zeroes(b"com.example.record/3jqfcqzm3ft2j"), 0); // level 0
        assert_eq!(leading_zeroes(b"com.example.record/3jqfcqzm3fu2j"), 0); // level 0
        assert_eq!(leading_zeroes(b"com.example.record/3jqfcqzm3fx2j"), 2); // level 2
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
        assert_eq!(tree.depth(None).await.unwrap(), 1);

        tree.delete("com.example.record/3jqfcqzm3fs2j").await.unwrap(); // level 1

        assert_eq!(
            tree.root,
            Cid::from_str("bafyreie4kjuxbwkhzg2i5dljaswcroeih4dgiqq6pazcmunwt2byd725vi").unwrap()
        );
    }

    #[tokio::test]
    async fn mst_insertion_split() {
        let bs = MemoryBlockStore::new();
        let mut tree = Tree::create(bs).await.unwrap();

        let root11 =
            Cid::from_str("bafyreiettyludka6fpgp33stwxfuwhkzlur6chs4d2v4nkmq2j3ogpdjem").unwrap();
        let root12 =
            Cid::from_str("bafyreid2x5eqs4w4qxvc5jiwda4cien3gw2q6cshofxwnvv7iucrmfohpm").unwrap();

        /*
         *
         *                *                                  *
         *       _________|________                      ____|_____
         *       |   |    |    |   |                    |    |     |
         *       *   d    *    i   *       ->           *    f     *
         *     __|__    __|__    __|__                __|__      __|___
         *    |  |  |  |  |  |  |  |  |              |  |  |    |  |   |
         *    a  b  c  e  g  h  j  k  l              *  d  *    *  i   *
         *                                         __|__   |   _|_   __|__
         *                                        |  |  |  |  |   | |  |  |
         *                                        a  b  c  e  g   h j  k  l
         *
         */
        tree.add("com.example.record/3jqfcqzm3fo2j", value_cid()).await.unwrap(); // A; level 0
        tree.add("com.example.record/3jqfcqzm3fp2j", value_cid()).await.unwrap(); // B; level 0
        tree.add("com.example.record/3jqfcqzm3fr2j", value_cid()).await.unwrap(); // C; level 0
        tree.add("com.example.record/3jqfcqzm3fs2j", value_cid()).await.unwrap(); // D; level 1
        tree.add("com.example.record/3jqfcqzm3ft2j", value_cid()).await.unwrap(); // E; level 0
                                                                                  // GAP for F
        tree.add("com.example.record/3jqfcqzm3fz2j", value_cid()).await.unwrap(); // G; level 0
        tree.add("com.example.record/3jqfcqzm4fc2j", value_cid()).await.unwrap(); // H; level 0
        tree.add("com.example.record/3jqfcqzm4fd2j", value_cid()).await.unwrap(); // I; level 1
        tree.add("com.example.record/3jqfcqzm4ff2j", value_cid()).await.unwrap(); // J; level 0
        tree.add("com.example.record/3jqfcqzm4fg2j", value_cid()).await.unwrap(); // K; level 0
        tree.add("com.example.record/3jqfcqzm4fh2j", value_cid()).await.unwrap(); // L; level 0

        assert_eq!(tree.root, root11);

        // insert F, which will push E out of the node with G+H to a new node under D
        tree.add("com.example.record/3jqfcqzm3fx2j", value_cid()).await.unwrap(); // F; level 2

        assert_eq!(tree.root, root12);

        // insert K again. An error should be returned.
        assert!(matches!(
            tree.add("com.example.record/3jqfcqzm4fg2j", value_cid()).await.unwrap_err(), // K; level 0
            Error::KeyAlreadyExists
        ));

        assert_eq!(tree.root, root12);

        // remove F, which should push E back over with G+H
        tree.delete("com.example.record/3jqfcqzm3fx2j").await.unwrap(); // F; level 2

        assert_eq!(tree.root, root11);
    }

    #[tokio::test]
    async fn mst_two_layers() {
        let bs = MemoryBlockStore::new();
        let mut tree = Tree::create(bs).await.unwrap();

        let root10 =
            Cid::from_str("bafyreidfcktqnfmykz2ps3dbul35pepleq7kvv526g47xahuz3rqtptmky").unwrap();
        let root12 =
            Cid::from_str("bafyreiavxaxdz7o7rbvr3zg2liox2yww46t7g6hkehx4i4h3lwudly7dhy").unwrap();
        let root12_2 =
            Cid::from_str("bafyreig4jv3vuajbsybhyvb7gggvpwh2zszwfyttjrj6qwvcsp24h6popu").unwrap();

        /*
         *
         *          *        ->            *
         *        __|__                  __|__
         *       |     |                |  |  |
         *       a     c                *  b  *
         *                              |     |
         *                              *     *
         *                              |     |
         *                              a     c
         *
         */
        tree.add("com.example.record/3jqfcqzm3ft2j", value_cid()).await.unwrap(); // A; level 0
        tree.add("com.example.record/3jqfcqzm3fz2j", value_cid()).await.unwrap(); // C; level 0

        assert_eq!(tree.root, root10);

        // insert B, which is two levels above
        tree.add("com.example.record/3jqfcqzm3fx2j", value_cid()).await.unwrap(); // B; level 2

        assert_eq!(tree.root, root12);

        // remove B
        tree.delete("com.example.record/3jqfcqzm3fx2j").await.unwrap(); // B; level 2

        assert_eq!(tree.root, root10);

        // insert B (level=2) and D (level=1)
        tree.add("com.example.record/3jqfcqzm3fx2j", value_cid()).await.unwrap(); // B; level 2
        tree.add("com.example.record/3jqfcqzm4fd2j", value_cid()).await.unwrap(); // D; level 1

        assert_eq!(tree.root, root12_2);

        // remove D
        tree.delete("com.example.record/3jqfcqzm4fd2j").await.unwrap(); // D; level 1

        assert_eq!(tree.root, root12);
    }

    #[tokio::test]
    async fn mst_two_layers_rev() {
        let bs = MemoryBlockStore::new();
        let mut tree = Tree::create(bs).await.unwrap();

        let root10 =
            Cid::from_str("bafyreidfcktqnfmykz2ps3dbul35pepleq7kvv526g47xahuz3rqtptmky").unwrap();
        let root12 =
            Cid::from_str("bafyreiavxaxdz7o7rbvr3zg2liox2yww46t7g6hkehx4i4h3lwudly7dhy").unwrap();

        // This is the same test as `mst_two_layers`, but with the top level entry inserted first.
        tree.add("com.example.record/3jqfcqzm3fx2j", value_cid()).await.unwrap(); // B; level 2
        tree.add("com.example.record/3jqfcqzm3ft2j", value_cid()).await.unwrap(); // A; level 0
        tree.add("com.example.record/3jqfcqzm3fz2j", value_cid()).await.unwrap(); // C; level 0

        assert_eq!(tree.root, root12);

        // remove B
        tree.delete("com.example.record/3jqfcqzm3fx2j").await.unwrap(); // B; level 2

        assert_eq!(tree.root, root10);
    }

    #[tokio::test]
    async fn mst_insert() {
        let bs = MemoryBlockStore::new();
        let mut tree = Tree::create(bs).await.unwrap();

        tree.add("com.example.record/3jqfcqzm3fo2j", Cid::default()).await.unwrap();
    }
}
