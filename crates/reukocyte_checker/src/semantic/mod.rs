//! Semantic analysis for Ruby AST.
//!
//! This module provides the `SemanticModel` which tracks AST node relationships
//! during traversal, enabling efficient parent/ancestor lookups from any node.
//!
//! ## Architecture
//!
//! The design is inspired by Ruff's semantic model, adapted for Ruby:
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────┐
//! │ SemanticModel                                                   │
//! │ ├── nodes: Nodes          (all visited nodes with parent refs)  │
//! │ ├── current_node_id       (currently visiting node)             │
//! │ └── (future: scopes, bindings, etc.)                            │
//! └─────────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Usage
//!
//! During AST traversal:
//! 1. `push_node()` registers a node and sets it as current
//! 2. Rules can access `current_node_id()`, `parent()`, `ancestors()`, etc.
//! 3. `pop_node()` restores the previous node as current
//!
//! After traversal, any `NodeId` can be used to look up its parent chain.

mod model;
mod nodes;

pub use model::SemanticModel;
// NodeId is currently internal; export when needed by rules
#[allow(unused_imports)]
pub use nodes::NodeId;
