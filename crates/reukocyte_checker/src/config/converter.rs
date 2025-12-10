//! Converter from RubocopYaml to internal Config format.
//!
//! This module handles the transformation from RuboCop's YAML configuration
//! format to Reukocyte's internal Config structs.

use super::layout::{
    access_modifier_indentation::{
        AccessModifierIndentationConfig, EnforcedStyle as AccessModifierStyle,
    },
    begin_end_alignment::{BeginEndAlignmentConfig, EnforcedStyleAlignWith as BeginEndStyle},
    def_end_alignment::{DefEndAlignmentConfig, EnforcedStyleAlignWith as DefEndStyle},
    empty_lines::EmptyLinesConfig,
    end_alignment::{EndAlignmentConfig, EnforcedStyleAlignWith as EndAlignmentStyle},
    indentation_consistency::{
        EnforcedStyle as IndentationConsistencyStyle, IndentationConsistencyConfig,
    },
    indentation_style::{EnforcedStyle as IndentationStyleStyle, IndentationStyleConfig},
    indentation_width::IndentationWidthConfig,
    leading_empty_lines::LeadingEmptyLinesConfig,
    trailing_empty_lines::{EnforcedStyle as TrailingEmptyLinesStyle, TrailingEmptyLinesConfig},
    trailing_whitespace::TrailingWhitespaceConfig,
};
use super::lint::debugger::DebuggerConfig;
use super::yaml::{CopConfig, RubocopYaml};
use super::Config;
use crate::diagnostic::Severity;

impl Config {
    /// Create a Config from a parsed RubocopYaml.
    ///
    /// Unknown cops and settings are silently ignored.
    pub fn from_rubocop_yaml(yaml: &RubocopYaml) -> Self {
        let mut config = Config::default();

        for (cop_name, cop_config) in &yaml.cops {
            // Apply cop-specific settings (including enabled/severity)
            match cop_name.as_str() {
                // Layout cops
                "Layout/EndAlignment" => {
                    apply_end_alignment(&mut config.layout.end_alignment, cop_config);
                }
                "Layout/IndentationWidth" => {
                    apply_indentation_width(&mut config.layout.indentation_width, cop_config);
                }
                "Layout/AccessModifierIndentation" => {
                    apply_access_modifier_indentation(
                        &mut config.layout.access_modifier_indentation,
                        cop_config,
                    );
                }
                "Layout/BeginEndAlignment" => {
                    apply_begin_end_alignment(&mut config.layout.begin_end_alignment, cop_config);
                }
                "Layout/DefEndAlignment" => {
                    apply_def_end_alignment(&mut config.layout.def_end_alignment, cop_config);
                }
                "Layout/IndentationConsistency" => {
                    apply_indentation_consistency(
                        &mut config.layout.indentation_consistency,
                        cop_config,
                    );
                }
                "Layout/IndentationStyle" => {
                    apply_indentation_style(&mut config.layout.indentation_style, cop_config);
                }
                "Layout/TrailingWhitespace" => {
                    apply_trailing_whitespace(&mut config.layout.trailing_whitespace, cop_config);
                }
                "Layout/TrailingEmptyLines" => {
                    apply_trailing_empty_lines(&mut config.layout.trailing_empty_lines, cop_config);
                }
                "Layout/LeadingEmptyLines" => {
                    apply_leading_empty_lines(&mut config.layout.leading_empty_lines, cop_config);
                }
                "Layout/EmptyLines" => {
                    apply_empty_lines(&mut config.layout.empty_lines, cop_config);
                }
                // Lint cops
                "Lint/Debugger" => {
                    apply_debugger(&mut config.lint.debugger, cop_config);
                }
                // Unknown cops are silently ignored
                _ => {}
            }
        }

        config
    }
}

/// Apply base configuration (enabled, severity) to a config struct.
fn apply_base_config<T: HasBaseConfig>(config: &mut T, cop: &CopConfig) {
    if let Some(enabled_val) = &cop.enabled {
        config.set_enabled(enabled_val.is_enabled());
    }
    if let Some(severity_str) = &cop.severity {
        if let Some(severity) = parse_severity(severity_str) {
            config.set_severity(severity);
        }
    }
}

/// Trait for config structs that have enabled/severity fields.
trait HasBaseConfig {
    fn set_enabled(&mut self, enabled: bool);
    fn set_severity(&mut self, severity: Severity);
}

