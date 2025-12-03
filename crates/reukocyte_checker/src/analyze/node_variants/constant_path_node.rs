#![allow(unused_variables)]
use crate::checker::Checker;
use ruby_prism::ConstantPathNode;

/// Run lint rules over a [`ConstantPathNode`] syntax node.
pub(crate) fn constant_path_node(node: &ConstantPathNode, checker: &mut Checker) {
    // TODO: Add rules for ConstantPathNode
}
