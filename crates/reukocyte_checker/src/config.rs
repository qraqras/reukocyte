mod base;
pub mod layout;
pub mod lint;
mod loader;
mod macros;
pub(crate) mod serde_helpers;
mod yaml;
pub use base::BaseCopConfig;
pub use layout::*;
pub use loader::{LoadError, load_rubocop_yaml, parse_rubocop_yaml};
pub use yaml::{AllCopsConfig, InheritFrom, RubocopYaml};

/// The main configuration struct.
#[derive(Debug, Clone, Default)]
pub struct Config {
    pub all_cops: AllCopsConfig,
    pub layout: layout::LayoutConfig,
    pub lint: lint::LintConfig,
}
