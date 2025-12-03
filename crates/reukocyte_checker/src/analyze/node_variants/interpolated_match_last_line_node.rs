#![allow(unused_variables)]
use crate::checker::Checker;
use ruby_prism::InterpolatedMatchLastLineNode;

/// Run lint rules over a [`InterpolatedMatchLastLineNode`] syntax node.
pub(crate) fn interpolated_match_last_line_node(node: &InterpolatedMatchLastLineNode, checker: &mut Checker) {
    // TODO: Add rules for InterpolatedMatchLastLineNode
}
