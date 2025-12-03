#![allow(unused_variables)]
use crate::checker::Checker;
use ruby_prism::ConstantPathOrWriteNode;

/// Run lint rules over a [`ConstantPathOrWriteNode`] syntax node.
pub(crate) fn constant_path_or_write_node(node: &ConstantPathOrWriteNode, checker: &mut Checker) {
    // TODO: Add rules for ConstantPathOrWriteNode
}
