//! Semantic model for Ruby AST analysis.
//!
//! The `SemanticModel` tracks the current position in the AST during traversal
//! and provides efficient access to parent and ancestor nodes.

use ruby_prism::Node;

use super::nodes::{NodeId, Nodes};

/// Semantic model for a Ruby file.
///
/// Tracks all visited AST nodes and their parent relationships during traversal.
/// This enables efficient lookups of parent/ancestor nodes from any position.
///
/// ## Example
///
/// ```ignore
/// let mut semantic = SemanticModel::new();
///
/// // During traversal
/// let id = semantic.push_node(some_node);
/// // ... visit children ...
/// semantic.pop_node();
///
/// // Later, look up parent
/// if let Some(parent) = semantic.parent() {
///     // ...
/// }
/// ```
#[derive(Debug)]
pub struct SemanticModel<'a> {
    /// All visited nodes with parent pointers.
    nodes: Nodes<'a>,

    /// The ID of the currently visiting node.
    current_node_id: Option<NodeId>,
}

impl<'a> SemanticModel<'a> {
    /// Create a new, empty semantic model.
    #[inline]
    pub fn new() -> Self {
        Self {
            nodes: Nodes::new(),
            current_node_id: None,
        }
    }

    // ========== Node traversal ==========

    /// Push a node onto the traversal stack.
    ///
    /// This registers the node in the model and sets it as the current node.
    /// The current node becomes the parent for subsequent push operations.
    ///
    /// Returns the unique ID assigned to this node.
    #[inline]
    pub fn push_node(&mut self, node: Node<'a>) -> NodeId {
        let parent_id = self.current_node_id;
        let new_id = self.nodes.insert(node, parent_id);
        self.current_node_id = Some(new_id);
        new_id
    }

    /// Pop the current node from the traversal stack.
    ///
    /// This restores the parent node as the current node.
    #[inline]
    pub fn pop_node(&mut self) {
        if let Some(id) = self.current_node_id {
            self.current_node_id = self.nodes.parent_id(id);
        }
    }

    // ========== Current node accessors ==========

    /// Get the ID of the currently visiting node.
    #[inline]
    pub fn current_node_id(&self) -> Option<NodeId> {
        self.current_node_id
    }

    /// Get the currently visiting node.
    #[inline]
    pub fn current_node(&self) -> Option<&Node<'a>> {
        self.current_node_id.and_then(|id| self.nodes.get(id))
    }

    /// Get the currently visiting node with its ID.
    #[inline]
    pub fn current_node_with_id(&self) -> Option<(NodeId, &Node<'a>)> {
        self.current_node_id.and_then(|id| self.nodes.get(id).map(|node| (id, node)))
    }

    /// Get the parent of the current node.
    #[inline]
    pub fn parent(&self) -> Option<&Node<'a>> {
        self.current_node_id.and_then(|id| self.nodes.parent_id(id)).and_then(|pid| self.nodes.get(pid))
    }

    /// Get the parent of the current node with its ID.
    #[inline]
    pub fn parent_with_id(&self) -> Option<(NodeId, &Node<'a>)> {
        self.current_node_id
            .and_then(|id| self.nodes.parent_id(id))
            .and_then(|pid| self.nodes.get(pid).map(|node| (pid, node)))
    }

    /// Get the parent ID of the current node.
    #[inline]
    pub fn parent_id(&self) -> Option<NodeId> {
        self.current_node_id.and_then(|id| self.nodes.parent_id(id))
    }

    /// Get the Nth ancestor of the current node (0 = parent, 1 = grandparent, etc.)
    #[inline]
    pub fn ancestor(&self, n: usize) -> Option<&Node<'a>> {
        self.current_node_id
            .and_then(|id| self.nodes.ancestor_ids(id).nth(n + 1))
            .and_then(|aid| self.nodes.get(aid))
    }

    /// Get the Nth ancestor of the current node with its ID (0 = parent, 1 = grandparent, etc.)
    #[inline]
    pub fn ancestor_with_id(&self, n: usize) -> Option<(NodeId, &Node<'a>)> {
        self.current_node_id
            .and_then(|id| self.nodes.ancestor_ids(id).nth(n + 1))
            .and_then(|aid| self.nodes.get(aid).map(|node| (aid, node)))
    }

    /// Get the ID of the Nth ancestor of the current node (0 = parent, 1 = grandparent, etc.)
    #[inline]
    pub fn ancestor_id(&self, n: usize) -> Option<NodeId> {
        self.current_node_id.and_then(|id| self.nodes.ancestor_ids(id).nth(n + 1))
    }

    /// Iterate over ancestors of the current node (parent, grandparent, ...).
    ///
    /// Does NOT include the current node itself.
    #[inline]
    pub fn ancestors(&self) -> impl Iterator<Item = &Node<'a>> + '_ {
        self.current_node_id
            .into_iter()
            .flat_map(|id| self.nodes.ancestor_ids(id).skip(1))
            .filter_map(|id| self.nodes.get(id))
    }

    /// Iterate over ancestors of the current node with their IDs (parent, grandparent, ...).
    ///
    /// Does NOT include the current node itself.
    #[inline]
    pub fn ancestors_with_ids(&self) -> impl Iterator<Item = (NodeId, &Node<'a>)> + '_ {
        self.current_node_id
            .into_iter()
            .flat_map(|id| self.nodes.ancestor_ids(id).skip(1))
            .filter_map(|id| self.nodes.get(id).map(|node| (id, node)))
    }

    /// Check if any ancestor matches the given predicate.
    #[inline]
    pub fn has_ancestor<F>(&self, predicate: F) -> bool
    where
        F: Fn(&Node<'a>) -> bool,
    {
        self.ancestors().any(predicate)
    }

    /// Find the first ancestor that matches the predicate (closest to current node).
    #[inline]
    pub fn find_ancestor<F>(&self, predicate: F) -> Option<&Node<'a>>
    where
        F: Fn(&Node<'a>) -> bool,
    {
        self.ancestors().find(|node| predicate(node))
    }

    // ========== Arbitrary node accessors ==========

    /// Get a node by its ID.
    #[inline]
    pub fn node(&self, node_id: NodeId) -> Option<&Node<'a>> {
        self.nodes.get(node_id)
    }

    /// Get the parent ID of a specific node.
    #[inline]
    pub fn parent_id_of(&self, node_id: NodeId) -> Option<NodeId> {
        self.nodes.parent_id(node_id)
    }

    /// Get the parent of a specific node.
    #[inline]
    pub fn parent_of(&self, node_id: NodeId) -> Option<&Node<'a>> {
        self.nodes.parent(node_id)
    }

    /// Iterate over ancestors of a specific node.
    #[inline]
    pub fn ancestors_of(&self, node_id: NodeId) -> impl Iterator<Item = &Node<'a>> + '_ {
        self.nodes.ancestors(node_id).skip(1) // Skip the node itself
    }

    /// Get the ID of the Nth ancestor of a specific node (0 = parent, 1 = grandparent, etc.)
    #[inline]
    pub fn ancestor_id_of(&self, node_id: NodeId, n: usize) -> Option<NodeId> {
        self.nodes.ancestor_ids(node_id).nth(n + 1)
    }

    // ========== Internal access ==========

    /// Get access to the underlying nodes collection.
    #[inline]
    pub fn nodes(&self) -> &Nodes<'a> {
        &self.nodes
    }
}

impl Default for SemanticModel<'_> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_semantic_model_creation() {
        let model: SemanticModel = SemanticModel::new();
        assert!(model.current_node_id().is_none());
        assert!(model.parent().is_none());
    }
}
