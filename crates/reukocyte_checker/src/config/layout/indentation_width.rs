use crate::config::BaseCopConfig;
use serde::Deserialize;

/// Configuration for Layout/IndentationWidth.
#[derive(Debug, Clone, Deserialize)]
#[serde(default, rename_all = "PascalCase")]
pub struct IndentationWidth {
    #[serde(flatten)]
    pub base: BaseCopConfig,
    pub allowed_patterns: Vec<i32>,
    pub width: i32,
}
impl Default for IndentationWidth {
    fn default() -> Self {
        Self {
            base: BaseCopConfig::default(),
            allowed_patterns: Vec::new(),
            width: 2,
        }
    }
}
