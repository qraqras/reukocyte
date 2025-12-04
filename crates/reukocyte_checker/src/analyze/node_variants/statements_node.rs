#![allow(unused_variables)]
use crate::checker::Checker;
use crate::rules::layout::indentation_width;
use ruby_prism::StatementsNode;

/// Run lint rules over a [`StatementsNode`] syntax node.
pub(crate) fn statements_node(node: &StatementsNode, checker: &mut Checker) {
    indentation_width::on_statements(node, false, checker);
}
