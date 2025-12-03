#![allow(unused_variables)]
use crate::checker::Checker;
use ruby_prism::SourceLineNode;

/// Run lint rules over a [`SourceLineNode`] syntax node.
pub(crate) fn source_line_node(node: &SourceLineNode, checker: &mut Checker) {
    // TODO: Add rules for SourceLineNode
}
