#![allow(unused_variables)]
use crate::checker::Checker;
use ruby_prism::SourceFileNode;

/// Run lint rules over a [`SourceFileNode`] syntax node.
pub(crate) fn source_file_node(node: &SourceFileNode, checker: &mut Checker) {
    // TODO: Add rules for SourceFileNode
}
