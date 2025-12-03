#![allow(unused_variables)]
use crate::checker::Checker;
use ruby_prism::RegularExpressionNode;

/// Run lint rules over a [`RegularExpressionNode`] syntax node.
pub(crate) fn regular_expression_node(node: &RegularExpressionNode, checker: &mut Checker) {
    // TODO: Add rules for RegularExpressionNode
}