// Implement HasBaseConfig for all config structs
impl HasBaseConfig for EndAlignmentConfig {
    fn set_enabled(&mut self, enabled: bool) { self.enabled = enabled; }
    fn set_severity(&mut self, severity: Severity) { self.severity = severity; }
}
impl HasBaseConfig for IndentationWidthConfig {
    fn set_enabled(&mut self, enabled: bool) { self.enabled = enabled; }
    fn set_severity(&mut self, severity: Severity) { self.severity = severity; }
}
impl HasBaseConfig for AccessModifierIndentationConfig {
    fn set_enabled(&mut self, enabled: bool) { self.enabled = enabled; }
    fn set_severity(&mut self, severity: Severity) { self.severity = severity; }
}
impl HasBaseConfig for BeginEndAlignmentConfig {
    fn set_enabled(&mut self, enabled: bool) { self.enabled = enabled; }
    fn set_severity(&mut self, severity: Severity) { self.severity = severity; }
}
impl HasBaseConfig for DefEndAlignmentConfig {
    fn set_enabled(&mut self, enabled: bool) { self.enabled = enabled; }
    fn set_severity(&mut self, severity: Severity) { self.severity = severity; }
}
impl HasBaseConfig for IndentationConsistencyConfig {
    fn set_enabled(&mut self, enabled: bool) { self.enabled = enabled; }
    fn set_severity(&mut self, severity: Severity) { self.severity = severity; }
}
impl HasBaseConfig for IndentationStyleConfig {
    fn set_enabled(&mut self, enabled: bool) { self.enabled = enabled; }
    fn set_severity(&mut self, severity: Severity) { self.severity = severity; }
}
impl HasBaseConfig for TrailingWhitespaceConfig {
    fn set_enabled(&mut self, enabled: bool) { self.enabled = enabled; }
    fn set_severity(&mut self, severity: Severity) { self.severity = severity; }
}
impl HasBaseConfig for TrailingEmptyLinesConfig {
    fn set_enabled(&mut self, enabled: bool) { self.enabled = enabled; }
    fn set_severity(&mut self, severity: Severity) { self.severity = severity; }
}
impl HasBaseConfig for LeadingEmptyLinesConfig {
    fn set_enabled(&mut self, enabled: bool) { self.enabled = enabled; }
    fn set_severity(&mut self, severity: Severity) { self.severity = severity; }
}
impl HasBaseConfig for EmptyLinesConfig {
    fn set_enabled(&mut self, enabled: bool) { self.enabled = enabled; }
    fn set_severity(&mut self, severity: Severity) { self.severity = severity; }
}
impl HasBaseConfig for DebuggerConfig {
    fn set_enabled(&mut self, enabled: bool) { self.enabled = enabled; }
    fn set_severity(&mut self, severity: Severity) { self.severity = severity; }
}

/// Helper to get a string value from cop options.
fn get_string_option(cop: &CopConfig, key: &str) -> Option<String> {
    cop.options.get(key).and_then(|v| match v {
        serde_yaml::Value::String(s) => Some(s.clone()),
        _ => None,
    })
}

/// Helper to get an integer value from cop options.
fn get_int_option(cop: &CopConfig, key: &str) -> Option<i64> {
    cop.options.get(key).and_then(|v| match v {
        serde_yaml::Value::Number(n) => n.as_i64(),
        _ => None,
    })
}

/// Parse severity from string.
fn parse_severity(s: &str) -> Option<Severity> {
    match s.to_lowercase().as_str() {
        "error" | "fatal" => Some(Severity::Error),
        "warning" | "warn" => Some(Severity::Warning),
        "convention" => Some(Severity::Convention),
        "info" | "refactor" => Some(Severity::Convention), // Map to convention
        _ => None,
    }
}

// ============================================================================
// Cop-specific configuration appliers
// ============================================================================

fn apply_end_alignment(config: &mut EndAlignmentConfig, cop: &CopConfig) {
    apply_base_config(config, cop);
    if let Some(style) = get_string_option(cop, "EnforcedStyleAlignWith") {
        config.enforced_style_align_with = match style.to_lowercase().as_str() {
            "keyword" => EndAlignmentStyle::Keyword,
            "variable" => EndAlignmentStyle::Variable,
            "start_of_line" => EndAlignmentStyle::StartOfLine,
            _ => config.enforced_style_align_with, // Keep default on unknown value
        };
    }
}

