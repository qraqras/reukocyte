#![allow(unused_variables)]
use crate::checker::Checker;
use ruby_prism::NoKeywordsParameterNode;

/// Run lint rules over a [`NoKeywordsParameterNode`] syntax node.
pub(crate) fn no_keywords_parameter_node(node: &NoKeywordsParameterNode, checker: &mut Checker) {
    // TODO: Add rules for NoKeywordsParameterNode
}
