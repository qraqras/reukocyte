#![allow(unused_variables)]
use crate::checker::Checker;
use ruby_prism::MatchLastLineNode;

/// Run lint rules over a [`MatchLastLineNode`] syntax node.
pub(crate) fn match_last_line_node(node: &MatchLastLineNode, checker: &mut Checker) {
    // TODO: Add rules for MatchLastLineNode
}
