//! YAML file loader for .rubocop.yml configuration files.
//!
//! This module handles:
//! - Reading and parsing .rubocop.yml files
//! - Resolving `inherit_from` references
//! - Merging configurations (child overrides parent)

use std::collections::HashSet;
use std::io;
use std::path::{Path, PathBuf};

use super::yaml::{RubocopYaml, merge_configs};

/// Error type for configuration loading.
#[derive(Debug)]
pub enum LoadError {
    /// IO error when reading a file.
    Io(io::Error),
    /// YAML parsing error.
    Yaml(serde_yaml::Error),
    /// Circular inheritance detected.
    CircularInheritance(PathBuf),
}

impl std::fmt::Display for LoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LoadError::Io(e) => write!(f, "IO error: {}", e),
            LoadError::Yaml(e) => write!(f, "YAML parsing error: {}", e),
            LoadError::CircularInheritance(p) => {
                write!(f, "Circular inheritance detected: {}", p.display())
            }
        }
    }
}

impl std::error::Error for LoadError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            LoadError::Io(e) => Some(e),
            LoadError::Yaml(e) => Some(e),
            LoadError::CircularInheritance(_) => None,
        }
    }
}

impl From<io::Error> for LoadError {
    fn from(e: io::Error) -> Self {
        LoadError::Io(e)
    }
}

impl From<serde_yaml::Error> for LoadError {
    fn from(e: serde_yaml::Error) -> Self {
        LoadError::Yaml(e)
    }
}

/// Load a .rubocop.yml file and resolve all `inherit_from` references.
///
/// This function:
/// 1. Reads the specified YAML file
/// 2. Recursively loads all `inherit_from` files
/// 3. Merges configurations (later files override earlier ones)
///
/// # Arguments
/// * `path` - Path to the .rubocop.yml file
///
/// # Returns
/// * `Ok(RubocopYaml)` - The fully resolved configuration
/// * `Err(LoadError)` - If loading or parsing fails
pub fn load_rubocop_yaml(path: &Path) -> Result<RubocopYaml, LoadError> {
    let mut visited = HashSet::new();
    load_with_inheritance(path, &mut visited)
}

/// Load a .rubocop.yml file from a string (useful for testing).
pub fn parse_rubocop_yaml(content: &str) -> Result<RubocopYaml, LoadError> {
    Ok(serde_yaml::from_str(content)?)
}

/// Internal function that tracks visited files to detect circular inheritance.
fn load_with_inheritance(path: &Path, visited: &mut HashSet<PathBuf>) -> Result<RubocopYaml, LoadError> {
    // Canonicalize path to detect circular references
    let canonical = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());

    if visited.contains(&canonical) {
        return Err(LoadError::CircularInheritance(canonical));
    }
    visited.insert(canonical.clone());

    // Read and parse the YAML file
    let content = std::fs::read_to_string(path)?;
    let mut config: RubocopYaml = serde_yaml::from_str(&content)?;

    // Resolve inherit_from
    if !config.inherit_from.is_empty() {
        let base_dir = path.parent().unwrap_or(Path::new("."));
        let inherited_paths = config.inherit_from.to_paths();

        // Load inherited configs in order and merge
        for inherit_path in inherited_paths {
            let full_path = if inherit_path.is_absolute() {
                inherit_path
            } else {
                base_dir.join(&inherit_path)
            };

            // Skip if file doesn't exist (silent ignore)
            if !full_path.exists() {
                continue;
            }

            match load_with_inheritance(&full_path, visited) {
                Ok(parent_config) => {
                    // Merge: child overrides parent
                    config = merge_configs(parent_config, config);
                }
                Err(_) => {
                    // Silent ignore on error (per user request)
                    continue;
                }
            }
        }
    }

    Ok(config)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::layout::end_alignment::EnforcedStyleAlignWith;

    #[test]
    fn test_parse_simple() {
        let yaml = r#"
Layout/EndAlignment:
  EnforcedStyleAlignWith: variable
"#;
        let config = parse_rubocop_yaml(yaml).unwrap();
        assert_eq!(config.end_alignment.enforced_style_align_with, EnforcedStyleAlignWith::Variable);
    }

    #[test]
    fn test_merge_configs() {
        let parent_yaml = r#"
Layout/EndAlignment:
  Enabled: true
  EnforcedStyleAlignWith: keyword
"#;
        let child_yaml = r#"
Layout/EndAlignment:
  EnforcedStyleAlignWith: variable
"#;
        let parent = parse_rubocop_yaml(parent_yaml).unwrap();
        let child = parse_rubocop_yaml(child_yaml).unwrap();

        let merged = merge_configs(parent, child);

        // Enabled should still be true
        assert!(merged.end_alignment.enabled);

        // EnforcedStyleAlignWith should come from child
        assert_eq!(merged.end_alignment.enforced_style_align_with, EnforcedStyleAlignWith::Variable);
    }

    #[test]
    fn test_merge_all_cops() {
        let parent_yaml = r#"
AllCops:
  TargetRubyVersion: 3.1
  Exclude:
    - vendor/**/*
"#;
        let child_yaml = r#"
AllCops:
  TargetRubyVersion: 3.2
"#;
        let parent = parse_rubocop_yaml(parent_yaml).unwrap();
        let child = parse_rubocop_yaml(child_yaml).unwrap();

        let merged = merge_configs(parent, child);

        // Child's ruby version overrides parent
        assert_eq!(merged.all_cops.target_ruby_version, Some("3.2".to_string()));
        // Parent's exclude is kept (child didn't specify)
        assert_eq!(merged.all_cops.exclude.len(), 1);
    }

    #[test]
    fn test_enabled_false() {
        let yaml = r#"
Layout/EndAlignment:
  Enabled: false
"#;
        let config = parse_rubocop_yaml(yaml).unwrap();
        assert!(!config.end_alignment.enabled);
    }
}
