#![allow(unused_variables)]
use crate::checker::Checker;
use ruby_prism::ModuleNode;

/// Run lint rules over a [`ModuleNode`] syntax node.
pub(crate) fn module_node(node: &ModuleNode, checker: &mut Checker) {
    // TODO: Add rules for ModuleNode
}
