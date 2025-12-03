#![allow(unused_variables)]
use crate::checker::Checker;
use ruby_prism::IndexTargetNode;

/// Run lint rules over a [`IndexTargetNode`] syntax node.
pub(crate) fn index_target_node(node: &IndexTargetNode, checker: &mut Checker) {
    // TODO: Add rules for IndexTargetNode
}
