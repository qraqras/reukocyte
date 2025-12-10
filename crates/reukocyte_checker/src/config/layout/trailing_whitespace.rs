use crate::config::BaseCopConfig;
use serde::Deserialize;

/// Configuration for Layout/TrailingWhitespace.
#[derive(Debug, Clone, Deserialize)]
#[serde(default, rename_all = "PascalCase")]
pub struct TrailingWhitespace {
    /// Base configuration (enabled, severity, exclude, include).
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exclude_parsing() {
        let yaml = r#"
Exclude:
  - "test.rb"
  - "vendor/**/*"
"#;
        let config: TrailingWhitespace = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(config.base.exclude.len(), 2);
        assert_eq!(config.base.exclude[0], "test.rb");
    }
}
