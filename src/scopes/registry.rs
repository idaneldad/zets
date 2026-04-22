//! ScopeRegistry — the live registry of all scopes a running ZETS instance has.
//!
//! This is the "graph of graphs" that Idan requested. The registry is itself
//! stored as nodes in the system graph, so ZETS can reason about its own
//! graph topology.
//!
//! Example: "What scopes do I have loaded?" → route reads from here.

use std::collections::HashMap;
use std::path::PathBuf;

use super::{EncryptionTier, ScopeId, ScopePaths};

/// A single graph scope — its location, tier, permissions.
#[derive(Debug, Clone)]
pub struct GraphScope {
    pub id: ScopeId,
    pub instance_name: String,  // e.g., "idan" for a user scope, "en" for a lang
    pub path: PathBuf,
    pub encryption: EncryptionTier,
    pub writable: bool,
    /// If true, this scope is currently loaded in RAM.
    pub loaded: bool,
    /// Byte count on disk.
    pub disk_size: u64,
}

impl GraphScope {
    pub fn new(
        id: ScopeId,
        instance_name: impl Into<String>,
        path: PathBuf,
    ) -> Self {
        let encryption = id.default_encryption();
        let writable = id.is_writable();
        Self {
            id,
            instance_name: instance_name.into(),
            path,
            encryption,
            writable,
            loaded: false,
            disk_size: 0,
        }
    }

    /// Qualified name: "scope.instance", e.g., "user.idan", "language.he"
    pub fn qualified_name(&self) -> String {
        format!("{}.{}", self.id.name(), self.instance_name)
    }

    pub fn exists_on_disk(&self) -> bool {
        self.path.exists()
    }
}

/// Registry of all scopes this ZETS instance can see.
pub struct ScopeRegistry {
    pub paths: ScopePaths,
    scopes: HashMap<String, GraphScope>,
}

impl ScopeRegistry {
    pub fn new(paths: ScopePaths) -> Self {
        Self {
            paths,
            scopes: HashMap::new(),
        }
    }

    /// Build a default registry by probing the standard locations.
    pub fn discover(root: impl Into<PathBuf>) -> Self {
        let paths = ScopePaths::new(root);
        let mut reg = Self::new(paths);
        reg.auto_discover();
        reg
    }

    fn auto_discover(&mut self) {
        // System scope
        let sys_path = self.paths.system();
        if sys_path.exists() {
            self.register(GraphScope::new(ScopeId::System, "core", sys_path.clone()));
        }

        // Data core
        let core_path = self.paths.data_core();
        if core_path.exists() {
            self.register(GraphScope::new(ScopeId::Data, "universal", core_path.clone()));
        }

        // Languages — scan packs/ for zets.XX files
        if let Some(packs_dir) = self.paths.data_core().parent() {
            if let Ok(entries) = std::fs::read_dir(packs_dir) {
                for entry in entries.flatten() {
                    let name = entry.file_name().to_string_lossy().to_string();
                    if let Some(lang) = name.strip_prefix("zets.") {
                        // Skip core, system, wal files
                        if lang == "core" || lang == "system" || lang.contains('.') {
                            continue;
                        }
                        self.register(GraphScope::new(
                            ScopeId::Language,
                            lang.to_string(),
                            entry.path(),
                        ));
                    }
                }
            }
        }
    }

    pub fn register(&mut self, mut scope: GraphScope) {
        if let Ok(meta) = std::fs::metadata(&scope.path) {
            scope.disk_size = meta.len();
        }
        self.scopes.insert(scope.qualified_name(), scope);
    }

    pub fn get(&self, qualified_name: &str) -> Option<&GraphScope> {
        self.scopes.get(qualified_name)
    }

    pub fn all(&self) -> Vec<&GraphScope> {
        let mut v: Vec<_> = self.scopes.values().collect();
        v.sort_by_key(|s| (s.id.cascade_priority(), s.instance_name.clone()));
        v
    }

