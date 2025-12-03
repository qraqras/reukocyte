#![allow(unused_variables)]
use crate::checker::Checker;
use ruby_prism::MatchWriteNode;

/// Run lint rules over a [`MatchWriteNode`] syntax node.
pub(crate) fn match_write_node(node: &MatchWriteNode, checker: &mut Checker) {
    // TODO: Add rules for MatchWriteNode
}
