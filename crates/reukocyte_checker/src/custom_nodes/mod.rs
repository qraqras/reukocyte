//! Custom node wrappers for unified handling of related node types.
//!
//! This module provides enum wrappers that unify multiple Prism node types
//! under a common interface, allowing rules to implement `Check<CustomNode>`
//! once instead of implementing for each individual node type.
//!
//! # Example
//!
//! ```ignore
//! use crate::custom_nodes::AssignmentNode;
//!
//! impl Check<AssignmentNode<'_>> for MyRule {
//!     fn check(node: &AssignmentNode, checker: &mut Checker) {
//!         // Handle all assignment types uniformly
//!     }
//! }
//! ```

mod assignment;
mod conditional;

pub use assignment::AssignmentNode;
pub use conditional::ConditionalNode;
