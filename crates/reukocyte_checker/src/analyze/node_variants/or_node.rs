#![allow(unused_variables)]
use crate::checker::Checker;
use ruby_prism::OrNode;

/// Run lint rules over a [`OrNode`] syntax node.
pub(crate) fn or_node(node: &OrNode, checker: &mut Checker) {
    // TODO: Add rules for OrNode
}
