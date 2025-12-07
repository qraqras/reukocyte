//! Assignment node wrapper for unified handling of all assignment types.
//!
//! This module provides an enum wrapper that unifies all assignment node types,
//! allowing rules to handle them with common logic.

use ruby_prism::*;

/// A wrapper enum for all assignment node types.
///
/// This allows rules to implement `Check<AssignmentNode<'_>>` once
/// instead of implementing for each individual assignment type.
#[derive(Debug, Clone, Copy)]
pub enum AssignmentNode<'a> {
    CallAndWrite(&'a CallAndWriteNode<'a>),
    CallOperatorWrite(&'a CallOperatorWriteNode<'a>),
    CallOrWrite(&'a CallOrWriteNode<'a>),
    ClassVariableAndWrite(&'a ClassVariableAndWriteNode<'a>),
    ClassVariableOperatorWrite(&'a ClassVariableOperatorWriteNode<'a>),
    ClassVariableOrWrite(&'a ClassVariableOrWriteNode<'a>),
    ClassVariableWrite(&'a ClassVariableWriteNode<'a>),
    ConstantAndWrite(&'a ConstantAndWriteNode<'a>),
    ConstantOperatorWrite(&'a ConstantOperatorWriteNode<'a>),
    ConstantOrWrite(&'a ConstantOrWriteNode<'a>),
    ConstantPathAndWrite(&'a ConstantPathAndWriteNode<'a>),
    ConstantPathOperatorWrite(&'a ConstantPathOperatorWriteNode<'a>),
    ConstantPathOrWrite(&'a ConstantPathOrWriteNode<'a>),
    ConstantPathWrite(&'a ConstantPathWriteNode<'a>),
    ConstantWrite(&'a ConstantWriteNode<'a>),
    GlobalVariableAndWrite(&'a GlobalVariableAndWriteNode<'a>),
    GlobalVariableOperatorWrite(&'a GlobalVariableOperatorWriteNode<'a>),
    GlobalVariableOrWrite(&'a GlobalVariableOrWriteNode<'a>),
    GlobalVariableWrite(&'a GlobalVariableWriteNode<'a>),
    IndexAndWrite(&'a IndexAndWriteNode<'a>),
    IndexOperatorWrite(&'a IndexOperatorWriteNode<'a>),
    IndexOrWrite(&'a IndexOrWriteNode<'a>),
    InstanceVariableAndWrite(&'a InstanceVariableAndWriteNode<'a>),
    InstanceVariableOperatorWrite(&'a InstanceVariableOperatorWriteNode<'a>),
    InstanceVariableOrWrite(&'a InstanceVariableOrWriteNode<'a>),
    InstanceVariableWrite(&'a InstanceVariableWriteNode<'a>),
    LocalVariableAndWrite(&'a LocalVariableAndWriteNode<'a>),
    LocalVariableOperatorWrite(&'a LocalVariableOperatorWriteNode<'a>),
    LocalVariableOrWrite(&'a LocalVariableOrWriteNode<'a>),
    LocalVariableWrite(&'a LocalVariableWriteNode<'a>),
    MatchWrite(&'a MatchWriteNode<'a>),
    MultiWrite(&'a MultiWriteNode<'a>),
}

