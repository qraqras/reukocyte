#![allow(unused_variables)]
use crate::checker::Checker;
use ruby_prism::XStringNode;

/// Run lint rules over a [`XStringNode`] syntax node.
pub(crate) fn x_string_node(node: &XStringNode, checker: &mut Checker) {
    // TODO: Add rules for XStringNode
}
