#![allow(unused_variables)]
use crate::checker::Checker;
use ruby_prism::WhileNode;

/// Run lint rules over a [`WhileNode`] syntax node.
pub(crate) fn while_node(node: &WhileNode, checker: &mut Checker) {
    // TODO: Add rules for WhileNode
}
