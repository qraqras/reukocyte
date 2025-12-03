#![allow(unused_variables)]
use crate::checker::Checker;
use ruby_prism::MatchPredicateNode;

/// Run lint rules over a [`MatchPredicateNode`] syntax node.
pub(crate) fn match_predicate_node(node: &MatchPredicateNode, checker: &mut Checker) {
    // TODO: Add rules for MatchPredicateNode
}
