#![allow(unused_variables)]
use crate::checker::Checker;
use ruby_prism::ConstantPathWriteNode;

/// Run lint rules over a [`ConstantPathWriteNode`] syntax node.
pub(crate) fn constant_path_write_node(node: &ConstantPathWriteNode, checker: &mut Checker) {
    // TODO: Add rules for ConstantPathWriteNode
}