fn apply_indentation_width(config: &mut IndentationWidthConfig, cop: &CopConfig) {
    apply_base_config(config, cop);
    if let Some(width) = get_int_option(cop, "Width") {
        config.width = width as i32;
    }
}

fn apply_access_modifier_indentation(
    config: &mut AccessModifierIndentationConfig,
    cop: &CopConfig,
) {
    apply_base_config(config, cop);
    if let Some(style) = get_string_option(cop, "EnforcedStyle") {
        config.enforced_style = match style.to_lowercase().as_str() {
            "indent" => AccessModifierStyle::Indent,
            "outdent" => AccessModifierStyle::Outdent,
            _ => config.enforced_style,
        };
    }
    if let Some(width) = get_int_option(cop, "IndentationWidth") {
        config.indentation_width = Some(width as usize);
    }
}

fn apply_begin_end_alignment(config: &mut BeginEndAlignmentConfig, cop: &CopConfig) {
    apply_base_config(config, cop);
    if let Some(style) = get_string_option(cop, "EnforcedStyleAlignWith") {
        config.enforced_style_align_with = match style.to_lowercase().as_str() {
            "start_of_line" => BeginEndStyle::StartOfLine,
            "begin" => BeginEndStyle::Begin,
            _ => config.enforced_style_align_with,
        };
    }
}

fn apply_def_end_alignment(config: &mut DefEndAlignmentConfig, cop: &CopConfig) {
    apply_base_config(config, cop);
    if let Some(style) = get_string_option(cop, "EnforcedStyleAlignWith") {
        config.enforced_style_align_with = match style.to_lowercase().as_str() {
            "start_of_line" => DefEndStyle::StartOfLine,
            "def" => DefEndStyle::Def,
            _ => config.enforced_style_align_with,
        };
    }
}

fn apply_indentation_consistency(config: &mut IndentationConsistencyConfig, cop: &CopConfig) {
    apply_base_config(config, cop);
    if let Some(style) = get_string_option(cop, "EnforcedStyle") {
        config.enforced_style = match style.to_lowercase().replace(' ', "_").as_str() {
            "normal" => IndentationConsistencyStyle::Normal,
            "indented_internal_methods" => IndentationConsistencyStyle::IndentedInternalMethods,
            _ => config.enforced_style,
        };
    }
}

fn apply_indentation_style(config: &mut IndentationStyleConfig, cop: &CopConfig) {
    apply_base_config(config, cop);
    if let Some(style) = get_string_option(cop, "EnforcedStyle") {
        config.enforced_style = match style.to_lowercase().as_str() {
            "spaces" => IndentationStyleStyle::Spaces,
            "tabs" => IndentationStyleStyle::Tabs,
            _ => config.enforced_style,
        };
    }
    if let Some(width) = get_int_option(cop, "IndentationWidth") {
        config.indentation_width = width as usize;
    }
}

fn apply_trailing_whitespace(config: &mut TrailingWhitespaceConfig, cop: &CopConfig) {
    apply_base_config(config, cop);
    // No cop-specific options for TrailingWhitespace
}

fn apply_trailing_empty_lines(config: &mut TrailingEmptyLinesConfig, cop: &CopConfig) {
    apply_base_config(config, cop);
    if let Some(style) = get_string_option(cop, "EnforcedStyle") {
        config.enforced_style = match style.to_lowercase().as_str() {
            "final_newline" => TrailingEmptyLinesStyle::FinalNewline,
            "final_blank_line" => TrailingEmptyLinesStyle::FinalBlankLine,
            _ => config.enforced_style,
        };
    }
}

fn apply_leading_empty_lines(config: &mut LeadingEmptyLinesConfig, cop: &CopConfig) {
    apply_base_config(config, cop);
    // No cop-specific options for LeadingEmptyLines
}

fn apply_empty_lines(config: &mut EmptyLinesConfig, cop: &CopConfig) {
    apply_base_config(config, cop);
    // No cop-specific options for EmptyLines
}

