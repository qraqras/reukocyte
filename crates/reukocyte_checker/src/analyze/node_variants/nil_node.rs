#![allow(unused_variables)]
use crate::checker::Checker;
use ruby_prism::NilNode;

/// Run lint rules over a [`NilNode`] syntax node.
pub(crate) fn nil_node(node: &NilNode, checker: &mut Checker) {
    // TODO: Add rules for NilNode
}
