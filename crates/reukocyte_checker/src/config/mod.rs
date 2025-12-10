pub mod layout;
pub mod lint;
mod loader;
pub(crate) mod serde_helpers;
mod yaml;

pub use layout::*;
pub use loader::{LoadError, load_rubocop_yaml, parse_rubocop_yaml};
pub use yaml::{AllCopsConfig, InheritFrom, RubocopYaml};

/// The main configuration struct.
#[derive(Debug, Clone, Default)]
pub struct Config {
    /// Layout cop-specific configurations.
    pub layout: layout::LayoutConfig,
    /// Lint cop-specific configurations.
    pub lint: lint::LintConfig,
}

impl Config {
    /// Create a Config from a parsed RubocopYaml.
    ///
    /// This simply transfers the already-deserialized cop configurations
    /// from RubocopYaml to the internal Config structure.
    pub fn from_rubocop_yaml(yaml: &RubocopYaml) -> Self {
        Config {
            layout: layout::LayoutConfig {
                access_modifier_indentation: yaml.layout_access_modifier_indentation.clone(),
                begin_end_alignment: yaml.layout_begin_end_alignment.clone(),
                def_end_alignment: yaml.layout_def_end_alignment.clone(),
                empty_lines: yaml.layout_empty_lines.clone(),
                end_alignment: yaml.layout_end_alignment.clone(),
                indentation_consistency: yaml.layout_indentation_consistency.clone(),
                indentation_style: yaml.layout_indentation_style.clone(),
                indentation_width: yaml.layout_indentation_width.clone(),
                leading_empty_lines: yaml.layout_leading_empty_lines.clone(),
                trailing_empty_lines: yaml.layout_trailing_empty_lines.clone(),
                trailing_whitespace: yaml.layout_trailing_whitespace.clone(),
            },
            lint: lint::LintConfig {
                debugger: yaml.lint_debugger.clone(),
            },
        }
    }
}