fn apply_debugger(config: &mut DebuggerConfig, cop: &CopConfig) {
    apply_base_config(config, cop);
    // No cop-specific options for Debugger
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::parse_rubocop_yaml;

    #[test]
    fn test_end_alignment_keyword() {
        let yaml = r#"
Layout/EndAlignment:
  EnforcedStyleAlignWith: keyword
"#;
        let rubocop = parse_rubocop_yaml(yaml).unwrap();
        let config = Config::from_rubocop_yaml(&rubocop);

        assert_eq!(
            config.layout.end_alignment.enforced_style_align_with,
            EndAlignmentStyle::Keyword
        );
    }

    #[test]
    fn test_end_alignment_variable() {
        let yaml = r#"
Layout/EndAlignment:
  EnforcedStyleAlignWith: variable
"#;
        let rubocop = parse_rubocop_yaml(yaml).unwrap();
        let config = Config::from_rubocop_yaml(&rubocop);

        assert_eq!(
            config.layout.end_alignment.enforced_style_align_with,
            EndAlignmentStyle::Variable
        );
    }

    #[test]
    fn test_indentation_width() {
        let yaml = r#"
Layout/IndentationWidth:
  Width: 4
"#;
        let rubocop = parse_rubocop_yaml(yaml).unwrap();
        let config = Config::from_rubocop_yaml(&rubocop);

        assert_eq!(config.layout.indentation_width.width, 4);
    }

    #[test]
    fn test_access_modifier_indentation() {
        let yaml = r#"
Layout/AccessModifierIndentation:
  EnforcedStyle: outdent
  IndentationWidth: 4
"#;
        let rubocop = parse_rubocop_yaml(yaml).unwrap();
        let config = Config::from_rubocop_yaml(&rubocop);

        assert_eq!(
            config.layout.access_modifier_indentation.enforced_style,
            AccessModifierStyle::Outdent
        );
        assert_eq!(
            config.layout.access_modifier_indentation.indentation_width,
            Some(4)
        );
    }

    #[test]
    fn test_begin_end_alignment() {
        let yaml = r#"
Layout/BeginEndAlignment:
  EnforcedStyleAlignWith: begin
  Severity: error
"#;
        let rubocop = parse_rubocop_yaml(yaml).unwrap();
        let config = Config::from_rubocop_yaml(&rubocop);

        assert_eq!(
            config.layout.begin_end_alignment.enforced_style_align_with,
            BeginEndStyle::Begin
        );
        assert_eq!(config.layout.begin_end_alignment.severity, Severity::Error);
    }

    #[test]
    fn test_indentation_consistency() {
        let yaml = r#"
Layout/IndentationConsistency:
  EnforcedStyle: indented_internal_methods
"#;
        let rubocop = parse_rubocop_yaml(yaml).unwrap();
        let config = Config::from_rubocop_yaml(&rubocop);

        assert_eq!(
            config.layout.indentation_consistency.enforced_style,
            IndentationConsistencyStyle::IndentedInternalMethods
        );
    }

    #[test]
    fn test_unknown_cop_ignored() {
        let yaml = r#"
Layout/UnknownCop:
  SomeSetting: value
Lint/UnknownLint:
  Enabled: false
"#;
        let rubocop = parse_rubocop_yaml(yaml).unwrap();
        // Should not panic, just return default config
        let config = Config::from_rubocop_yaml(&rubocop);

        // Defaults should be preserved
        assert_eq!(
            config.layout.end_alignment.enforced_style_align_with,
            EndAlignmentStyle::Keyword
        );
    }

    #[test]
    fn test_multiple_cops() {
        let yaml = r#"
Layout/EndAlignment:
  EnforcedStyleAlignWith: variable
Layout/IndentationWidth:
  Width: 4
Layout/AccessModifierIndentation:
  EnforcedStyle: outdent
"#;
        let rubocop = parse_rubocop_yaml(yaml).unwrap();
        let config = Config::from_rubocop_yaml(&rubocop);

        assert_eq!(
            config.layout.end_alignment.enforced_style_align_with,
            EndAlignmentStyle::Variable
        );
        assert_eq!(config.layout.indentation_width.width, 4);
        assert_eq!(
            config.layout.access_modifier_indentation.enforced_style,
            AccessModifierStyle::Outdent
        );
    }

    #[test]
    fn test_enabled_false() {
        let yaml = r#"
Layout/DefEndAlignment:
  Enabled: false
"#;
        let rubocop = parse_rubocop_yaml(yaml).unwrap();
        let config = Config::from_rubocop_yaml(&rubocop);

        assert!(!config.layout.def_end_alignment.enabled);
    }

    #[test]
    fn test_severity_override() {
        let yaml = r#"
Layout/EndAlignment:
  Severity: error
"#;
        let rubocop = parse_rubocop_yaml(yaml).unwrap();
        let config = Config::from_rubocop_yaml(&rubocop);

        assert_eq!(config.layout.end_alignment.severity, Severity::Error);
    }
}
