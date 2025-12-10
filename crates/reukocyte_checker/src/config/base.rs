//! Base configuration shared by all cops.

use crate::config::serde_helpers::{deserialize_enabled, deserialize_severity};
use crate::diagnostic::Severity;
use serde::Deserialize;

/// Base configuration fields shared by all cops.
///
/// This struct is meant to be used with `#[serde(flatten)]` in each cop's config.
///
/// # Example
/// ```ignore
/// #[derive(Debug, Clone, Deserialize)]
/// #[serde(default, rename_all = "PascalCase")]
/// pub struct MyCop {
///     #[serde(flatten)]
///     pub base: BaseCopConfig,
///     // Cop-specific fields below...
///     pub my_option: String,
/// }
/// ```
#[derive(Debug, Clone, Deserialize)]
#[serde(default, rename_all = "PascalCase")]
pub struct BaseCopConfig {
    /// Whether this cop is enabled.
    #[serde(deserialize_with = "deserialize_enabled")]
    pub enabled: bool,
    /// Severity level for this cop.
    #[serde(deserialize_with = "deserialize_severity")]
    pub severity: Severity,
    /// Files to exclude from this cop.
    #[serde(default)]
    pub exclude: Vec<String>,
    /// Files to include for this cop (cop only runs on matching files).
    #[serde(default)]
    pub include: Vec<String>,
}
impl Default for BaseCopConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            severity: Severity::Convention,
            exclude: Vec::new(),
            include: Vec::new(),
        }
    }
}

impl BaseCopConfig {
    /// Creates a new base config with the specified default severity.
    pub fn with_severity(severity: Severity) -> Self {
        Self {
            severity,
            ..Default::default()
        }
    }
}
