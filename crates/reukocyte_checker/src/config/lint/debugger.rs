use crate::config::BaseCopConfig;
use crate::diagnostic::Severity;
use serde::Deserialize;

/// Configuration for Lint/Debugger.
#[derive(Debug, Clone, Deserialize)]
#[serde(default, rename_all = "PascalCase")]
pub struct Debugger {
    #[serde(flatten)]
    pub base: BaseCopConfig,
}
impl Default for Debugger {
    fn default() -> Self {
        Self {
            base: BaseCopConfig::with_severity(Severity::Warning),
        }
    }
}
