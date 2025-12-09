use super::nodes::NodeId;
use super::nodes::Nodes;
use ruby_prism::Node;

/// Semantic model for a Ruby file.
#[derive(Debug)]
pub struct SemanticModel<'a> {
    nodes: Nodes<'a>,
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
    /// Get access to the underlying nodes collection.
    #[inline]
    pub fn nodes(&self) -> &Nodes<'a> {
        &self.nodes
    }
    /// Push a node onto the traversal stack.
    #[inline]
    pub fn push_node(&mut self, node: Node<'a>) -> NodeId {
        let parent_id = self.current_node_id;
        let new_id = self.nodes.insert(node, parent_id);
        self.current_node_id = Some(new_id);
        new_id
    }
    /// Pop the current node from the traversal stack.
    #[inline]
    pub fn pop_node(&mut self) {
        if let Some(id) = self.current_node_id {
            self.current_node_id = self.nodes.parent_id(id);
        }
    }
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
    /// Get the parent ID of the current node.
    #[inline]
    pub fn parent_id(&self) -> Option<NodeId> {
        self.current_node_id.and_then(|id| self.nodes.parent_id(id))
    }
    /// Get the parent of the current node.
    #[inline]
    pub fn parent(&self) -> Option<&Node<'a>> {
        self.current_node_id
            .and_then(|id| self.nodes.parent_id(id))
            .and_then(|parent_id| self.nodes.get(parent_id))
    }
    /// Get the parent of the current node with its ID.
    #[inline]
    pub fn parent_with_id(&self) -> Option<(NodeId, &Node<'a>)> {
        self.current_node_id
            .and_then(|id| self.nodes.parent_id(id))
            .and_then(|parent_id| self.nodes.get(parent_id).map(|node| (parent_id, node)))
    }
    /// Get the ID of the Nth ancestor of the current node (0 = parent, 1 = grandparent, etc.)
    #[inline]
    pub fn ancestor_id(&self, n: usize) -> Option<NodeId> {
        self.current_node_id.and_then(|id| self.nodes.ancestor_ids(id).nth(n + 1))
    }
    /// Get the Nth ancestor of the current node (0 = parent, 1 = grandparent, etc.)
    #[inline]
    pub fn ancestor(&self, n: usize) -> Option<&Node<'a>> {
        self.current_node_id
            .and_then(|id| self.nodes.ancestor_ids(id).nth(n + 1))
            .and_then(|ancestor_id| self.nodes.get(ancestor_id))
    }
    /// Get the Nth ancestor of the current node with its ID (0 = parent, 1 = grandparent, etc.)
    #[inline]
    pub fn ancestor_with_id(&self, n: usize) -> Option<(NodeId, &Node<'a>)> {
        self.current_node_id
            .and_then(|id| self.nodes.ancestor_ids(id).nth(n + 1))
            .and_then(|ancestor_id| self.nodes.get(ancestor_id).map(|node| (ancestor_id, node)))
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
    /// Get the NodeId for a node.
    #[inline]
    pub fn node_id_for(&self, node: &Node<'_>) -> Option<NodeId> {
        let loc = node.location();
        self.nodes.node_id_for_location(loc.start_offset(), loc.end_offset())
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