impl<'a> AssignmentNode<'a> {
    /// Get the location of the entire assignment node.
    pub fn location(&self) -> Location<'a> {
        match self {
            Self::CallAndWrite(n) => n.location(),
            Self::CallOperatorWrite(n) => n.location(),
            Self::CallOrWrite(n) => n.location(),
            Self::ClassVariableAndWrite(n) => n.location(),
            Self::ClassVariableOperatorWrite(n) => n.location(),
            Self::ClassVariableOrWrite(n) => n.location(),
            Self::ClassVariableWrite(n) => n.location(),
            Self::ConstantAndWrite(n) => n.location(),
            Self::ConstantOperatorWrite(n) => n.location(),
            Self::ConstantOrWrite(n) => n.location(),
            Self::ConstantPathAndWrite(n) => n.location(),
            Self::ConstantPathOperatorWrite(n) => n.location(),
            Self::ConstantPathOrWrite(n) => n.location(),
            Self::ConstantPathWrite(n) => n.location(),
            Self::ConstantWrite(n) => n.location(),
            Self::GlobalVariableAndWrite(n) => n.location(),
            Self::GlobalVariableOperatorWrite(n) => n.location(),
            Self::GlobalVariableOrWrite(n) => n.location(),
            Self::GlobalVariableWrite(n) => n.location(),
            Self::IndexAndWrite(n) => n.location(),
            Self::IndexOperatorWrite(n) => n.location(),
            Self::IndexOrWrite(n) => n.location(),
            Self::InstanceVariableAndWrite(n) => n.location(),
            Self::InstanceVariableOperatorWrite(n) => n.location(),
            Self::InstanceVariableOrWrite(n) => n.location(),
            Self::InstanceVariableWrite(n) => n.location(),
            Self::LocalVariableAndWrite(n) => n.location(),
            Self::LocalVariableOperatorWrite(n) => n.location(),
            Self::LocalVariableOrWrite(n) => n.location(),
            Self::LocalVariableWrite(n) => n.location(),
            Self::MatchWrite(n) => n.location(),
            Self::MultiWrite(n) => n.location(),
        }
    }

    /// Get the operator location (e.g., `=`, `+=`, `||=`).
    ///
    /// Returns `None` for `MatchWrite` which doesn't have an operator.
    pub fn operator_loc(&self) -> Option<Location<'a>> {
        match self {
            Self::CallAndWrite(n) => Some(n.operator_loc()),
            Self::CallOperatorWrite(n) => Some(n.binary_operator_loc()),
            Self::CallOrWrite(n) => Some(n.operator_loc()),
            Self::ClassVariableAndWrite(n) => Some(n.operator_loc()),
            Self::ClassVariableOperatorWrite(n) => Some(n.binary_operator_loc()),
            Self::ClassVariableOrWrite(n) => Some(n.operator_loc()),
            Self::ClassVariableWrite(n) => Some(n.operator_loc()),
            Self::ConstantAndWrite(n) => Some(n.operator_loc()),
            Self::ConstantOperatorWrite(n) => Some(n.binary_operator_loc()),
            Self::ConstantOrWrite(n) => Some(n.operator_loc()),
            Self::ConstantPathAndWrite(n) => Some(n.operator_loc()),
            Self::ConstantPathOperatorWrite(n) => Some(n.binary_operator_loc()),
            Self::ConstantPathOrWrite(n) => Some(n.operator_loc()),
            Self::ConstantPathWrite(n) => Some(n.operator_loc()),
            Self::ConstantWrite(n) => Some(n.operator_loc()),
            Self::GlobalVariableAndWrite(n) => Some(n.operator_loc()),
            Self::GlobalVariableOperatorWrite(n) => Some(n.binary_operator_loc()),
            Self::GlobalVariableOrWrite(n) => Some(n.operator_loc()),
            Self::GlobalVariableWrite(n) => Some(n.operator_loc()),
            Self::IndexAndWrite(n) => Some(n.operator_loc()),
            Self::IndexOperatorWrite(n) => Some(n.binary_operator_loc()),
            Self::IndexOrWrite(n) => Some(n.operator_loc()),
            Self::InstanceVariableAndWrite(n) => Some(n.operator_loc()),
            Self::InstanceVariableOperatorWrite(n) => Some(n.binary_operator_loc()),
            Self::InstanceVariableOrWrite(n) => Some(n.operator_loc()),
            Self::InstanceVariableWrite(n) => Some(n.operator_loc()),
            Self::LocalVariableAndWrite(n) => Some(n.operator_loc()),
            Self::LocalVariableOperatorWrite(n) => Some(n.binary_operator_loc()),
            Self::LocalVariableOrWrite(n) => Some(n.operator_loc()),
            Self::LocalVariableWrite(n) => Some(n.operator_loc()),
            Self::MatchWrite(_) => None,
            Self::MultiWrite(n) => Some(n.operator_loc()),
        }
    }

    /// Get the value (right-hand side) of the assignment.
    pub fn value(&self) -> Node<'a> {
        match self {
            Self::CallAndWrite(n) => n.value(),
            Self::CallOperatorWrite(n) => n.value(),
            Self::CallOrWrite(n) => n.value(),
            Self::ClassVariableAndWrite(n) => n.value(),
            Self::ClassVariableOperatorWrite(n) => n.value(),
            Self::ClassVariableOrWrite(n) => n.value(),
            Self::ClassVariableWrite(n) => n.value(),
            Self::ConstantAndWrite(n) => n.value(),
            Self::ConstantOperatorWrite(n) => n.value(),
            Self::ConstantOrWrite(n) => n.value(),
            Self::ConstantPathAndWrite(n) => n.value(),
            Self::ConstantPathOperatorWrite(n) => n.value(),
            Self::ConstantPathOrWrite(n) => n.value(),
            Self::ConstantPathWrite(n) => n.value(),
            Self::ConstantWrite(n) => n.value(),
            Self::GlobalVariableAndWrite(n) => n.value(),
            Self::GlobalVariableOperatorWrite(n) => n.value(),
            Self::GlobalVariableOrWrite(n) => n.value(),
            Self::GlobalVariableWrite(n) => n.value(),
            Self::IndexAndWrite(n) => n.value(),
            Self::IndexOperatorWrite(n) => n.value(),
            Self::IndexOrWrite(n) => n.value(),
            Self::InstanceVariableAndWrite(n) => n.value(),
            Self::InstanceVariableOperatorWrite(n) => n.value(),
            Self::InstanceVariableOrWrite(n) => n.value(),
            Self::InstanceVariableWrite(n) => n.value(),
            Self::LocalVariableAndWrite(n) => n.value(),
            Self::LocalVariableOperatorWrite(n) => n.value(),
            Self::LocalVariableOrWrite(n) => n.value(),
            Self::LocalVariableWrite(n) => n.value(),
            Self::MatchWrite(n) => n.call().as_node(),
            Self::MultiWrite(n) => n.value(),
        }
    }
}

