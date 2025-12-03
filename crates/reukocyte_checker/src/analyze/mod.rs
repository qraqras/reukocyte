//! Analyze module - central dispatch for all lint rules.
//!
//! This module contains functions that run lint rules over AST nodes.
//! Inspired by Ruff's analyze module, this provides:
//! - Clear visibility of which rules run on which nodes
//! - Centralized rule dispatch
//! - Easy addition of new rules
//!
//! Each function corresponds to a node type and calls all relevant rules.

mod call;

pub(crate) use call::call_node;
