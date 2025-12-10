use reukocyte_checker::Diagnostic;
use rustc_hash::FxHashMap;
use serde::Serialize;

/// RuboCop-compatible JSON output format.
#[derive(Debug, Serialize)]
pub struct JsonOutput {
    pub metadata: Metadata,
    pub files: Vec<FileOffenses>,
    pub summary: Summary,
}

/// Metadata about the inspection.
/// FIXME: Reukocyte does not depend on Ruby, so these values are placeholders.
#[derive(Debug, Serialize)]
pub struct Metadata {
    pub rubocop_version: String,
    pub ruby_engine: String,
    pub ruby_version: String,
    pub ruby_patchlevel: String,
    pub ruby_platform: String,
}
impl Default for Metadata {
    fn default() -> Self {
        Self {
            rubocop_version: format!("rueko {}", env!("CARGO_PKG_VERSION")),
            ruby_engine: "ruby".to_string(),
            ruby_version: "3.0.0".to_string(), // Placeholder
            ruby_patchlevel: "0".to_string(),
            ruby_platform: std::env::consts::OS.to_string(),
        }
    }
}

/// Offenses for a single file.
#[derive(Debug, Serialize)]
pub struct FileOffenses {
    pub path: String,
    pub offenses: Vec<Offense>,
}

/// A single offense.
#[derive(Debug, Serialize)]
pub struct Offense {
    pub severity: String,
    pub message: String,
    pub cop_name: String,
    pub corrected: bool,
    pub correctable: bool,
    pub location: Location,
}

/// Location of an offense.
#[derive(Debug, Serialize)]
pub struct Location {
    pub start_line: usize,
    pub start_column: usize,
    pub last_line: usize,
    pub last_column: usize,
    pub length: usize,
    pub line: usize,
    pub column: usize,
}

/// Summary statistics.
#[derive(Debug, Serialize)]
pub struct Summary {
    pub offense_count: usize,
    pub target_file_count: usize,
    pub inspected_file_count: usize,
}

impl JsonOutput {
    /// Create a new JSON output from inspection results.
    pub fn new(file_results: FxHashMap<String, Vec<Diagnostic>>, corrected_counts: FxHashMap<String, usize>) -> Self {
        let mut total_offenses = 0;
        let mut files = Vec::new();
        for (path, diagnostics) in &file_results {
            let corrected_count = corrected_counts.get(path).copied().unwrap_or(0);
            let offenses: Vec<Offense> = diagnostics
                .iter()
                .map(|d| Offense {
                    severity: d.severity.as_str().to_string(),
                    message: d.message.clone(),
                    cop_name: d.rule().to_string(),
                    corrected: false, // Already remaining offenses
                    correctable: d.fix.is_some(),
                    location: Location {
                        start_line: d.line_start,
                        start_column: d.column_start,
                        last_line: d.line_end,
                        last_column: d.column_end,
                        length: d.end.saturating_sub(d.start),
                        line: d.line_start,
                        column: d.column_start,
                    },
                })
                .collect();

            total_offenses += offenses.len() + corrected_count;

            files.push(FileOffenses { path: path.clone(), offenses });
        }

        // Sort files by path for deterministic output
        files.sort_by(|a, b| a.path.cmp(&b.path));

        Self {
            metadata: Metadata::default(),
            files,
            summary: Summary {
                offense_count: total_offenses,
                target_file_count: file_results.len(),
                inspected_file_count: file_results.len(),
            },
        }
    }

    /// Serialize to JSON string.
    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).unwrap_or_else(|_| "{}".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_json_output_empty() {
        let output = JsonOutput::new(FxHashMap::default(), FxHashMap::default());
        assert_eq!(output.summary.offense_count, 0);
        assert!(output.files.is_empty());
    }
}
