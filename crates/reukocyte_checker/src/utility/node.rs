use ruby_prism::Node;

use crate::custom_nodes::ConditionalNode;

pub fn is_assignment(node: &Node) -> bool {
    match node {
        Node::CallAndWriteNode { .. } => true,
        Node::CallOperatorWriteNode { .. } => true,
        Node::CallOrWriteNode { .. } => true,
        Node::ClassVariableAndWriteNode { .. } => true,
        Node::ClassVariableOperatorWriteNode { .. } => true,
        Node::ClassVariableOrWriteNode { .. } => true,
        Node::ClassVariableWriteNode { .. } => true,
        Node::ConstantAndWriteNode { .. } => true,
        Node::ConstantOperatorWriteNode { .. } => true,
        Node::ConstantOrWriteNode { .. } => true,
        Node::ConstantPathAndWriteNode { .. } => true,
        Node::ConstantPathOperatorWriteNode { .. } => true,
        Node::ConstantPathOrWriteNode { .. } => true,
        Node::ConstantPathWriteNode { .. } => true,
        Node::ConstantWriteNode { .. } => true,
        Node::GlobalVariableAndWriteNode { .. } => true,
        Node::GlobalVariableOperatorWriteNode { .. } => true,
        Node::GlobalVariableOrWriteNode { .. } => true,
        Node::GlobalVariableWriteNode { .. } => true,
        Node::IndexAndWriteNode { .. } => true,
        Node::IndexOperatorWriteNode { .. } => true,
        Node::IndexOrWriteNode { .. } => true,
        Node::InstanceVariableAndWriteNode { .. } => true,
        Node::InstanceVariableOperatorWriteNode { .. } => true,
        Node::InstanceVariableOrWriteNode { .. } => true,
        Node::InstanceVariableWriteNode { .. } => true,
        Node::LocalVariableAndWriteNode { .. } => true,
        Node::LocalVariableOperatorWriteNode { .. } => true,
        Node::LocalVariableOrWriteNode { .. } => true,
        Node::LocalVariableWriteNode { .. } => true,
        Node::MatchWriteNode { .. } => true,
        Node::MultiWriteNode { .. } => true,
        _ => false,
    }
}

pub fn is_conditional(node: &Node) -> bool {
    match node {
        Node::CaseMatchNode { .. } => true,
        Node::CaseNode { .. } => true,
        Node::IfNode { .. } => true,
        Node::UnlessNode { .. } => true,
        Node::UntilNode { .. } => true,
        Node::WhileNode { .. } => true,
        _ => false,
    }
}

pub fn if_conditional_node<F>(node: &Node, f: F)
where
    F: FnOnce(&ConditionalNode),
{
    match node {
        Node::CaseMatchNode { .. } => f(&ConditionalNode::CaseMatchNode(&node.as_case_match_node().unwrap())),
        Node::CaseNode { .. } => f(&ConditionalNode::CaseNode(&node.as_case_node().unwrap())),
        Node::IfNode { .. } => f(&ConditionalNode::IfNode(&node.as_if_node().unwrap())),
        Node::UnlessNode { .. } => f(&ConditionalNode::UnlessNode(&node.as_unless_node().unwrap())),
        Node::UntilNode { .. } => f(&ConditionalNode::UntilNode(&node.as_until_node().unwrap())),
        Node::WhileNode { .. } => f(&ConditionalNode::WhileNode(&node.as_while_node().unwrap())),
        _ => {}
    }
}
