//! # ProcedureInstance — a sighting of a template in real code
//!
//! When we ingest code from GitHub, MCP documentation, or a user's own
//! files, each occurrence of a template becomes an Instance.
//!
//! Instances are LIGHT: they store a pointer to the canonical template
//! plus the context-specific variable names and the source location.
//!
//! Thousands of instances can share one template. The graph DEDUPLICATES:
//! instances with the same (template, binding, language) — if they're
//! common patterns — collapse into one atom with a `sighting_count`.

use std::collections::BTreeMap;

use super::template::TemplateId;

/// Which programming language (or notation) this instance was observed in.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Language {
    Rust,
    Python,
    JavaScript,
    TypeScript,
    Go,
    Java,
    CSharp,
    Ruby,
    Php,
    C,
    Cpp,
    Shell,
    /// SQL / database queries.
    Sql,
    /// Markdown/YAML API specs (OpenAPI, MCP manifests).
    Spec,
    /// Hebrew/English natural-language procedure (ZETS self-authored).
    NaturalLanguage,
    /// Math notation (LaTeX, plain).
    Math,
    Other(String),
}

impl Language {
    pub fn as_str(&self) -> String {
        match self {
            Language::Rust => "rust".into(),
            Language::Python => "python".into(),
            Language::JavaScript => "javascript".into(),
            Language::TypeScript => "typescript".into(),
            Language::Go => "go".into(),
            Language::Java => "java".into(),
            Language::CSharp => "csharp".into(),
            Language::Ruby => "ruby".into(),
            Language::Php => "php".into(),
            Language::C => "c".into(),
            Language::Cpp => "cpp".into(),
            Language::Shell => "shell".into(),
            Language::Sql => "sql".into(),
            Language::Spec => "spec".into(),
            Language::NaturalLanguage => "natural_language".into(),
            Language::Math => "math".into(),
            Language::Other(s) => format!("other.{}", s),
        }
    }
}

/// Where this instance was observed — enough to deduplicate and trace back.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SourceLocation {
    /// Repository or origin (e.g. "github.com/org/repo", "mcp.anthropic.com/gmail", "user:idan/file.py").
    pub origin: String,
    /// Specific file path within origin.
    pub file_path: Option<String>,
    /// Line number where the instance starts, if known.
    pub line: Option<u32>,
    /// Commit / version at ingest time, if applicable.
    pub version: Option<String>,
}

impl SourceLocation {
    pub fn new(origin: impl Into<String>) -> Self {
        SourceLocation {
            origin: origin.into(),
            file_path: None,
            line: None,
            version: None,
        }
    }

    pub fn at(mut self, file: impl Into<String>, line: u32) -> Self {
        self.file_path = Some(file.into());
        self.line = Some(line);
        self
    }

    pub fn versioned(mut self, v: impl Into<String>) -> Self {
        self.version = Some(v.into());
        self
    }

    /// Short handle for the location — used as a dedup key.
    pub fn handle(&self) -> String {
        match (&self.file_path, self.line) {
            (Some(f), Some(l)) => format!("{}:{}:{}", self.origin, f, l),
            (Some(f), None) => format!("{}:{}", self.origin, f),
            _ => self.origin.clone(),
        }
    }
}

/// A single observed use of a template.
#[derive(Debug, Clone)]
pub struct ProcedureInstance {
    /// Which template this is an instance of.
    pub template_id: TemplateId,
    /// The observation location.
    pub source: SourceLocation,
    /// Language of the sighting.
    pub language: Language,
    /// How each parameter role was named in THIS instance.
    /// Key = role (e.g. "url"), Value = local name (e.g. "target_url").
    /// Only stored when the names matter (NameRole::Domain) or when
    /// we explicitly choose to trace the original.
    pub binding: BTreeMap<String, String>,
    /// Any extra params passed that aren't in the template.
    /// For `requests.post(url, timeout=30)` — `{"timeout": "30"}`.
    pub extra_args: BTreeMap<String, String>,
    /// How many times have we seen this specific binding?
    /// 1 = first sighting. >1 = seen repeatedly (common pattern).
    pub sighting_count: u32,
    /// When first seen (Unix ms).
    pub first_seen_ms: i64,
    /// When last seen (Unix ms).
    pub last_seen_ms: i64,
}

impl ProcedureInstance {
    pub fn new(
        template_id: TemplateId,
        source: SourceLocation,
        language: Language,
        now_ms: i64,
    ) -> Self {
        ProcedureInstance {
            template_id,
            source,
            language,
            binding: BTreeMap::new(),
            extra_args: BTreeMap::new(),
            sighting_count: 1,
            first_seen_ms: now_ms,
            last_seen_ms: now_ms,
        }
    }

    /// Bind a template parameter (by role) to a local name in this instance.
    pub fn bind(mut self, role: impl Into<String>, local_name: impl Into<String>) -> Self {
        self.binding.insert(role.into(), local_name.into());
        self
    }

    pub fn with_extra(mut self, k: impl Into<String>, v: impl Into<String>) -> Self {
        self.extra_args.insert(k.into(), v.into());
        self
    }

    /// Record another sighting — bump count and timestamp.
    pub fn touch(&mut self, now_ms: i64) {
        self.sighting_count += 1;
        self.last_seen_ms = now_ms;
    }

    /// A dedup key for this instance: combines template, binding, language.
    /// Two sightings with same key collapse via `touch`.
    pub fn dedup_key(&self) -> String {
        let mut parts = vec![
            self.template_id.0.clone(),
            self.language.as_str(),
        ];
        for (role, name) in &self.binding {
            parts.push(format!("{}={}", role, name));
        }
        parts.join("|")
    }

