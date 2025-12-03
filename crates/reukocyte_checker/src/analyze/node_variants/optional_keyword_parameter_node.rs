#![allow(unused_variables)]
use crate::checker::Checker;
use ruby_prism::OptionalKeywordParameterNode;

/// Run lint rules over a [`OptionalKeywordParameterNode`] syntax node.
pub(crate) fn optional_keyword_parameter_node(node: &OptionalKeywordParameterNode, checker: &mut Checker) {
    // TODO: Add rules for OptionalKeywordParameterNode
}