// ============================================================================
// From implementations for easy conversion
// ============================================================================

impl<'a> From<&'a CallAndWriteNode<'a>> for AssignmentNode<'a> {
    fn from(node: &'a CallAndWriteNode<'a>) -> Self {
        Self::CallAndWrite(node)
    }
}

impl<'a> From<&'a CallOperatorWriteNode<'a>> for AssignmentNode<'a> {
    fn from(node: &'a CallOperatorWriteNode<'a>) -> Self {
        Self::CallOperatorWrite(node)
    }
}

impl<'a> From<&'a CallOrWriteNode<'a>> for AssignmentNode<'a> {
    fn from(node: &'a CallOrWriteNode<'a>) -> Self {
        Self::CallOrWrite(node)
    }
}

impl<'a> From<&'a ClassVariableAndWriteNode<'a>> for AssignmentNode<'a> {
    fn from(node: &'a ClassVariableAndWriteNode<'a>) -> Self {
        Self::ClassVariableAndWrite(node)
    }
}

impl<'a> From<&'a ClassVariableOperatorWriteNode<'a>> for AssignmentNode<'a> {
    fn from(node: &'a ClassVariableOperatorWriteNode<'a>) -> Self {
        Self::ClassVariableOperatorWrite(node)
    }
}

impl<'a> From<&'a ClassVariableOrWriteNode<'a>> for AssignmentNode<'a> {
    fn from(node: &'a ClassVariableOrWriteNode<'a>) -> Self {
        Self::ClassVariableOrWrite(node)
    }
}

impl<'a> From<&'a ClassVariableWriteNode<'a>> for AssignmentNode<'a> {
    fn from(node: &'a ClassVariableWriteNode<'a>) -> Self {
        Self::ClassVariableWrite(node)
    }
}

impl<'a> From<&'a ConstantAndWriteNode<'a>> for AssignmentNode<'a> {
    fn from(node: &'a ConstantAndWriteNode<'a>) -> Self {
        Self::ConstantAndWrite(node)
    }
}

impl<'a> From<&'a ConstantOperatorWriteNode<'a>> for AssignmentNode<'a> {
    fn from(node: &'a ConstantOperatorWriteNode<'a>) -> Self {
        Self::ConstantOperatorWrite(node)
    }
}

impl<'a> From<&'a ConstantOrWriteNode<'a>> for AssignmentNode<'a> {
    fn from(node: &'a ConstantOrWriteNode<'a>) -> Self {
        Self::ConstantOrWrite(node)
    }
}

impl<'a> From<&'a ConstantPathAndWriteNode<'a>> for AssignmentNode<'a> {
    fn from(node: &'a ConstantPathAndWriteNode<'a>) -> Self {
        Self::ConstantPathAndWrite(node)
    }
}

impl<'a> From<&'a ConstantPathOperatorWriteNode<'a>> for AssignmentNode<'a> {
    fn from(node: &'a ConstantPathOperatorWriteNode<'a>) -> Self {
        Self::ConstantPathOperatorWrite(node)
    }
}

impl<'a> From<&'a ConstantPathOrWriteNode<'a>> for AssignmentNode<'a> {
    fn from(node: &'a ConstantPathOrWriteNode<'a>) -> Self {
        Self::ConstantPathOrWrite(node)
    }
}

impl<'a> From<&'a ConstantPathWriteNode<'a>> for AssignmentNode<'a> {
    fn from(node: &'a ConstantPathWriteNode<'a>) -> Self {
        Self::ConstantPathWrite(node)
    }
}

