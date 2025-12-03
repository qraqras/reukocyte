#![allow(unused_variables)]
use crate::checker::Checker;
use ruby_prism::HashPatternNode;

/// Run lint rules over a [`HashPatternNode`] syntax node.
pub(crate) fn hash_pattern_node(node: &HashPatternNode, checker: &mut Checker) {
    // TODO: Add rules for HashPatternNode
}
