#![allow(unused_variables)]
use crate::checker::Checker;
use ruby_prism::KeywordRestParameterNode;

/// Run lint rules over a [`KeywordRestParameterNode`] syntax node.
pub(crate) fn keyword_rest_parameter_node(node: &KeywordRestParameterNode, checker: &mut Checker) {
    // TODO: Add rules for KeywordRestParameterNode
}
