#![allow(unused_variables)]
use crate::checker::Checker;
use ruby_prism::ArrayPatternNode;

/// Run lint rules over a [`ArrayPatternNode`] syntax node.
pub(crate) fn array_pattern_node(node: &ArrayPatternNode, checker: &mut Checker) {
    // TODO: Add rules for ArrayPatternNode
}
