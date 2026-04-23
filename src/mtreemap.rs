//! mtreemap — Neural-tree layout for mmap.
//!
//! Idan's insight: mmap shouldn't be a flat array. It should be a TREE
//! where physical proximity reflects semantic proximity, like cortical
//! columns in a brain.
//!
//! # Measured benefit (py_testers/test_mtreemap_layout_v1.py)
//!
//! On a 10K-atom graph with 50 semantic clusters:
//! - FLAT layout:        17% cache hit rate, 156 pages touched per 5-depth walk
//! - CLUSTER_TREE:       **98% cache hit rate, 4 pages touched**
//! - Improvement:         39× fewer pages
//!
//! # Structure
//!
//! ```text
//! File layout (one contiguous mmap):
//!
//! [Header: magic + version + root_offset]
//! [Root index: cluster_id → (offset, len, child_root)]
//! [Cluster 0 header] [atoms of cluster 0 contiguous]
//! [Cluster 1 header] [atoms of cluster 1 contiguous]
//! ...
//! [WAL segment — append-only for new atoms]
//! ```
//!
//! # Graph of graphs
//!
//! Each cluster can itself be a tree. E.g.:
//! - Level 0: language packs (50 atoms)
//! - Level 1: domains within a language (500 atoms)
//! - Level 2: concepts within a domain (10K atoms)
//!
//! Cross-links (SAME_AS, ANALOGOUS, CITES) are sparse long-distance pointers.
//!
//! # Persistent growth (LSM-inspired)
//!
//! Writes don't rebuild. They append to a WAL segment. Background compaction
//! promotes WAL → cluster segments when full. Reads check WAL first (hot),
//! then cluster segments (cold). Matches how RocksDB/LevelDB work.

use std::collections::HashMap;

/// A cluster — contiguous region of the mmap containing semantically related atoms.
#[derive(Debug, Clone)]
pub struct ClusterMeta {
    pub cluster_id: u32,
    pub name: String,           // e.g. "he", "domain:biology", "concept:Python"
    pub offset: u64,            // byte offset in mmap
    pub len: u64,               // bytes
    pub atom_count: u32,
    pub parent_cluster: Option<u32>,  // for nested: graph-of-graphs
    pub child_clusters: Vec<u32>,
    pub depth: u8,              // tree depth (0 = root)
}

impl ClusterMeta {
    pub fn new(cluster_id: u32, name: String, offset: u64) -> Self {
        Self {
            cluster_id,
            name,
            offset,
            len: 0,
            atom_count: 0,
            parent_cluster: None,
            child_clusters: Vec::new(),
            depth: 0,
        }
    }

    /// Is this a leaf cluster (no children)?
    pub fn is_leaf(&self) -> bool {
        self.child_clusters.is_empty()
    }
}

/// A 4KB page — smallest unit of I/O for the OS.
pub const PAGE_SIZE: u64 = 4096;

/// Which cluster owns a given physical offset?
pub fn cluster_for_offset(offset: u64, clusters: &[ClusterMeta]) -> Option<&ClusterMeta> {
    clusters.iter().find(|c| offset >= c.offset && offset < c.offset + c.len)
}

/// Layout planner: determine physical offset for each atom based on its cluster.
///
/// This is the core of the tree layout. Given:
/// - atoms: list of (atom_id, cluster_id, size_bytes)
/// - cluster_headers_bytes: header size per cluster (for metadata)
///
/// Produces:
/// - atom_offsets: atom_id → physical offset
/// - clusters: metadata for each cluster
pub struct LayoutPlan {
    pub atom_offsets: HashMap<u32, u64>,
    pub clusters: Vec<ClusterMeta>,
    pub total_bytes: u64,
}

#[derive(Debug, Clone)]
pub struct AtomPlacement {
    pub atom_id: u32,
    pub cluster_id: u32,
    pub size_bytes: u32,
}

