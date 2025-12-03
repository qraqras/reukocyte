#![allow(unused_variables)]
use crate::checker::Checker;
use ruby_prism::MissingNode;

/// Run lint rules over a [`MissingNode`] syntax node.
pub(crate) fn missing_node(node: &MissingNode, checker: &mut Checker) {
    // TODO: Add rules for MissingNode
}
