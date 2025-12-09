//! Node storage and parent tracking.
//!
//! Provides `NodeId` for identifying nodes and `Nodes` for storing all visited
//! nodes with their parent relationships.

use ruby_prism::Node;
use std::num::NonZeroU32;

/// A unique identifier for an AST node within a file.
///
/// Uses `NonZeroU32` internally to allow `Option<NodeId>` to be the same size
/// as `NodeId` (niche optimization).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct NodeId(NonZeroU32);

impl NodeId {
    /// Create a new NodeId from an index.
    ///
    /// # Panics
    /// Panics if index >= u32::MAX - 1 (reserved for niche).
    #[inline]
    fn new(index: usize) -> Self {
        debug_assert!(index < (u32::MAX - 1) as usize, "NodeId index overflow");
        // Add 1 to make it non-zero
        Self(NonZeroU32::new((index as u32) + 1).unwrap())
    }

    /// Get the index as usize.
    #[inline]
    fn index(self) -> usize {
        (self.0.get() - 1) as usize
    }
}

/// An AST node with a pointer to its parent node.
#[derive(Debug)]
struct NodeWithParent<'a> {
    /// The AST node.
    node: Node<'a>,
    /// The ID of the parent node, if any.
    parent: Option<NodeId>,
}

/// Storage for all visited AST nodes, indexed by `NodeId`.
///
/// Nodes are inserted during AST traversal, and parent relationships are
/// automatically tracked based on the traversal order.
#[derive(Debug)]
pub struct Nodes<'a> {
    nodes: Vec<NodeWithParent<'a>>,
}

impl<'a> Nodes<'a> {
    /// Create a new, empty Nodes collection.
    #[inline]
    pub fn new() -> Self {
        Self {
            nodes: Vec::with_capacity(256), // Typical file has many nodes
        }
    }

    /// Insert a new node and return its unique ID.
    ///
    /// # Arguments
    /// * `node` - The AST node to insert
    /// * `parent` - The ID of the parent node, if any
    #[inline]
    pub fn insert(&mut self, node: Node<'a>, parent: Option<NodeId>) -> NodeId {
        let id = NodeId::new(self.nodes.len());
        self.nodes.push(NodeWithParent { node, parent });
        id
    }

    /// Get the parent ID of a node.
    #[inline]
    pub fn parent_id(&self, node_id: NodeId) -> Option<NodeId> {
        self.nodes.get(node_id.index()).and_then(|n| n.parent)
    }

    /// Get a node by its ID.
    #[inline]
    pub fn get(&self, node_id: NodeId) -> Option<&Node<'a>> {
        self.nodes.get(node_id.index()).map(|n| &n.node)
    }

    /// Get the parent node of a given node.
    #[inline]
    pub fn parent(&self, node_id: NodeId) -> Option<&Node<'a>> {
        self.parent_id(node_id).and_then(|pid| self.get(pid))
    }

    /// Iterate over all ancestor IDs, starting from the given node.
    ///
    /// The iterator yields the node itself first, then its parent, grandparent, etc.
    #[inline]
    pub fn ancestor_ids(&self, node_id: NodeId) -> impl Iterator<Item = NodeId> + '_ {
        std::iter::successors(Some(node_id), |&id| self.parent_id(id))
    }

    /// Iterate over all ancestor nodes, starting from the given node.
    ///
    /// The iterator yields the node itself first, then its parent, grandparent, etc.
    #[inline]
    pub fn ancestors(&self, node_id: NodeId) -> impl Iterator<Item = &Node<'a>> + '_ {
        self.ancestor_ids(node_id).filter_map(|id| self.get(id))
    }

    /// Get the number of nodes stored.
    #[inline]
    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    /// Check if the collection is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }
}

impl Default for Nodes<'_> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_id_creation() {
        let id0 = NodeId::new(0);
        let id1 = NodeId::new(1);

        assert_eq!(id0.index(), 0);
        assert_eq!(id1.index(), 1);
        assert_ne!(id0, id1);
    }

    #[test]
    fn test_option_node_id_size() {
        // Verify niche optimization works
        assert_eq!(std::mem::size_of::<NodeId>(), std::mem::size_of::<Option<NodeId>>());
    }
}
