#![allow(unused_variables)]
use crate::checker::Checker;
use ruby_prism::DefNode;

/// Run lint rules over a [`DefNode`] syntax node.
pub(crate) fn def_node(node: &DefNode, checker: &mut Checker) {
    // TODO: Add rules for DefNode
}
