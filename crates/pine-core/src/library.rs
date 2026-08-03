//! Resolving `import`ed libraries to their source.

use std::collections::HashMap;

/// Loads the source of a library by its import path.
pub trait LibraryLoader {
    fn load_library(&self, path: &str) -> Result<String, String>;
}

/// A [`LibraryLoader`] backed by an in-memory set of named sources: register a
/// library's text under a path and `import <path>` resolves against it.
#[derive(Debug, Default, Clone)]
pub struct FileResolver {
    files: HashMap<String, String>,
}

impl FileResolver {
    pub fn new() -> Self {
        Self::default()
    }

    /// Register `source` under `path` in place.
    pub fn add(&mut self, path: &str, source: &str) {
        self.files.insert(path.to_string(), source.to_string());
    }

    /// Register `source` under `path`, returning self for chaining.
    pub fn with_file(mut self, path: &str, source: &str) -> Self {
        self.add(path, source);
        self
    }
}

impl LibraryLoader for FileResolver {
    fn load_library(&self, path: &str) -> Result<String, String> {
        self.files
            .get(path)
            .cloned()
            .ok_or_else(|| format!("no library registered for '{path}'"))
    }
}
