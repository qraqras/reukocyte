#![allow(unused_variables)]
use crate::checker::Checker;
use ruby_prism::EnsureNode;

/// Run lint rules over a [`EnsureNode`] syntax node.
pub(crate) fn ensure_node(node: &EnsureNode, checker: &mut Checker) {
    // TODO: Add rules for EnsureNode
}
