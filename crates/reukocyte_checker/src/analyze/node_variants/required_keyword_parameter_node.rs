#![allow(unused_variables)]
use crate::checker::Checker;
use ruby_prism::RequiredKeywordParameterNode;

/// Run lint rules over a [`RequiredKeywordParameterNode`] syntax node.
pub(crate) fn required_keyword_parameter_node(node: &RequiredKeywordParameterNode, checker: &mut Checker) {
    // TODO: Add rules for RequiredKeywordParameterNode
}
