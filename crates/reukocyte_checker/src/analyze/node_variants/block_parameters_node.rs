#![allow(unused_variables)]
use crate::checker::Checker;
use ruby_prism::BlockParametersNode;

/// Run lint rules over a [`BlockParametersNode`] syntax node.
pub(crate) fn block_parameters_node(node: &BlockParametersNode, checker: &mut Checker) {
    // TODO: Add rules for BlockParametersNode
}
