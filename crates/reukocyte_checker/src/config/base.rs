use crate::config::serde_helpers::deserialize_enabled;
use crate::config::serde_helpers::deserialize_severity;
use crate::diagnostic::Severity;
use globset::Glob;
use globset::GlobSet;
use globset::GlobSetBuilder;
use serde::Deserialize;

/// Base configuration fields shared by all cops.
///
/// This struct is meant to be used with `#[serde(flatten)]` in each config.
///
/// # Example
/// ```ignore
/// #[derive(Debug, Clone, Deserialize)]
/// #[serde(default, rename_all = "PascalCase")]
/// pub struct ConfigName {
///     #[serde(flatten)]
///     pub base: BaseCopConfig,
///     pub option: String,
/// }
/// ```
#[derive(Debug, Clone, Deserialize)]
#[serde(default, rename_all = "PascalCase")]
pub struct BaseCopConfig {
    #[serde(deserialize_with = "deserialize_enabled")]
    pub enabled: bool,
    #[serde(deserialize_with = "deserialize_severity")]
    pub severity: Severity,
    #[serde(default)]
    pub exclude: Vec<String>,
    #[serde(default)]
    pub include: Vec<String>,
    #[serde(skip)]
    pub(crate) include_glob: Option<GlobSet>,
    #[serde(skip)]
    pub(crate) exclude_glob: Option<GlobSet>,
}
impl Default for BaseCopConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            severity: Severity::Convention,
            exclude: Vec::new(),
            include: Vec::new(),
            include_glob: None,
            exclude_glob: None,
        }
    }
}
impl BaseCopConfig {
    /// Creates a new base config with the specified default severity.
    pub fn with_severity(severity: Severity) -> Self {
        Self {
            severity,
            ..Default::default()
        }
    }
    /// Compiles the include and exclude glob patterns.
    pub fn compile_globs(&mut self) {
        // helper to compile patterns
        fn compile(patterns: &Vec<String>) -> Option<GlobSet> {
            if patterns.is_empty() {
                None
            } else {
                let mut builder = GlobSetBuilder::new();
                for pattern in patterns {
                    if let Ok(glob) = Glob::new(pattern) {
                        builder.add(glob);
                    }
                }
                if let Ok(glob_set) = builder.build() { Some(glob_set) } else { None }
            }
        }
        // compile include
        self.include_glob = if self.include.is_empty() { None } else { compile(&self.include) };
        // compile exclude
        self.exclude_glob = if self.exclude.is_empty() { None } else { compile(&self.exclude) };
    }
}
