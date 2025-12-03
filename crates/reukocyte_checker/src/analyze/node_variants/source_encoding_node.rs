#![allow(unused_variables)]
use crate::checker::Checker;
use ruby_prism::SourceEncodingNode;

/// Run lint rules over a [`SourceEncodingNode`] syntax node.
pub(crate) fn source_encoding_node(node: &SourceEncodingNode, checker: &mut Checker) {
    // TODO: Add rules for SourceEncodingNode
}