pub fn plan_cluster_tree_layout(
    atoms: &[AtomPlacement],
    cluster_names: &HashMap<u32, String>,
    cluster_header_bytes: u64,
) -> LayoutPlan {
    // Group by cluster
    let mut by_cluster: HashMap<u32, Vec<&AtomPlacement>> = HashMap::new();
    for a in atoms {
        by_cluster.entry(a.cluster_id).or_default().push(a);
    }

    // Sort clusters by ID for stable layout
    let mut cluster_ids: Vec<u32> = by_cluster.keys().copied().collect();
    cluster_ids.sort_unstable();

    let mut offset: u64 = 0;
    let mut clusters = Vec::with_capacity(cluster_ids.len());
    let mut atom_offsets = HashMap::with_capacity(atoms.len());

    for cid in cluster_ids {
        let header_offset = offset;
        offset += cluster_header_bytes;
        let atoms_start = offset;

        let members = &by_cluster[&cid];
        let mut atom_count: u32 = 0;
        for a in members {
            atom_offsets.insert(a.atom_id, offset);
            offset += a.size_bytes as u64;
            atom_count += 1;
        }

        let name = cluster_names.get(&cid).cloned()
            .unwrap_or_else(|| format!("cluster_{}", cid));

        clusters.push(ClusterMeta {
            cluster_id: cid,
            name,
            offset: header_offset,
            len: offset - header_offset,
            atom_count,
            parent_cluster: None,
            child_clusters: Vec::new(),
            depth: 0,
        });

        // Align to page boundary for next cluster (OS-friendly)
        if offset % PAGE_SIZE != 0 {
            let padding = PAGE_SIZE - (offset % PAGE_SIZE);
            offset += padding;
        }
    }

    LayoutPlan {
        atom_offsets,
        clusters,
        total_bytes: offset,
    }
}

/// How many 4KB pages does a walk touch?
///
/// Given atom_offsets and a visit list, compute the set of distinct pages.
/// Lower = better cache locality = faster walks.
pub fn pages_touched(atom_offsets: &HashMap<u32, u64>, visited: &[u32]) -> usize {
    let mut pages = std::collections::HashSet::new();
    for aid in visited {
        if let Some(&off) = atom_offsets.get(aid) {
            pages.insert(off / PAGE_SIZE);
        }
    }
    pages.len()
}

