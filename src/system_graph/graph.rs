//! SystemGraph — the registry of all routes, organized by tier.
//!
//! Hot routes live fully in RAM (loaded from bootstrap or a small header
//! section of the system pack). Warm/Cold routes are loaded on first call
//! via mmap — same mechanism as the data graph.
//!
//! This mirrors the "data graph is lazy mmap" strategy, but for the
//! system/procedural side of things.

use std::collections::HashMap;

use super::bootstrap::all_bootstrap_routes;
use super::routes::{Route, RouteId, Tier};

pub struct SystemGraph {
    /// All loaded routes, indexed by id.
    routes: HashMap<RouteId, Route>,
    /// Routes marked Warm/Cold but not yet materialized.
    /// Points to bytes we can deserialize on demand.
    #[allow(dead_code)]
    lazy_index: HashMap<RouteId, Tier>,
    /// Stats
    hot_count: usize,
    warm_count: usize,
    cold_count: usize,
}

impl SystemGraph {
    /// Create a fresh system graph with bootstrap routes loaded.
    pub fn new_bootstrap() -> Self {
        let mut g = Self {
            routes: HashMap::new(),
            lazy_index: HashMap::new(),
            hot_count: 0,
            warm_count: 0,
            cold_count: 0,
        };
        for r in all_bootstrap_routes() {
            g.insert(r);
        }
        g
    }

    /// Empty graph — caller will populate.
    pub fn empty() -> Self {
        Self {
            routes: HashMap::new(),
            lazy_index: HashMap::new(),
            hot_count: 0,
            warm_count: 0,
            cold_count: 0,
        }
    }

    pub fn insert(&mut self, route: Route) {
        match route.tier {
            Tier::Hot => self.hot_count += 1,
            Tier::Warm => self.warm_count += 1,
            Tier::Cold => self.cold_count += 1,
            Tier::Archive => {}
        }
        self.routes.insert(route.id, route);
    }

    pub fn get(&self, id: RouteId) -> Option<&Route> {
        self.routes.get(&id)
    }

    pub fn routes(&self) -> &HashMap<RouteId, Route> {
        &self.routes
    }

    pub fn stats(&self) -> SystemGraphStats {
        let total_bytecode: usize = self.routes.values().map(|r| r.byte_count()).sum();
        SystemGraphStats {
            total_routes: self.routes.len(),
            hot: self.hot_count,
            warm: self.warm_count,
            cold: self.cold_count,
            total_bytecode_bytes: total_bytecode,
            avg_bytecode_per_route: if self.routes.is_empty() {
                0
            } else {
                total_bytecode / self.routes.len()
            },
        }
    }

    /// Get a human-readable dump of all loaded routes.
    pub fn describe(&self) -> String {
        let mut out = String::new();
        let mut routes: Vec<&Route> = self.routes.values().collect();
        routes.sort_by_key(|r| r.id);
        for r in routes {
            out.push_str(&format!(
                "  [{:?}] route {} '{}' ({} bytes, {} consts, {} params)\n",
                r.tier, r.id, r.name, r.byte_count(), r.constants.len(), r.param_count
            ));
            if !r.doc.is_empty() {
                out.push_str(&format!("      doc: {}\n", r.doc));
            }
        }
        out
    }
}

#[derive(Debug, Clone)]
pub struct SystemGraphStats {
    pub total_routes: usize,
    pub hot: usize,
    pub warm: usize,
    pub cold: usize,
    pub total_bytecode_bytes: usize,
    pub avg_bytecode_per_route: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bootstrap_graph_has_routes() {
        let g = SystemGraph::new_bootstrap();
        let s = g.stats();
        assert!(s.total_routes >= 3);
        assert_eq!(s.hot, s.total_routes);
        assert_eq!(s.warm, 0);
        assert_eq!(s.cold, 0);
    }

    #[test]
    fn bootstrap_is_small() {
        let g = SystemGraph::new_bootstrap();
        let s = g.stats();
        // Bootstrap (3 simple routes) should be well under 1 KB total
        assert!(s.total_bytecode_bytes < 1024);
    }

    #[test]
    fn describe_non_empty() {
        let g = SystemGraph::new_bootstrap();
        let d = g.describe();
        assert!(!d.is_empty());
        assert!(d.contains("learn_from_definition"));
    }
}
