#![allow(unused_variables)]
use crate::checker::Checker;
use ruby_prism::InterpolatedStringNode;

/// Run lint rules over a [`InterpolatedStringNode`] syntax node.
pub(crate) fn interpolated_string_node(node: &InterpolatedStringNode, checker: &mut Checker) {
    // TODO: Add rules for InterpolatedStringNode
}
