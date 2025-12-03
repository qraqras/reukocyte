#![allow(unused_variables)]
use crate::checker::Checker;
use ruby_prism::FindPatternNode;

/// Run lint rules over a [`FindPatternNode`] syntax node.
pub(crate) fn find_pattern_node(node: &FindPatternNode, checker: &mut Checker) {
    // TODO: Add rules for FindPatternNode
}
