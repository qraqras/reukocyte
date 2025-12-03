#![allow(unused_variables)]
use crate::checker::Checker;
use ruby_prism::InterpolatedXStringNode;

/// Run lint rules over a [`InterpolatedXStringNode`] syntax node.
pub(crate) fn interpolated_x_string_node(node: &InterpolatedXStringNode, checker: &mut Checker) {
    // TODO: Add rules for InterpolatedXStringNode
}