    /// Is this a "trivially bound" instance — all params use template roles
    /// as their names, no surprises? Indicates either ZETS-authored code
    /// or code following conventions.
    pub fn is_trivially_bound(&self) -> bool {
        self.binding.iter().all(|(role, name)| role == name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_instance() {
        let inst = ProcedureInstance::new(
            TemplateId::new("http.post"),
            SourceLocation::new("github.com/org/repo").at("client.py", 42),
            Language::Python,
            1000,
        )
        .bind("url", "target_url")
        .bind("headers", "my_headers")
        .bind("body", "payload")
        .with_extra("timeout", "30");

        assert_eq!(inst.template_id.0, "http.post");
        assert_eq!(inst.binding.get("url"), Some(&"target_url".into()));
        assert_eq!(inst.extra_args.get("timeout"), Some(&"30".into()));
        assert_eq!(inst.sighting_count, 1);
    }

    #[test]
    fn test_two_different_names_same_template() {
        // Python: target_url, my_headers, payload
        let py = ProcedureInstance::new(
            TemplateId::new("http.post"),
            SourceLocation::new("github.com/a/b").at("p.py", 1),
            Language::Python,
            1000,
        )
        .bind("url", "target_url")
        .bind("headers", "my_headers")
        .bind("body", "payload");

        // JavaScript: endpoint, h, b
        let js = ProcedureInstance::new(
            TemplateId::new("http.post"),
            SourceLocation::new("github.com/c/d").at("client.js", 5),
            Language::JavaScript,
            1000,
        )
        .bind("url", "endpoint")
        .bind("headers", "h")
        .bind("body", "b");

        // Both point to SAME template
        assert_eq!(py.template_id, js.template_id);
        // But different dedup keys — different languages, different names
        assert_ne!(py.dedup_key(), js.dedup_key());
    }

    #[test]
    fn test_dedup_same_binding_same_lang() {
        // Same language, same binding → same dedup key
        let a = ProcedureInstance::new(
            TemplateId::new("http.post"),
            SourceLocation::new("repo:a").at("x.py", 1),
            Language::Python,
            1000,
        )
        .bind("url", "url")
        .bind("headers", "headers");

        let b = ProcedureInstance::new(
            TemplateId::new("http.post"),
            SourceLocation::new("repo:b").at("y.py", 5),
            Language::Python,
            2000,
        )
        .bind("url", "url")
        .bind("headers", "headers");

        // Sources differ, but binding and template are the same
        assert_eq!(a.dedup_key(), b.dedup_key());
    }

    #[test]
    fn test_touch_increments_count() {
        let mut inst = ProcedureInstance::new(
            TemplateId::new("fs.read"),
            SourceLocation::new("origin"),
            Language::Rust,
            1000,
        );
        assert_eq!(inst.sighting_count, 1);
        inst.touch(2000);
        assert_eq!(inst.sighting_count, 2);
        assert_eq!(inst.last_seen_ms, 2000);
        inst.touch(3000);
        assert_eq!(inst.sighting_count, 3);
    }

    #[test]
    fn test_trivially_bound_detection() {
        let trivial = ProcedureInstance::new(
            TemplateId::new("http.post"),
            SourceLocation::new("zets_self"),
            Language::Rust,
            1000,
        )
        .bind("url", "url")
        .bind("body", "body");

        let non_trivial = ProcedureInstance::new(
            TemplateId::new("http.post"),
            SourceLocation::new("user_code"),
            Language::Python,
            1000,
        )
        .bind("url", "endpoint")
        .bind("body", "payload");

        assert!(trivial.is_trivially_bound());
        assert!(!non_trivial.is_trivially_bound());
    }

    #[test]
    fn test_source_location_handle() {
        let loc1 = SourceLocation::new("github.com/a/b").at("f.py", 42);
        assert_eq!(loc1.handle(), "github.com/a/b:f.py:42");

        let loc2 = SourceLocation::new("mcp.anthropic.com/gmail");
        assert_eq!(loc2.handle(), "mcp.anthropic.com/gmail");
    }

    #[test]
    fn test_multi_language_same_template() {
        let template = TemplateId::new("http.post");
        // Same template, 4 languages, 4 different bindings
        let rust = ProcedureInstance::new(
            template.clone(),
            SourceLocation::new("r"),
            Language::Rust,
            1000,
        )
        .bind("url", "uri");

        let go = ProcedureInstance::new(
            template.clone(),
            SourceLocation::new("g"),
            Language::Go,
            1000,
        )
        .bind("url", "address");

        let java = ProcedureInstance::new(
            template.clone(),
            SourceLocation::new("j"),
            Language::Java,
            1000,
        )
        .bind("url", "requestUrl");

        let py = ProcedureInstance::new(
            template,
            SourceLocation::new("p"),
            Language::Python,
            1000,
        )
        .bind("url", "url");

        let keys: Vec<_> = [&rust, &go, &java, &py]
            .iter()
            .map(|i| i.dedup_key())
            .collect();

        // All different — so all 4 instances stored separately
        // but ALL POINT TO THE SAME TEMPLATE
        for i in 0..keys.len() {
            for j in i + 1..keys.len() {
                assert_ne!(keys[i], keys[j], "languages should dedup distinctly");
            }
        }
    }

    #[test]
    fn test_extra_args_preserved() {
        let inst = ProcedureInstance::new(
            TemplateId::new("http.post"),
            SourceLocation::new("origin"),
            Language::Python,
            1000,
        )
        .bind("url", "url")
        .with_extra("timeout", "30")
        .with_extra("verify", "true");

        assert_eq!(inst.extra_args.len(), 2);
        assert_eq!(inst.extra_args.get("timeout"), Some(&"30".into()));
    }
}
