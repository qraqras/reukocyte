#![allow(unused_variables)]
use crate::checker::Checker;
use ruby_prism::ConstantWriteNode;

/// Run lint rules over a [`ConstantWriteNode`] syntax node.
pub(crate) fn constant_write_node(node: &ConstantWriteNode, checker: &mut Checker) {
    // TODO: Add rules for ConstantWriteNode
}
