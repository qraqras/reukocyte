#![allow(unused_variables)]
use crate::checker::Checker;
use ruby_prism::ImplicitRestNode;

/// Run lint rules over a [`ImplicitRestNode`] syntax node.
pub(crate) fn implicit_rest_node(node: &ImplicitRestNode, checker: &mut Checker) {
    // TODO: Add rules for ImplicitRestNode
}