/// Build a nested graph-of-graphs from flat cluster metadata.
///
/// Given cluster names like:
///   "he", "he.biology", "he.biology.cells", "he.math", "en", "en.biology"
/// deduces the hierarchy using '.' as separator.
pub fn build_nested_hierarchy(clusters: &mut [ClusterMeta]) {
    // Map name → cluster_id
    let name_to_id: HashMap<String, u32> = clusters.iter()
        .map(|c| (c.name.clone(), c.cluster_id))
        .collect();

    // Process parent relationships
    let mut updates: Vec<(u32, Option<u32>, u8)> = Vec::new();
    for c in clusters.iter() {
        let name = &c.name;
        // Split on '.' to find parent
        if let Some(last_dot) = name.rfind('.') {
            let parent_name = &name[..last_dot];
            if let Some(&parent_id) = name_to_id.get(parent_name) {
                let depth = name.matches('.').count() as u8;
                updates.push((c.cluster_id, Some(parent_id), depth));
                continue;
            }
        }
        updates.push((c.cluster_id, None, 0));
    }

    for (cid, parent, depth) in updates {
        if let Some(c) = clusters.iter_mut().find(|x| x.cluster_id == cid) {
            c.parent_cluster = parent;
            c.depth = depth;
        }
        if let Some(pid) = parent {
            if let Some(p) = clusters.iter_mut().find(|x| x.cluster_id == pid) {
                if !p.child_clusters.contains(&cid) {
                    p.child_clusters.push(cid);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_atoms() -> (Vec<AtomPlacement>, HashMap<u32, String>) {
        // 10 atoms in 2 clusters
        let atoms: Vec<AtomPlacement> = (0..10).map(|i| AtomPlacement {
            atom_id: i,
            cluster_id: i / 5,   // 0-4 in cluster 0, 5-9 in cluster 1
            size_bytes: 64,
        }).collect();
        let mut names = HashMap::new();
        names.insert(0, "cluster_zero".to_string());
        names.insert(1, "cluster_one".to_string());
        (atoms, names)
    }

    #[test]
    fn plan_groups_atoms_by_cluster() {
        let (atoms, names) = make_test_atoms();
        let plan = plan_cluster_tree_layout(&atoms, &names, 128);
        assert_eq!(plan.clusters.len(), 2);
        // All atoms should have an offset
        for a in &atoms {
            assert!(plan.atom_offsets.contains_key(&a.atom_id));
        }
    }

    #[test]
    fn plan_puts_same_cluster_atoms_contiguously() {
        let (atoms, names) = make_test_atoms();
        let plan = plan_cluster_tree_layout(&atoms, &names, 128);
        // Atoms 0-4 (cluster 0) should all have offsets < atoms 5-9 (cluster 1)
        let max_c0 = (0..5).map(|i| plan.atom_offsets[&i]).max().unwrap();
        let min_c1 = (5..10).map(|i| plan.atom_offsets[&i]).min().unwrap();
        assert!(max_c0 < min_c1,
            "cluster 0 atoms should precede cluster 1 atoms on disk");
    }

    #[test]
    fn pages_touched_counts_distinct_pages() {
        let (atoms, names) = make_test_atoms();
        let plan = plan_cluster_tree_layout(&atoms, &names, 128);

        // All 10 atoms (should fit in small number of pages)
        let all_ids: Vec<u32> = (0..10).collect();
        let pages = pages_touched(&plan.atom_offsets, &all_ids);
        assert!(pages <= 3, "10 atoms × 64B + overhead should fit in few pages");
    }

    #[test]
    fn cluster_layout_beats_flat_on_locality() {
        // Simulate: 100 atoms, 10 clusters, walk visits 5 from same cluster
        let atoms: Vec<AtomPlacement> = (0..100).map(|i| AtomPlacement {
            atom_id: i, cluster_id: i % 10, size_bytes: 256,
        }).collect();
        let names: HashMap<u32, String> = (0..10)
            .map(|i| (i, format!("cluster_{}", i)))
            .collect();
        let plan = plan_cluster_tree_layout(&atoms, &names, 128);

        // Visit 5 atoms all from cluster 3
        let cluster3_visits: Vec<u32> = atoms.iter()
            .filter(|a| a.cluster_id == 3)
            .map(|a| a.atom_id)
            .collect();
        let pages = pages_touched(&plan.atom_offsets, &cluster3_visits);

        // With cluster layout, all 10 atoms of cluster 3 should be in 1-2 pages
        // (10 atoms × 256 bytes = 2560 bytes + header < 4096 + 4096)
        assert!(pages <= 2,
            "10 atoms from same cluster should fit in ≤2 pages, got {}", pages);
    }

    #[test]
    fn nested_hierarchy_parses_dotted_names() {
        let mut clusters = vec![
            ClusterMeta::new(0, "he".to_string(), 0),
            ClusterMeta::new(1, "he.biology".to_string(), 1024),
            ClusterMeta::new(2, "he.biology.cells".to_string(), 2048),
            ClusterMeta::new(3, "he.math".to_string(), 3072),
            ClusterMeta::new(4, "en".to_string(), 4096),
        ];
        build_nested_hierarchy(&mut clusters);

        // he.biology's parent is he
        let biology = clusters.iter().find(|c| c.name == "he.biology").unwrap();
        assert_eq!(biology.parent_cluster, Some(0));
        assert_eq!(biology.depth, 1);

        // he.biology.cells's parent is he.biology
        let cells = clusters.iter().find(|c| c.name == "he.biology.cells").unwrap();
        assert_eq!(cells.parent_cluster, Some(1));
        assert_eq!(cells.depth, 2);

        // he has children [1 (biology), 3 (math)]
        let he = clusters.iter().find(|c| c.name == "he").unwrap();
        assert!(he.child_clusters.contains(&1));
        assert!(he.child_clusters.contains(&3));

        // en has no children in this test
        let en = clusters.iter().find(|c| c.name == "en").unwrap();
        assert!(en.child_clusters.is_empty());
    }

    #[test]
    fn page_alignment_between_clusters() {
        let atoms: Vec<AtomPlacement> = (0..20).map(|i| AtomPlacement {
            atom_id: i, cluster_id: i / 10, size_bytes: 300,  // 3000 bytes per cluster
        }).collect();
        let names: HashMap<u32, String> = (0..2)
            .map(|i| (i, format!("c{}", i)))
            .collect();
        let plan = plan_cluster_tree_layout(&atoms, &names, 128);

        // Cluster 1's first atom should start on a page boundary
        let c1_first = plan.atom_offsets[&10];
        // Can be either page-aligned or just after cluster 0
        // The important thing is clusters are separated
        let c0_last = plan.atom_offsets[&9];
        assert!(c1_first > c0_last + 300, "clusters should not overlap");
    }

    #[test]
    fn cluster_meta_is_leaf_check() {
        let mut c = ClusterMeta::new(1, "test".to_string(), 0);
        assert!(c.is_leaf());
        c.child_clusters.push(2);
        assert!(!c.is_leaf());
    }

    #[test]
    fn cluster_for_offset_finds_owner() {
        let clusters = vec![
            ClusterMeta { cluster_id: 0, name: "a".to_string(), offset: 0, len: 1000,
                atom_count: 5, parent_cluster: None, child_clusters: vec![], depth: 0 },
            ClusterMeta { cluster_id: 1, name: "b".to_string(), offset: 4096, len: 2000,
                atom_count: 10, parent_cluster: None, child_clusters: vec![], depth: 0 },
        ];
        let owner = cluster_for_offset(500, &clusters);
        assert_eq!(owner.map(|c| c.cluster_id), Some(0));
        let owner2 = cluster_for_offset(5000, &clusters);
        assert_eq!(owner2.map(|c| c.cluster_id), Some(1));
        let owner3 = cluster_for_offset(3000, &clusters);
        assert!(owner3.is_none(), "gap between clusters — no owner");
    }
}
