//! Runner for executing cops

use crate::cop::{CheckContext, Cop};
use crate::offense::Offense;
use rayon::prelude::*;
use std::path::Path;

/// Runner for executing cops on files
pub struct Runner {
    cops: Vec<Box<dyn Cop>>,
}

impl Runner {
    pub fn new() -> Self {
        Self { cops: Vec::new() }
    }

    /// Add a cop to the runner
    pub fn add_cop(&mut self, cop: Box<dyn Cop>) {
        self.cops.push(cop);
    }

    /// Run all cops on a single file
    pub fn run_file(&self, file_path: &Path) -> Result<Vec<Offense>, RunnerError> {
        let source = std::fs::read_to_string(file_path)?;
        let file_path_str = file_path.to_string_lossy();

        let context = CheckContext {
            source: &source,
            file_path: &file_path_str,
        };

        let offenses: Vec<Offense> = self
            .cops
            .iter()
            .flat_map(|cop| cop.check(&context))
            .collect();

        Ok(offenses)
    }

    /// Run all cops on multiple files in parallel
    pub fn run_files(&self, file_paths: &[&Path]) -> Vec<Result<Vec<Offense>, RunnerError>> {
        file_paths
            .par_iter()
            .map(|path| self.run_file(path))
            .collect()
    }
}

impl Default for Runner {
    fn default() -> Self {
        Self::new()
    }
}

/// Errors that can occur during running
#[derive(Debug, thiserror::Error)]
pub enum RunnerError {
    #[error("Failed to read file: {0}")]
    Io(#[from] std::io::Error),
}
