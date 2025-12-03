#![allow(unused_variables)]
use crate::checker::Checker;
use ruby_prism::IfNode;

/// Run lint rules over a [`IfNode`] syntax node.
pub(crate) fn if_node(node: &IfNode, checker: &mut Checker) {
    // TODO: Add rules for IfNode
}
