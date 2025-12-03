#![allow(unused_variables)]
use crate::checker::Checker;
use ruby_prism::BlockNode;

/// Run lint rules over a [`BlockNode`] syntax node.
pub(crate) fn block_node(node: &BlockNode, checker: &mut Checker) {
    // TODO: Add rules for BlockNode
}
