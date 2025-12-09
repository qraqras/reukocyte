use crate::checker::Checker;
use crate::semantic::NodeId;
use ruby_prism::*;

const PRIVATE: &[u8] = b"private";
const PROTECTED: &[u8] = b"protected";
const PUBLIC: &[u8] = b"public";
const MODULE_FUNCTION: &[u8] = b"module_function";

pub fn is_macro(node_id: &NodeId, checker: &Checker) -> bool {
    checker.semantic().nodes().get(*node_id).is_some_and(|node| match node.as_call_node() {
        Some(call_node) => call_node.receiver().is_none() && in_macro_scope(node_id, checker),
        None => false,
    })
}

pub fn is_access_modifier(node_id: &NodeId, checker: &Checker) -> bool {
    is_macro(node_id, checker) && (is_bare_access_modifier_declaration(node_id, checker) || is_non_bare_access_modifier_declaration(node_id, checker))
}

pub fn is_bare_access_modifier(node_id: &NodeId, checker: &Checker) -> bool {
    is_macro(node_id, checker) && is_bare_access_modifier_declaration(node_id, checker)
}

pub fn is_non_bare_access_modifier(node_id: &NodeId, checker: &Checker) -> bool {
    is_macro(node_id, checker) && is_non_bare_access_modifier_declaration(node_id, checker)
}

pub fn is_special_modifier(node_id: &NodeId, checker: &Checker) -> bool {
    checker.semantic().nodes().get(*node_id).is_some_and(|node| match node.as_call_node() {
        Some(call_node) => is_bare_access_modifier(node_id, checker) && (call_node.name().as_slice() == PRIVATE || call_node.name().as_slice() == PROTECTED),
        None => false,
    })
}

pub fn is_adjacent_def_modifier(node: &CallNode, _checker: &Checker) -> bool {
    node.receiver().is_none()
        && node
            .arguments()
            .is_some_and(|arguments| arguments.arguments().iter().next().is_some_and(|first| first.as_def_node().is_some()))
}

fn in_macro_scope(node_id: &NodeId, checker: &Checker) -> bool {
    let mut ancestors_iter = checker.semantic().nodes().ancestors(*node_id).skip(1).peekable();

    // If no ancestors, we're at module level
    if ancestors_iter.peek().is_none() {
        return true;
    }

    let mut in_statements = false;
    for ancestor in ancestors_iter {
        match ancestor {
            Node::ClassNode { .. } => return true,
            Node::ModuleNode { .. } => return true,
            Node::BeginNode { .. }
            | Node::BlockNode { .. }
            | Node::ElseNode { .. }
            | Node::EnsureNode { .. }
            | Node::IfNode { .. }
            | Node::LambdaNode { .. }
            | Node::RescueNode { .. }
            | Node::UnlessNode { .. } => {
                if in_statements {
                    in_statements = false;
                } else {
                    return false;
                }
            }
            Node::StatementsNode { .. } => {
                in_statements = true;
            }
            _ => return false,
        }
    }
    false
}

fn is_bare_access_modifier_declaration(node_id: &NodeId, checker: &Checker) -> bool {
    checker.semantic().nodes().get(*node_id).is_some_and(|node| match node.as_call_node() {
        Some(call_node) => {
            let name = call_node.name().as_slice();
            call_node.receiver().is_none()
                && call_node.arguments().is_none()
                && (name == PUBLIC || name == PROTECTED || name == PRIVATE || name == MODULE_FUNCTION)
        }
        None => false,
    })
}

fn is_non_bare_access_modifier_declaration(node_id: &NodeId, checker: &Checker) -> bool {
    checker.semantic().nodes().get(*node_id).is_some_and(|node| match node.as_call_node() {
        Some(call_node) => {
            let name = call_node.name().as_slice();
            call_node.receiver().is_none()
                && call_node.arguments().is_some()
                && (name == PUBLIC || name == PROTECTED || name == PRIVATE || name == MODULE_FUNCTION)
        }
        None => false,
    })
}
