#![allow(unused_variables)]
use crate::checker::Checker;
use ruby_prism::BlockLocalVariableNode;

/// Run lint rules over a [`BlockLocalVariableNode`] syntax node.
pub(crate) fn block_local_variable_node(node: &BlockLocalVariableNode, checker: &mut Checker) {
    // TODO: Add rules for BlockLocalVariableNode
}
