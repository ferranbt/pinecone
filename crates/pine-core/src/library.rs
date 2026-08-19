//! Resolving `import`ed libraries to their source.

use std::cell::RefCell;
use std::collections::HashMap;
use std::path::PathBuf;

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

/// A [`LibraryLoader`] that reads libraries from directories on disk.
pub struct DirLoader {
    roots: Vec<PathBuf>,
    loaded: RefCell<HashMap<String, String>>,
}

impl DirLoader {
    pub fn new(roots: Vec<PathBuf>) -> Self {
        Self {
            roots,
            loaded: RefCell::new(HashMap::new()),
        }
    }

    /// The source of a library this loader already resolved, if any. Keyed by
    /// import path — the same string sema attributes its diagnostics to.
    pub fn source_of(&self, path: &str) -> Option<String> {
        self.loaded.borrow().get(path).cloned()
    }

    /// The on-disk file `path` resolves to, without reading it — for locating a
    /// library file (e.g. editor go-to-definition).
    pub fn resolve_path(&self, path: &str) -> Option<PathBuf> {
        self.candidates(path).into_iter().find(|c| c.is_file())
    }

    /// Every file `path` could name, most specific first.
    fn candidates(&self, path: &str) -> Vec<PathBuf> {
        let trimmed = path.trim_matches('/');
        let parts: Vec<&str> = trimmed.split('/').collect();

        let mut relative = vec![
            PathBuf::from(format!("{trimmed}.pine")),
            PathBuf::from(trimmed),
        ];
        // `<user>/<Name>/<version>`: also accept the library dropped in without
        // its version, or without its namespace entirely.
        if let [user, name, _version] = parts.as_slice() {
            relative.push(PathBuf::from(user).join(format!("{name}.pine")));
            relative.push(PathBuf::from(format!("{name}.pine")));
        }

        self.roots
            .iter()
            .flat_map(|root| relative.iter().map(|rel| root.join(rel)))
            .collect()
    }
}

impl LibraryLoader for DirLoader {
    fn load_library(&self, path: &str) -> Result<String, String> {
        if let Some(cached) = self.source_of(path) {
            return Ok(cached);
        }

        for candidate in self.candidates(path) {
            if !candidate.is_file() {
                continue;
            }
            let source = std::fs::read_to_string(&candidate)
                .map_err(|e| format!("{}: {e}", candidate.display()))?;
            self.loaded
                .borrow_mut()
                .insert(path.to_string(), source.clone());
            return Ok(source);
        }

        if self.roots.is_empty() {
            return Err("no library directory given (pass --lib <DIR>)".to_string());
        }
        Err(format!(
            "not found under {}",
            self.roots
                .iter()
                .map(|r| r.display().to_string())
                .collect::<Vec<_>>()
                .join(", ")
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};

    /// A unique temp directory that removes itself on drop.
    struct TempDir(PathBuf);

    impl TempDir {
        fn new() -> Self {
            static COUNTER: AtomicU32 = AtomicU32::new(0);
            let id = COUNTER.fetch_add(1, Ordering::Relaxed);
            let dir =
                std::env::temp_dir().join(format!("pine-dirloader-{}-{id}", std::process::id()));
            std::fs::create_dir_all(&dir).unwrap();
            TempDir(dir)
        }

        fn write(&self, rel: &str, contents: &str) {
            let path = self.0.join(rel);
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(path, contents).unwrap();
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn loads_a_pine_file_and_caches_it() {
        let dir = TempDir::new();
        dir.write("mylib.pine", "// lib source");
        let loader = DirLoader::new(vec![dir.0.clone()]);

        assert_eq!(loader.load_library("mylib").unwrap(), "// lib source");
        // A resolved source is cached under the import path.
        assert_eq!(loader.source_of("mylib").as_deref(), Some("// lib source"));
    }

    #[test]
    fn a_versioned_path_falls_back_to_the_bare_name() {
        let dir = TempDir::new();
        dir.write("Stats.pine", "// stats");
        let loader = DirLoader::new(vec![dir.0.clone()]);

        // `<user>/<Name>/<version>` also resolves against a bare `<Name>.pine`.
        assert_eq!(loader.load_library("alice/Stats/3").unwrap(), "// stats");
    }
}
