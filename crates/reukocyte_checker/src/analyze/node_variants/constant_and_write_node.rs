#![allow(unused_variables)]
use crate::checker::Checker;
use ruby_prism::ConstantAndWriteNode;

/// Run lint rules over a [`ConstantAndWriteNode`] syntax node.
pub(crate) fn constant_and_write_node(node: &ConstantAndWriteNode, checker: &mut Checker) {
    // TODO: Add rules for ConstantAndWriteNode
}
