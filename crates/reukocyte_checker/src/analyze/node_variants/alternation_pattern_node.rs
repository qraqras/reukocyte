#![allow(unused_variables)]
use crate::checker::Checker;
use ruby_prism::AlternationPatternNode;

/// Run lint rules over a [`AlternationPatternNode`] syntax node.
pub(crate) fn alternation_pattern_node(node: &AlternationPatternNode, checker: &mut Checker) {
    // TODO: Add rules for AlternationPatternNode
}
