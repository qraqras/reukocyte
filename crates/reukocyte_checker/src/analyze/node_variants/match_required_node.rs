#![allow(unused_variables)]
use crate::checker::Checker;
use ruby_prism::MatchRequiredNode;

/// Run lint rules over a [`MatchRequiredNode`] syntax node.
pub(crate) fn match_required_node(node: &MatchRequiredNode, checker: &mut Checker) {
    // TODO: Add rules for MatchRequiredNode
}