impl<'a> From<&'a ConstantWriteNode<'a>> for AssignmentNode<'a> {
    fn from(node: &'a ConstantWriteNode<'a>) -> Self {
        Self::ConstantWrite(node)
    }
}

impl<'a> From<&'a GlobalVariableAndWriteNode<'a>> for AssignmentNode<'a> {
    fn from(node: &'a GlobalVariableAndWriteNode<'a>) -> Self {
        Self::GlobalVariableAndWrite(node)
    }
}

impl<'a> From<&'a GlobalVariableOperatorWriteNode<'a>> for AssignmentNode<'a> {
    fn from(node: &'a GlobalVariableOperatorWriteNode<'a>) -> Self {
        Self::GlobalVariableOperatorWrite(node)
    }
}

impl<'a> From<&'a GlobalVariableOrWriteNode<'a>> for AssignmentNode<'a> {
    fn from(node: &'a GlobalVariableOrWriteNode<'a>) -> Self {
        Self::GlobalVariableOrWrite(node)
    }
}

impl<'a> From<&'a GlobalVariableWriteNode<'a>> for AssignmentNode<'a> {
    fn from(node: &'a GlobalVariableWriteNode<'a>) -> Self {
        Self::GlobalVariableWrite(node)
    }
}

impl<'a> From<&'a IndexAndWriteNode<'a>> for AssignmentNode<'a> {
    fn from(node: &'a IndexAndWriteNode<'a>) -> Self {
        Self::IndexAndWrite(node)
    }
}

impl<'a> From<&'a IndexOperatorWriteNode<'a>> for AssignmentNode<'a> {
    fn from(node: &'a IndexOperatorWriteNode<'a>) -> Self {
        Self::IndexOperatorWrite(node)
    }
}

impl<'a> From<&'a IndexOrWriteNode<'a>> for AssignmentNode<'a> {
    fn from(node: &'a IndexOrWriteNode<'a>) -> Self {
        Self::IndexOrWrite(node)
    }
}

impl<'a> From<&'a InstanceVariableAndWriteNode<'a>> for AssignmentNode<'a> {
    fn from(node: &'a InstanceVariableAndWriteNode<'a>) -> Self {
        Self::InstanceVariableAndWrite(node)
    }
}

impl<'a> From<&'a InstanceVariableOperatorWriteNode<'a>> for AssignmentNode<'a> {
    fn from(node: &'a InstanceVariableOperatorWriteNode<'a>) -> Self {
        Self::InstanceVariableOperatorWrite(node)
    }
}

impl<'a> From<&'a InstanceVariableOrWriteNode<'a>> for AssignmentNode<'a> {
    fn from(node: &'a InstanceVariableOrWriteNode<'a>) -> Self {
        Self::InstanceVariableOrWrite(node)
    }
}

impl<'a> From<&'a InstanceVariableWriteNode<'a>> for AssignmentNode<'a> {
    fn from(node: &'a InstanceVariableWriteNode<'a>) -> Self {
        Self::InstanceVariableWrite(node)
    }
}

impl<'a> From<&'a LocalVariableAndWriteNode<'a>> for AssignmentNode<'a> {
    fn from(node: &'a LocalVariableAndWriteNode<'a>) -> Self {
        Self::LocalVariableAndWrite(node)
    }
}

impl<'a> From<&'a LocalVariableOperatorWriteNode<'a>> for AssignmentNode<'a> {
    fn from(node: &'a LocalVariableOperatorWriteNode<'a>) -> Self {
        Self::LocalVariableOperatorWrite(node)
    }
}

impl<'a> From<&'a LocalVariableOrWriteNode<'a>> for AssignmentNode<'a> {
    fn from(node: &'a LocalVariableOrWriteNode<'a>) -> Self {
        Self::LocalVariableOrWrite(node)
    }
}

impl<'a> From<&'a LocalVariableWriteNode<'a>> for AssignmentNode<'a> {
    fn from(node: &'a LocalVariableWriteNode<'a>) -> Self {
        Self::LocalVariableWrite(node)
    }
}

impl<'a> From<&'a MatchWriteNode<'a>> for AssignmentNode<'a> {
    fn from(node: &'a MatchWriteNode<'a>) -> Self {
        Self::MatchWrite(node)
    }
}

impl<'a> From<&'a MultiWriteNode<'a>> for AssignmentNode<'a> {
    fn from(node: &'a MultiWriteNode<'a>) -> Self {
        Self::MultiWrite(node)
    }
}
