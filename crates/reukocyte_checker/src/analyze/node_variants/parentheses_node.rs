#![allow(unused_variables)]
use crate::checker::Checker;
use ruby_prism::ParenthesesNode;

/// Run lint rules over a [`ParenthesesNode`] syntax node.
pub(crate) fn parentheses_node(node: &ParenthesesNode, checker: &mut Checker) {
    // TODO: Add rules for ParenthesesNode
}
