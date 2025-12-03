#![allow(unused_variables)]
use crate::checker::Checker;
use crate::rules;
use ruby_prism::CallNode;

/// Run lint rules over a [`CallNode`] syntax node.
pub(crate) fn call_node(node: &CallNode, checker: &mut Checker) {
    // Lint/Debugger - detect debugger statements
    rules::lint::debugger::check(checker, node);
}
