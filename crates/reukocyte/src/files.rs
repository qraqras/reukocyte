use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Default file extensions to include (RuboCop compatible).
/// See: https://github.com/rubocop/rubocop/blob/master/config/default.yml
const RUBY_EXTENSIONS: &[&str] = &[
    "rb",
    "arb",
    "axlsx",
    "builder",
    "fcgi",
    "gemfile",
    "gemspec",
    "god",
    "jb",
    "jbuilder",
    "mspec",
    "opal",
    "pluginspec",
    "podspec",
    "rabl",
    "rake",
    "rbuild",
    "rbw",
    "rbx",
    "ru",
    "ruby",
    "schema",
    "spec",
    "thor",
    "watchr",
];

/// Default file names to include (RuboCop compatible).
/// See: https://github.com/rubocop/rubocop/blob/master/config/default.yml
const RUBY_FILENAMES: &[&str] = &[
    // **************** Dotfiles ****************
    ".irbrc",
    ".pryrc",
    ".simplecov",
    // **************** Build/Config files ****************
    "buildfile",
    "Appraisals",
    "Berksfile",
    "Brewfile",
    "Buildfile",
    "Capfile",
    "Cheffile",
    "Dangerfile",
    "Deliverfile",
    "Fastfile",
    "Gemfile",
    "Guardfile",
    "Jarfile",
    "Mavenfile",
    "Podfile",
    "Puppetfile",
    "Rakefile",
    "rakefile",
    "Schemafile",
    "Snapfile",
    "Steepfile",
    "Thorfile",
    "Vagabondfile",
    "Vagrantfile",
];

/// Directories to skip during traversal (RuboCop compatible defaults).
const EXCLUDED_DIRS: &[&str] = &[".git", "node_modules", "tmp", "vendor"];

/// Collect all Ruby files from the given paths.
///
/// This function handles:
/// - Single files (returned as-is if they are Ruby files)
/// - Directories (recursively walked for Ruby files)
///
/// Note: Does NOT respect .gitignore (RuboCop compatible).
/// Use .rubocop.yml Exclude patterns instead.
pub fn collect_ruby_files(paths: &[PathBuf]) -> Vec<PathBuf> {
    let mut files = Vec::new();
    for path in paths {
        if path.is_file() {
            if is_ruby_file(path) {
                files.push(path.clone());
            }
        } else if path.is_dir() {
            files.extend(walk_directory(path));
        }
    }
    files
}

/// Walk a directory and collect all Ruby files.
fn walk_directory(dir: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    // Use walkdir for simple recursive directory traversal
    // RuboCop does NOT respect .gitignore, so we don't either
    for entry in WalkDir::new(dir).follow_links(true).into_iter().filter_entry(|e| !is_excluded_dir(e)).flatten() {
        let path = entry.path();
        if path.is_file() && is_ruby_file(path) {
            files.push(path.to_path_buf());
        }
    }
    files
}

/// Check if a directory entry should be excluded.
fn is_excluded_dir(entry: &walkdir::DirEntry) -> bool {
    entry.file_type().is_dir() && entry.file_name().to_str().is_some_and(|s| EXCLUDED_DIRS.contains(&s))
}

/// Check if a file is a Ruby file based on extension or filename.
fn is_ruby_file(path: &Path) -> bool {
    // Check by extension
    if let Some(ext) = path.extension() {
        if let Some(ext_str) = ext.to_str() {
            if RUBY_EXTENSIONS.contains(&ext_str) {
                return true;
            }
        }
    }
    // Check by filename (for files like Gemfile, Rakefile, etc.)
    if let Some(filename) = path.file_name() {
        if let Some(filename_str) = filename.to_str() {
            if RUBY_FILENAMES.contains(&filename_str) {
                return true;
            }
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_ruby_file_by_extension() {
        // Common extensions
        assert!(is_ruby_file(Path::new("test.rb")));
        assert!(is_ruby_file(Path::new("path/to/file.rake")));
        assert!(is_ruby_file(Path::new("my_gem.gemspec")));
        // Additional RuboCop extensions
        assert!(is_ruby_file(Path::new("config.ru")));
        assert!(is_ruby_file(Path::new("view.jbuilder")));
        assert!(is_ruby_file(Path::new("template.builder")));
        assert!(is_ruby_file(Path::new("app.podspec")));
        // Non-Ruby files
        assert!(!is_ruby_file(Path::new("file.txt")));
        assert!(!is_ruby_file(Path::new("file.py")));
    }

    #[test]
    fn test_is_ruby_file_by_name() {
        // Common filenames
        assert!(is_ruby_file(Path::new("Gemfile")));
        assert!(is_ruby_file(Path::new("Rakefile")));
        assert!(is_ruby_file(Path::new("path/to/Gemfile")));
        // Additional RuboCop filenames
        assert!(is_ruby_file(Path::new("Dangerfile")));
        assert!(is_ruby_file(Path::new("Fastfile")));
        assert!(is_ruby_file(Path::new("Steepfile")));
        assert!(is_ruby_file(Path::new(".pryrc")));
        assert!(is_ruby_file(Path::new(".irbrc")));
        // Case sensitivity (rakefile vs Rakefile)
        assert!(is_ruby_file(Path::new("rakefile")));
        // Non-Ruby files
        assert!(!is_ruby_file(Path::new("Makefile")));
    }
}
