use crate::config::BaseCopConfig;
use serde::Deserialize;

/// Configuration for Layout/TrailingWhitespace.
#[derive(Debug, Clone, Deserialize)]
#[serde(default, rename_all = "PascalCase")]
pub struct TrailingWhitespace {
    #[serde(flatten)]
    pub base: BaseCopConfig,
}
impl Default for TrailingWhitespace {
    fn default() -> Self {
        Self {
            base: BaseCopConfig::default(),
        }
    }
}
