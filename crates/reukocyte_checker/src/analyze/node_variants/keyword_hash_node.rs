#![allow(unused_variables)]
use crate::checker::Checker;
use ruby_prism::KeywordHashNode;

/// Run lint rules over a [`KeywordHashNode`] syntax node.
pub(crate) fn keyword_hash_node(node: &KeywordHashNode, checker: &mut Checker) {
    // TODO: Add rules for KeywordHashNode
}
