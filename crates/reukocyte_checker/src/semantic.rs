use ruby_prism::*;
use rustc_hash::FxHashMap;

/// Node ID type for indexing.
pub type NodeId = usize;

/// Semantic model for indexing AST nodes and querying ancestors.
pub struct SemanticModel<'rk> {
    /// Owned nodes to keep them alive.
    pub nodes: Vec<Box<Node<'rk>>>,
    /// Map from node ID to the actual node.
    pub node_map: FxHashMap<NodeId, &'rk Node<'rk>>,
    /// Map from node pointer to node ID.
    pub node_id_map: FxHashMap<*const Node<'rk>, NodeId>,
    /// Map from node ID to its parent node ID.
    pub parent_map: FxHashMap<NodeId, NodeId>,
    /// Map from node ID to its child node IDs.
    pub children_map: FxHashMap<NodeId, Vec<NodeId>>,
    /// Current node ID counter.
    next_id: NodeId,
}

impl<'rk> SemanticModel<'rk> {
    /// Create a new empty SemanticModel.
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            node_map: FxHashMap::default(),
            node_id_map: FxHashMap::default(),
            parent_map: FxHashMap::default(),
            children_map: FxHashMap::default(),
            next_id: 0,
        }
    }

    /// Allocate a new node ID.
    fn alloc_id(&mut self) -> NodeId {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    /// Add a node to the model with a given parent.
    pub fn add_node(&mut self, node: *const Node<'rk>, parent_id: Option<NodeId>) -> NodeId {
        let id = self.alloc_id();
        self.node_map.insert(id, unsafe { &*node });
        self.node_id_map.insert(node, id);
        if let Some(parent_id) = parent_id {
            self.parent_map.insert(id, parent_id);
            self.children_map.entry(parent_id).or_insert_with(Vec::new).push(id);
        }
        id
    }

    /// Get the node ID for a given node pointer.
    pub fn node_id(&self, node: *const Node<'rk>) -> Option<NodeId> {
        self.node_id_map.get(&node).copied()
    }

    /// Get the parent node ID for a given node ID.
    pub fn parent_id(&self, node_id: NodeId) -> Option<NodeId> {
        self.parent_map.get(&node_id).copied()
    }

    /// Get the ancestors of a node (from closest to root).
    pub fn ancestors(&self, node: *const Node<'rk>) -> Vec<NodeId> {
        let mut ancestors = Vec::new();
        let mut current_id = self.node_id(node);
        while let Some(id) = current_id {
            if let Some(parent_id) = self.parent_id(id) {
                ancestors.push(parent_id);
                current_id = Some(parent_id);
            } else {
                break;
            }
        }
        ancestors
    }
}

/// Indexer to build the SemanticModel by traversing the AST.
pub struct Indexer<'rk> {
    model: SemanticModel<'rk>,
    parent_stack: Vec<NodeId>,
}

impl<'rk> Indexer<'rk> {
    /// Create a new Indexer.
    pub fn new() -> Self {
        Self {
            model: SemanticModel::new(),
            parent_stack: Vec::new(),
        }
    }

    /// Index the AST and return the SemanticModel.
    pub fn index(mut self, root: &Node<'rk>) -> SemanticModel<'rk> {
        self.visit(root);
        self.model
    }
}

impl<'rk> Visit<'rk> for Indexer<'rk> {
    fn visit_branch_node_enter(&mut self, node: Node<'rk>) {
        self.model.nodes.push(Box::new(node));
        let ptr = self.model.nodes.last().unwrap().as_ref() as *const Node<'rk>;
        let parent_id = self.parent_stack.last().map(|id| *id);
        self.model.add_node(ptr, parent_id);
        let id = self.model.node_id(ptr).unwrap();
        self.parent_stack.push(id);
    }
    fn visit_branch_node_leave(&mut self) {
        self.parent_stack.pop();
    }
    fn visit_leaf_node_enter(&mut self, node: Node<'rk>) {
        self.model.nodes.push(Box::new(node));
        let ptr = self.model.nodes.last().unwrap().as_ref() as *const Node<'rk>;
        let parent_id = self.parent_stack.last().map(|id| *id);
        self.model.add_node(ptr, parent_id);
    }
}
