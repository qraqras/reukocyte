//! Cop trait definition

use crate::offense::Offense;
use std::cell::RefCell;

/// Context provided to cops for checking
pub struct CheckContext<'a> {
    /// The source code being checked
    pub source: &'a str,

    /// The file path being checked
    pub file_path: &'a str,
}

/// Trait that all cops must implement
pub trait Cop: Send + Sync {
    /// Returns the name of the cop (e.g., "Layout/TrailingWhitespace")
    fn name(&self) -> &'static str;

    /// Check the source code and return any offenses found.
    /// This is the main entry point for cops that don't need AST.
    fn check(&self, context: &CheckContext) -> Vec<Offense>;

    /// Returns true if this cop requires AST for checking
    fn requires_ast(&self) -> bool {
        false
    }

    /// Returns true if this cop can auto-fix offenses
    fn supports_autocorrect(&self) -> bool {
        false
    }

    /// Auto-fix the source code and return the corrected version
    fn autocorrect(&self, source: &str) -> String {
        source.to_string()
    }
}

/// A cop that uses the Visitor pattern to traverse the AST.
/// Cops that need AST should implement this trait.
pub trait AstCop: Send + Sync {
    /// Returns the name of the cop (e.g., "Layout/IndentationWidth")
    fn name(&self) -> &'static str;

    /// Returns the collected offenses after visiting the AST
    fn offenses(&self) -> Vec<Offense>;

    /// Clear collected offenses (called before each file)
    fn clear(&self);

    /// Returns true if this cop can auto-fix offenses
    fn supports_autocorrect(&self) -> bool {
        false
    }

    /// Auto-fix the source code and return the corrected version
    fn autocorrect(&self, source: &str) -> String {
        source.to_string()
    }
}

/// Helper struct to collect offenses during AST traversal
#[derive(Debug, Default)]
pub struct OffenseCollector {
    offenses: RefCell<Vec<Offense>>,
    file_path: RefCell<String>,
}

impl OffenseCollector {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_file_path(&self, path: &str) {
        *self.file_path.borrow_mut() = path.to_string();
    }

    pub fn file_path(&self) -> String {
        self.file_path.borrow().clone()
    }

    pub fn add_offense(&self, offense: Offense) {
        self.offenses.borrow_mut().push(offense);
    }

    pub fn offenses(&self) -> Vec<Offense> {
        self.offenses.borrow().clone()
    }

    pub fn clear(&self) {
        self.offenses.borrow_mut().clear();
    }
}
