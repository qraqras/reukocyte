//! YAML intermediate representation for .rubocop.yml parsing.
//!
//! This module defines the raw YAML structure that maps directly to RuboCop's
//! configuration format. Each cop's configuration is parsed directly using
//! serde's `#[serde(rename)]` attribute, eliminating the need for manual conversion.

use super::layout::{
    AccessModifierIndentationConfig, BeginEndAlignmentConfig, DefEndAlignmentConfig, EmptyLinesConfig, EndAlignmentConfig, IndentationConsistencyConfig,
    IndentationStyleConfig, IndentationWidthConfig, LeadingEmptyLinesConfig, TrailingEmptyLinesConfig, TrailingWhitespaceConfig,
};
use super::lint::DebuggerConfig;
use serde::Deserialize;
use std::path::PathBuf;

/// Root structure of a .rubocop.yml file.
///
/// RuboCop YAML files have a flat structure where each top-level key is either:
/// - A special key like `inherit_from`, `AllCops`, etc.
/// - A cop name like `Layout/EndAlignment`, `Lint/Debugger`
///
/// Each cop configuration is directly deserialized using `#[serde(rename)]`.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct RubocopYaml {
    /// Files to inherit configuration from.
    /// Can be a single file or a list of files.
    #[serde(default)]
    pub inherit_from: InheritFrom,

    /// Global settings that apply to all cops.
    #[serde(rename = "AllCops", default)]
    pub all_cops: AllCopsConfig,

    // ========================================================================
    // Layout cops
    // ========================================================================
    #[serde(rename = "Layout/AccessModifierIndentation", default)]
    pub layout_access_modifier_indentation: AccessModifierIndentationConfig,

    #[serde(rename = "Layout/BeginEndAlignment", default)]
    pub layout_begin_end_alignment: BeginEndAlignmentConfig,

    #[serde(rename = "Layout/DefEndAlignment", default)]
    pub layout_def_end_alignment: DefEndAlignmentConfig,

    #[serde(rename = "Layout/EmptyLines", default)]
    pub layout_empty_lines: EmptyLinesConfig,

    #[serde(rename = "Layout/EndAlignment", default)]
    pub layout_end_alignment: EndAlignmentConfig,

    #[serde(rename = "Layout/IndentationConsistency", default)]
    pub layout_indentation_consistency: IndentationConsistencyConfig,

    #[serde(rename = "Layout/IndentationStyle", default)]
    pub layout_indentation_style: IndentationStyleConfig,

    #[serde(rename = "Layout/IndentationWidth", default)]
    pub layout_indentation_width: IndentationWidthConfig,

    #[serde(rename = "Layout/LeadingEmptyLines", default)]
    pub layout_leading_empty_lines: LeadingEmptyLinesConfig,

    #[serde(rename = "Layout/TrailingEmptyLines", default)]
    pub layout_trailing_empty_lines: TrailingEmptyLinesConfig,

    #[serde(rename = "Layout/TrailingWhitespace", default)]
    pub layout_trailing_whitespace: TrailingWhitespaceConfig,

    // ========================================================================
    // Lint cops
    // ========================================================================
    #[serde(rename = "Lint/Debugger", default)]
    pub lint_debugger: DebuggerConfig,
}

/// The `inherit_from` field can be a single string or a list of strings.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(untagged)]
pub enum InheritFrom {
    #[default]
    None,
    Single(String),
    Multiple(Vec<String>),
}

impl InheritFrom {
    /// Convert to a list of paths.
    pub fn to_paths(&self) -> Vec<PathBuf> {
        match self {
            InheritFrom::None => vec![],
            InheritFrom::Single(s) => vec![PathBuf::from(s)],
            InheritFrom::Multiple(v) => v.iter().map(PathBuf::from).collect(),
        }
    }

    /// Check if there are any inherited files.
    pub fn is_empty(&self) -> bool {
        match self {
            InheritFrom::None => true,
            InheritFrom::Single(_) => false,
            InheritFrom::Multiple(v) => v.is_empty(),
        }
    }
}

/// Global configuration that applies to all cops.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct AllCopsConfig {
    /// Target Ruby version (e.g., "3.2", "3.3").
    #[serde(default)]
    pub target_ruby_version: Option<String>,

    /// Files to exclude from all cops.
    #[serde(default)]
    pub exclude: Vec<String>,

    /// Files to include for all cops.
    #[serde(default)]
    pub include: Vec<String>,

    /// Whether to use cache.
    #[serde(default)]
    pub use_cache: Option<bool>,

    /// Cache root directory.
    #[serde(default)]
    pub cache_root_directory: Option<String>,

    /// New cops behavior: enable, disable, or pending.
    #[serde(default)]
    pub new_cops: Option<String>,

    /// Suggested extensions.
    #[serde(default)]
    pub suggested_extensions: Option<bool>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::layout::end_alignment::EnforcedStyleAlignWith;

    #[test]
    fn test_parse_simple_yaml() {
        let yaml = r#"
Layout/EndAlignment:
  Enabled: true
  EnforcedStyleAlignWith: variable
"#;
        let config: RubocopYaml = serde_yaml::from_str(yaml).unwrap();
        assert!(config.layout_end_alignment.enabled);
        assert_eq!(config.layout_end_alignment.enforced_style_align_with, EnforcedStyleAlignWith::Variable);
    }

    #[test]
    fn test_parse_inherit_from_single() {
        let yaml = r#"
inherit_from: .rubocop_todo.yml
"#;
        let config: RubocopYaml = serde_yaml::from_str(yaml).unwrap();
        let paths = config.inherit_from.to_paths();
        assert_eq!(paths.len(), 1);
        assert_eq!(paths[0], PathBuf::from(".rubocop_todo.yml"));
    }

    #[test]
    fn test_parse_inherit_from_multiple() {
        let yaml = r#"
inherit_from:
  - .rubocop_todo.yml
  - .rubocop_custom.yml
"#;
        let config: RubocopYaml = serde_yaml::from_str(yaml).unwrap();
        let paths = config.inherit_from.to_paths();
        assert_eq!(paths.len(), 2);
    }

    #[test]
    fn test_parse_all_cops() {
        let yaml = r#"
AllCops:
  TargetRubyVersion: 3.2
  NewCops: enable
  Exclude:
    - 'vendor/**/*'
    - 'db/schema.rb'
"#;
        let config: RubocopYaml = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(config.all_cops.target_ruby_version, Some("3.2".to_string()));
        assert_eq!(config.all_cops.new_cops, Some("enable".to_string()));
        assert_eq!(config.all_cops.exclude.len(), 2);
    }

    #[test]
    fn test_parse_enabled_false() {
        let yaml = r#"
Layout/EndAlignment:
  Enabled: false
"#;
        let config: RubocopYaml = serde_yaml::from_str(yaml).unwrap();
        assert!(!config.layout_end_alignment.enabled);
    }

    #[test]
    fn test_parse_enabled_pending() {
        let yaml = r#"
Layout/EndAlignment:
  Enabled: pending
"#;
        let config: RubocopYaml = serde_yaml::from_str(yaml).unwrap();
        // "pending" is treated as enabled (not explicitly disabled)
        assert!(config.layout_end_alignment.enabled);
    }
}
