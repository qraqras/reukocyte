#![allow(unused_variables)]
use crate::checker::Checker;
use ruby_prism::OptionalParameterNode;

/// Run lint rules over a [`OptionalParameterNode`] syntax node.
pub(crate) fn optional_parameter_node(node: &OptionalParameterNode, checker: &mut Checker) {
    // TODO: Add rules for OptionalParameterNode
}
