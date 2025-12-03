#![allow(unused_variables)]
use crate::checker::Checker;
use ruby_prism::FlipFlopNode;

/// Run lint rules over a [`FlipFlopNode`] syntax node.
pub(crate) fn flip_flop_node(node: &FlipFlopNode, checker: &mut Checker) {
    // TODO: Add rules for FlipFlopNode
}
