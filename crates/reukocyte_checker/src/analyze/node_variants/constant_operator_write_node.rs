#![allow(unused_variables)]
use crate::checker::Checker;
use ruby_prism::ConstantOperatorWriteNode;

/// Run lint rules over a [`ConstantOperatorWriteNode`] syntax node.
pub(crate) fn constant_operator_write_node(node: &ConstantOperatorWriteNode, checker: &mut Checker) {
    // TODO: Add rules for ConstantOperatorWriteNode
}
