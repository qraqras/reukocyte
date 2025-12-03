#![allow(unused_variables)]
use crate::checker::Checker;
use ruby_prism::ConstantPathAndWriteNode;

/// Run lint rules over a [`ConstantPathAndWriteNode`] syntax node.
pub(crate) fn constant_path_and_write_node(node: &ConstantPathAndWriteNode, checker: &mut Checker) {
    // TODO: Add rules for ConstantPathAndWriteNode
}
