#![allow(unused_variables)]
use crate::checker::Checker;
use ruby_prism::BlockParameterNode;

/// Run lint rules over a [`BlockParameterNode`] syntax node.
pub(crate) fn block_parameter_node(node: &BlockParameterNode, checker: &mut Checker) {
    // TODO: Add rules for BlockParameterNode
}
