#![allow(unused_variables)]
use crate::checker::Checker;
use ruby_prism::ConstantPathOperatorWriteNode;

/// Run lint rules over a [`ConstantPathOperatorWriteNode`] syntax node.
pub(crate) fn constant_path_operator_write_node(node: &ConstantPathOperatorWriteNode, checker: &mut Checker) {
    // TODO: Add rules for ConstantPathOperatorWriteNode
}
