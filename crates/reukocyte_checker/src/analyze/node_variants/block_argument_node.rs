#![allow(unused_variables)]
use crate::checker::Checker;
use ruby_prism::BlockArgumentNode;

/// Run lint rules over a [`BlockArgumentNode`] syntax node.
pub(crate) fn block_argument_node(node: &BlockArgumentNode, checker: &mut Checker) {
    // TODO: Add rules for BlockArgumentNode
}
