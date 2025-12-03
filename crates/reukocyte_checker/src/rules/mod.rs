//! Rules for checking Ruby source code.
//!
//! Rules are organized by category:
//! - `layout`: Style/formatting rules (e.g., trailing whitespace)
//! - `lint`: Code quality rules (e.g., debugger detection)

pub mod layout;
pub mod lint;
