//! Serde helpers for deserializing RuboCop YAML configuration.

use crate::diagnostic::Severity;
use serde::Deserialize;

/// Deserialize the `Enabled` field which can be a bool or string like "pending".
pub fn deserialize_enabled<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: serde::Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum EnabledValue {
        Bool(bool),
        String(String),
    }

    match EnabledValue::deserialize(deserializer)? {
        EnabledValue::Bool(b) => Ok(b),
        EnabledValue::String(s) => Ok(s.to_lowercase() != "false"),
    }
}

/// Deserialize the `Severity` field from a string.
pub fn deserialize_severity<'de, D>(deserializer: D) -> Result<Severity, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    Ok(parse_severity(&s))
}

/// Parse severity from string.
pub fn parse_severity(s: &str) -> Severity {
    match s.to_lowercase().as_str() {
        "refactor" | "r" => Severity::Refactor,
        "convention" | "c" => Severity::Convention,
        "warning" | "w" => Severity::Warning,
        "error" | "e" => Severity::Error,
        "fatal" | "f" => Severity::Fatal,
        _ => Severity::Warning,
    }
}
