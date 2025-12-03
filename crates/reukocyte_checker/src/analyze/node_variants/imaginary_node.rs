#![allow(unused_variables)]
use crate::checker::Checker;
use ruby_prism::ImaginaryNode;

/// Run lint rules over a [`ImaginaryNode`] syntax node.
pub(crate) fn imaginary_node(node: &ImaginaryNode, checker: &mut Checker) {
    // TODO: Add rules for ImaginaryNode
}