    pub fn by_scope(&self, id: ScopeId) -> Vec<&GraphScope> {
        self.scopes.values().filter(|s| s.id == id).collect()
    }

    /// Create (or stub) a writable scope if it doesn't exist yet.
    pub fn ensure_writable(&mut self, id: ScopeId, instance: &str) -> &GraphScope {
        let path = match id {
            ScopeId::User => self.paths.user(instance),
            ScopeId::Log => self.paths.log(),
            ScopeId::Testing => self.paths.testing(instance),
            ScopeId::Shared => self.paths.shared(instance),
            _ => panic!("ensure_writable called on read-only scope {:?}", id),
        };
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let scope = GraphScope::new(id, instance, path);
        let qn = scope.qualified_name();
        self.scopes.insert(qn.clone(), scope);
        self.scopes.get(&qn).unwrap()
    }

    /// Total size of all scopes on disk.
    pub fn total_disk_bytes(&self) -> u64 {
        self.scopes.values().map(|s| s.disk_size).sum()
    }

    /// Summary for display.
    pub fn describe(&self) -> String {
        let mut out = String::new();
        out.push_str("Scope Registry:\n");
        for scope in self.all() {
            out.push_str(&format!(
                "  [{:<9}] {:<20} path={:?} tier={} size={}B\n",
                scope.id.name(),
                scope.instance_name,
                scope.path.file_name().unwrap_or_default(),
                scope.encryption.description(),
                scope.disk_size,
            ));
        }
        out.push_str(&format!(
            "  Total: {} scope(s), {} bytes on disk\n",
            self.scopes.len(),
            self.total_disk_bytes()
        ));
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn registry_starts_empty() {
        let paths = ScopePaths::new("/tmp/test_zets");
        let reg = ScopeRegistry::new(paths);
        assert!(reg.all().is_empty());
    }

    #[test]
    fn register_and_retrieve() {
        let paths = ScopePaths::new("/tmp/test_zets");
        let mut reg = ScopeRegistry::new(paths);
        reg.register(GraphScope::new(
            ScopeId::Data,
            "universal",
            PathBuf::from("/tmp/foo"),
        ));
        assert_eq!(reg.all().len(), 1);
        assert!(reg.get("data.universal").is_some());
    }

    #[test]
    fn cascade_order_by_priority() {
        let paths = ScopePaths::new("/tmp/test_zets");
        let mut reg = ScopeRegistry::new(paths);
        reg.register(GraphScope::new(ScopeId::System, "x", PathBuf::from("/a")));
        reg.register(GraphScope::new(ScopeId::User, "idan", PathBuf::from("/b")));
        reg.register(GraphScope::new(ScopeId::Data, "universal", PathBuf::from("/c")));
        reg.register(GraphScope::new(ScopeId::Testing, "sim1", PathBuf::from("/d")));
        let order: Vec<&str> = reg.all().iter().map(|s| s.id.name()).collect();
        // Testing first, System last
        assert_eq!(order[0], "testing");
        assert_eq!(order[1], "user");
        assert_eq!(order[2], "data");
        assert_eq!(order[3], "system");
    }

    #[test]
    fn qualified_names() {
        let s = GraphScope::new(ScopeId::User, "idan", PathBuf::from("/x"));
        assert_eq!(s.qualified_name(), "user.idan");
        let s = GraphScope::new(ScopeId::Language, "he", PathBuf::from("/y"));
        assert_eq!(s.qualified_name(), "language.he");
    }

    #[test]
    fn ensure_writable_creates_user_scope() {
        let paths = ScopePaths::new("/tmp/test_zets_ensure");
        let mut reg = ScopeRegistry::new(paths);
        let scope = reg.ensure_writable(ScopeId::User, "alice");
        assert_eq!(scope.id, ScopeId::User);
        assert_eq!(scope.instance_name, "alice");
        assert!(scope.writable);
    }
}
