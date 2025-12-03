#![allow(unused_variables)]
use crate::checker::Checker;
use ruby_prism::InterpolatedRegularExpressionNode;

/// Run lint rules over a [`InterpolatedRegularExpressionNode`] syntax node.
pub(crate) fn interpolated_regular_expression_node(node: &InterpolatedRegularExpressionNode, checker: &mut Checker) {
    // TODO: Add rules for InterpolatedRegularExpressionNode
}
