#![allow(unused_variables)]
use crate::checker::Checker;
use ruby_prism::ImplicitNode;

/// Run lint rules over a [`ImplicitNode`] syntax node.
pub(crate) fn implicit_node(node: &ImplicitNode, checker: &mut Checker) {
    // TODO: Add rules for ImplicitNode
}
