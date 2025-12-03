#![allow(unused_variables)]
use crate::checker::Checker;
use ruby_prism::PinnedExpressionNode;

/// Run lint rules over a [`PinnedExpressionNode`] syntax node.
pub(crate) fn pinned_expression_node(node: &PinnedExpressionNode, checker: &mut Checker) {
    // TODO: Add rules for PinnedExpressionNode
}
