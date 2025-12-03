#![allow(unused_variables)]
use crate::checker::Checker;
use ruby_prism::DefinedNode;

/// Run lint rules over a [`DefinedNode`] syntax node.
pub(crate) fn defined_node(node: &DefinedNode, checker: &mut Checker) {
    // TODO: Add rules for DefinedNode
}
