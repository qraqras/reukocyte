#![allow(unused_variables)]
use crate::checker::Checker;
use ruby_prism::ReturnNode;

/// Run lint rules over a [`ReturnNode`] syntax node.
pub(crate) fn return_node(node: &ReturnNode, checker: &mut Checker) {
    // TODO: Add rules for ReturnNode
}
