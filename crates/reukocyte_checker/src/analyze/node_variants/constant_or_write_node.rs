#![allow(unused_variables)]
use crate::checker::Checker;
use ruby_prism::ConstantOrWriteNode;

/// Run lint rules over a [`ConstantOrWriteNode`] syntax node.
pub(crate) fn constant_or_write_node(node: &ConstantOrWriteNode, checker: &mut Checker) {
    // TODO: Add rules for ConstantOrWriteNode
}
