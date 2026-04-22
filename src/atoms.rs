//! AtomStore — compositional storage for reusable pieces.
//!
//! Core insight (from Idan's design): ZETS stores *atoms* (reusable pieces),
//! never whole scenes/sentences. A "dog" isn't one blob — it's a collection
//! of atoms (body, head, tail, fur-texture) that combine via edges.
//!
//! Three savings mechanisms:
//!   1. CONTENT DEDUP — two atoms with identical bytes share one storage.
//!   2. TEMPLATE+DELTA — similar atoms reference a template plus a small diff.
//!   3. INSTANCE GRAPH — a "scene" is edges to existing atoms, not new data.
//!
//! Body-mind architecture fit (per Idan):
//!   L4 Tools    → external IO
//!   L3 Sensory  → async: video/audio/text → propose atoms (dedup HERE)
//!   L2 Atoms    → THIS MODULE: stable content store
//!   L1 Cognition→ cognitive_modes walk the atom graph
//!
//! Critical property: same content → same atom id, always. Like the body:
//! one heart, many systems using it; one `wheel` atom, many cars referencing.

use std::collections::HashMap;

// ────────────────────────────────────────────────────────────────────
// Types
// ────────────────────────────────────────────────────────────────────

/// Stable id of an atom. u32 gives us 4 billion atoms — enough for our scale.
pub type AtomId = u32;

/// FNV-1a 64-bit — deterministic, stable across platforms, no deps.
/// Good enough for content addressing (collision probability negligible at our scale).
pub fn content_hash(bytes: &[u8]) -> u64 {
    let mut h: u64 = 0xcbf29ce484222325;
    for &b in bytes {
        h ^= b as u64;
        h = h.wrapping_mul(0x100000001b3);
    }
    h
}

/// What kind of thing this atom represents. Determines how it can be composed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum AtomKind {
    /// A conceptual label ("dog", "car", "love")
    Concept = 0,
    /// A piece of text (caption, sentence fragment)
    Text = 1,
    /// Image bytes (downscaled, compressed — small thumbnails)
    ImageFrame = 2,
    /// Audio chunk bytes
    AudioChunk = 3,
    /// 3D/pose descriptor (e.g., head orientation)
    PoseVector = 4,
    /// Template — a "base" atom that others reference as a delta source
    Template = 5,
    /// Delta — a small offset from a template atom
    Delta = 6,
    /// Composition — a node that groups atoms into an instance (e.g., a car)
    Composition = 7,
    /// Relation/predicate atom (HUG, LOVE, IS_A, ...)
    Relation = 8,
}

impl AtomKind {
    pub fn name(self) -> &'static str {
        match self {
            Self::Concept => "Concept",
            Self::Text => "Text",
            Self::ImageFrame => "ImageFrame",
            Self::AudioChunk => "AudioChunk",
            Self::PoseVector => "PoseVector",
            Self::Template => "Template",
            Self::Delta => "Delta",
            Self::Composition => "Composition",
            Self::Relation => "Relation",
        }
    }
}

/// A single atom: one piece of reusable content.
#[derive(Debug, Clone)]
pub struct Atom {
    pub id: AtomId,
    pub kind: AtomKind,
    /// Content bytes. For templates/compositions, may be empty (structural only).
    pub data: Vec<u8>,
    /// Content hash — lets us dedup.
    pub content_hash: u64,
    /// How many edges reference this atom — used for compaction/GC.
    pub refcount: u32,
    /// Provenance id (0 = unknown). Points to a prov row elsewhere.
    pub provenance: u32,
}

impl Atom {
    pub fn size_bytes(&self) -> usize {
        self.data.len()
    }
}

/// A reference from one atom to another with a relation type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AtomEdge {
    pub from: AtomId,
    pub to: AtomId,
    /// Relation type (application-defined: 0=HAS_PART, 1=IS_A, 2=TEMPLATE_OF, ...)
    pub relation: u8,
    /// Weight 0-100 — confidence/importance.
    pub weight: u8,
    /// A small metadata tag — e.g., "position index" when a car has 4 wheels
    /// this disambiguates the slot (0=FL, 1=FR, 2=RL, 3=RR).
    pub slot: u16,
}

/// A DELTA references a TEMPLATE plus a small diff.
/// For video frames: the diff is a sparse set of (x,y,color) changes.
/// For poses: the diff is a rotation vector.
/// For generic bytes: an xor-compressed diff.
#[derive(Debug, Clone)]
pub struct DeltaFromTemplate {
    pub template_id: AtomId,
    pub delta_bytes: Vec<u8>,
    pub diff_method: DiffMethod,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiffMethod {
    /// Byte-level XOR (good for near-identical bytes)
    XorBytes = 0,
    /// Sparse indices+values (good for images where few pixels changed)
    Sparse = 1,
    /// Rotation vector in degrees (for pose atoms)
    RotationDegrees = 2,
    /// Offset+scale (for audio pitch/tempo variations)
    AudioOffsetScale = 3,
}

// ────────────────────────────────────────────────────────────────────
// Store
// ────────────────────────────────────────────────────────────────────

/// The atom store. All atoms live here; instances are composed via edges.
pub struct AtomStore {
    atoms: Vec<Atom>,
    /// content_hash → atom_id lookup for dedup
    content_index: HashMap<u64, AtomId>,
    edges: Vec<AtomEdge>,
    /// from_id → list of edge indices (populated lazily)
    outgoing_index: HashMap<AtomId, Vec<u32>>,
    /// Total raw bytes we avoided storing thanks to dedup
    bytes_saved_by_dedup: u64,
    /// Total raw bytes saved by using templates
    bytes_saved_by_delta: u64,
}

impl AtomStore {
    pub fn new() -> Self {
        Self {
            atoms: Vec::new(),
            content_index: HashMap::new(),
            edges: Vec::new(),
            outgoing_index: HashMap::new(),
            bytes_saved_by_dedup: 0,
            bytes_saved_by_delta: 0,
        }
    }

    /// Store an atom. If content is already present, returns existing id
    /// (dedup!) and records the bytes saved.
    pub fn put(&mut self, kind: AtomKind, data: Vec<u8>) -> AtomId {
        let hash = content_hash(&data);
        if let Some(&existing) = self.content_index.get(&hash) {
            // Content dedup: we already have this. Increment refcount, skip storage.
            self.atoms[existing as usize].refcount += 1;
            self.bytes_saved_by_dedup += data.len() as u64;
            return existing;
        }
        let id = self.atoms.len() as AtomId;
        let size = data.len();
        let atom = Atom {
            id,
            kind,
            data,
            content_hash: hash,
            refcount: 1,
            provenance: 0,
        };
        self.atoms.push(atom);
        self.content_index.insert(hash, id);
        let _ = size;
        id
    }

    /// Store a DELTA that references a template. Saves bytes because we store
    /// only the diff, not full data.
    pub fn put_delta(&mut self, template_id: AtomId, delta_bytes: Vec<u8>,
                     method: DiffMethod, implied_full_size: usize) -> AtomId {
        // If this delta is "empty" (delta to self), just return the template.
        if delta_bytes.is_empty() {
            self.atoms[template_id as usize].refcount += 1;
            self.bytes_saved_by_delta += implied_full_size as u64;
            return template_id;
        }
        let saved = implied_full_size.saturating_sub(delta_bytes.len());
        self.bytes_saved_by_delta += saved as u64;

        // Encode delta: first byte = method, next 4 = template_id, rest = delta bytes
        let mut payload = Vec::with_capacity(5 + delta_bytes.len());
        payload.push(method as u8);
        payload.extend_from_slice(&template_id.to_le_bytes());
        payload.extend_from_slice(&delta_bytes);

        let hash = content_hash(&payload);
        if let Some(&existing) = self.content_index.get(&hash) {
            self.atoms[existing as usize].refcount += 1;
            self.bytes_saved_by_dedup += payload.len() as u64;
            return existing;
        }

        let id = self.atoms.len() as AtomId;
        self.atoms.push(Atom {
            id,
            kind: AtomKind::Delta,
            data: payload,
            content_hash: hash,
            refcount: 1,
            provenance: 0,
        });
        self.content_index.insert(hash, id);
        id
    }

    /// Link two atoms. The instance graph is just edges between existing atoms —
    /// composition without duplication. Like a wheel-atom referenced by 4 cars.
    pub fn link(&mut self, from: AtomId, to: AtomId, relation: u8, weight: u8, slot: u16) {
        let idx = self.edges.len() as u32;
        self.edges.push(AtomEdge { from, to, relation, weight, slot });
        self.outgoing_index.entry(from).or_default().push(idx);
        if let Some(a) = self.atoms.get_mut(to as usize) {
            a.refcount += 1;
        }
    }

    pub fn get(&self, id: AtomId) -> Option<&Atom> {
        self.atoms.get(id as usize)
    }

    /// Is this delta? Decode its template_id (for reconstruction).
    pub fn delta_template_of(&self, id: AtomId) -> Option<AtomId> {
        let atom = self.atoms.get(id as usize)?;
        if atom.kind != AtomKind::Delta { return None; }
        if atom.data.len() < 5 { return None; }
        let bytes: [u8; 4] = atom.data[1..5].try_into().ok()?;
        Some(u32::from_le_bytes(bytes))
    }

    /// Reconstruct full content of an atom. For a regular atom: returns data.
    /// For a delta: reconstructs by applying diff to template.
    pub fn reconstruct(&self, id: AtomId) -> Option<Vec<u8>> {
        let atom = self.atoms.get(id as usize)?;
        if atom.kind != AtomKind::Delta {
            return Some(atom.data.clone());
        }
        // Decode delta: [method_byte][template_id_4b][delta_bytes...]
        if atom.data.len() < 5 { return None; }
        let method = match atom.data[0] {
            0 => DiffMethod::XorBytes,
            1 => DiffMethod::Sparse,
            2 => DiffMethod::RotationDegrees,
            3 => DiffMethod::AudioOffsetScale,
            _ => return None,
        };
        let tid_bytes: [u8; 4] = atom.data[1..5].try_into().ok()?;
        let template_id = u32::from_le_bytes(tid_bytes);
        let template = self.atoms.get(template_id as usize)?;
        let delta_bytes = &atom.data[5..];

        match method {
            DiffMethod::XorBytes => {
                let mut out = template.data.clone();
                for (i, &d) in delta_bytes.iter().enumerate() {
                    if i < out.len() { out[i] ^= d; }
                }
                Some(out)
            }
            DiffMethod::Sparse => {
                // delta = repeated (offset:u32, byte:u8) pairs
                let mut out = template.data.clone();
                let mut i = 0;
                while i + 5 <= delta_bytes.len() {
                    let off_bytes: [u8; 4] = delta_bytes[i..i+4].try_into().ok()?;
                    let off = u32::from_le_bytes(off_bytes) as usize;
                    let val = delta_bytes[i+4];
                    if off < out.len() { out[off] = val; }
                    i += 5;
                }
                Some(out)
            }
            DiffMethod::RotationDegrees | DiffMethod::AudioOffsetScale => {
                // For these methods, the delta IS the full semantic content.
                // Caller interprets the meaning.
                Some(delta_bytes.to_vec())
            }
        }
    }

    /// Walk outgoing edges from an atom.
    pub fn outgoing(&self, from: AtomId) -> Vec<AtomEdge> {
        self.outgoing_index.get(&from)
            .map(|indices| indices.iter().map(|&i| self.edges[i as usize]).collect())
            .unwrap_or_default()
    }

    /// Find all edges pointing TO `target` with the given relation.
    /// O(E) — uses no incoming index, so avoid in hot paths.
    /// Suitable for occasional dreaming / meta-reasoning queries.
    pub fn incoming_by_relation(&self, target: AtomId, relation: u8) -> Vec<AtomEdge> {
        self.edges.iter()
            .filter(|e| e.to == target && e.relation == relation)
            .copied()
            .collect()
    }

    /// Total number of edges (for stats/metrics).
    pub fn edge_count(&self) -> usize { self.edges.len() }

    // ── Statistics ──

    pub fn stats(&self) -> StoreStats {
        let total_bytes: u64 = self.atoms.iter().map(|a| a.data.len() as u64).sum();
        let by_kind = self.count_by_kind();
        StoreStats {
            atom_count: self.atoms.len(),
            edge_count: self.edges.len(),
            total_bytes,
            bytes_saved_dedup: self.bytes_saved_by_dedup,
            bytes_saved_delta: self.bytes_saved_by_delta,
            by_kind,
        }
    }

    fn count_by_kind(&self) -> HashMap<AtomKind, usize> {
        let mut m: HashMap<AtomKind, usize> = HashMap::new();
        for a in &self.atoms {
            *m.entry(a.kind).or_insert(0) += 1;
        }
        m
    }
}

impl Default for AtomStore {
    fn default() -> Self { Self::new() }
}

#[derive(Debug, Clone)]
pub struct StoreStats {
    pub atom_count: usize,
    pub edge_count: usize,
    pub total_bytes: u64,
    pub bytes_saved_dedup: u64,
    pub bytes_saved_delta: u64,
    pub by_kind: HashMap<AtomKind, usize>,
}

impl StoreStats {
    /// Compression ratio vs storing everything uncompressed.
    pub fn compression_ratio(&self) -> f64 {
        let saved = self.bytes_saved_dedup + self.bytes_saved_delta;
        if self.total_bytes + saved == 0 { return 1.0; }
        (self.total_bytes + saved) as f64 / self.total_bytes.max(1) as f64
    }
}

// ────────────────────────────────────────────────────────────────────
// Relations (application layer — but kept here for convenience)
// ────────────────────────────────────────────────────────────────────

pub mod rel {
    pub const HAS_PART:        u8 = 0;
    pub const IS_A:            u8 = 1;
    pub const TEMPLATE_OF:     u8 = 2;
    pub const HAS_COLOR:       u8 = 3;
    pub const HAS_POSE:        u8 = 4;
    pub const HUG:             u8 = 5;
    pub const LOVE:            u8 = 6;
    pub const CRY:             u8 = 7;
    pub const LOST:            u8 = 8;
    pub const AT_LOCATION:     u8 = 9;
    pub const NEAR:            u8 = 10;
    pub const WALK_IN:         u8 = 11;
    pub const DEPICTS:         u8 = 12;  // ImageFrame atom → Concept it shows
    pub const SAYS:            u8 = 13;  // AudioChunk atom → Text atom
}

// ────────────────────────────────────────────────────────────────────
// Tests
// ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_is_deterministic() {
        assert_eq!(content_hash(b"hello"), content_hash(b"hello"));
        assert_ne!(content_hash(b"hello"), content_hash(b"hellp"));
    }

    #[test]
    fn put_dedups_identical_content() {
        let mut store = AtomStore::new();
        let id1 = store.put(AtomKind::ImageFrame, b"same bytes".to_vec());
        let id2 = store.put(AtomKind::ImageFrame, b"same bytes".to_vec());
        assert_eq!(id1, id2, "identical content must dedup to same id");
        assert_eq!(store.stats().atom_count, 1);
        assert!(store.stats().bytes_saved_dedup >= 10);
    }

    #[test]
    fn put_keeps_different_content_separate() {
        let mut store = AtomStore::new();
        let id1 = store.put(AtomKind::Concept, b"dog".to_vec());
        let id2 = store.put(AtomKind::Concept, b"cat".to_vec());
        assert_ne!(id1, id2);
        assert_eq!(store.stats().atom_count, 2);
    }

    #[test]
    fn car_with_four_wheels_stores_wheel_once() {
        let mut store = AtomStore::new();
        let wheel = store.put(AtomKind::Concept, b"wheel".to_vec());
        let chassis = store.put(AtomKind::Concept, b"chassis-model-X".to_vec());
        let car = store.put(AtomKind::Composition, b"car-instance-001".to_vec());

        store.link(car, chassis, rel::HAS_PART, 100, 0);
        store.link(car, wheel, rel::HAS_PART, 100, 0); // front-left
        store.link(car, wheel, rel::HAS_PART, 100, 1); // front-right
        store.link(car, wheel, rel::HAS_PART, 100, 2); // rear-left
        store.link(car, wheel, rel::HAS_PART, 100, 3); // rear-right

        let s = store.stats();
        assert_eq!(s.atom_count, 3, "wheel+chassis+car = 3 atoms total");
        assert_eq!(s.edge_count, 5);
        assert_eq!(store.get(wheel).unwrap().refcount, 5); // 1 put + 4 links
    }

    #[test]
    fn mannequin_body_plus_three_head_poses() {
        let mut store = AtomStore::new();
        let body = store.put(AtomKind::Concept, b"body-model-17-with-dress".to_vec());
        // Template head pose (front)
        let head_front = store.put(AtomKind::PoseVector, vec![0, 0, 0]); // rot 0,0,0
        // Left rotation = delta from front
        let head_left = store.put_delta(head_front, vec![45, 0, 0],
            DiffMethod::RotationDegrees, 3);
        // Right rotation
        let head_right = store.put_delta(head_front, vec![0xD3, 0xFF, 0, 0],  // -45 degrees
            DiffMethod::RotationDegrees, 3);

        let mannequin_1 = store.put(AtomKind::Composition, b"mannequin-01".to_vec());
        let mannequin_2 = store.put(AtomKind::Composition, b"mannequin-02".to_vec());
        let mannequin_3 = store.put(AtomKind::Composition, b"mannequin-03".to_vec());

        store.link(mannequin_1, body, rel::HAS_PART, 100, 0);
        store.link(mannequin_1, head_front, rel::HAS_POSE, 100, 0);
        store.link(mannequin_2, body, rel::HAS_PART, 100, 0);
        store.link(mannequin_2, head_left, rel::HAS_POSE, 100, 0);
        store.link(mannequin_3, body, rel::HAS_PART, 100, 0);
        store.link(mannequin_3, head_right, rel::HAS_POSE, 100, 0);

        // Body has refcount 1 (put) + 3 (links) = 4
        assert_eq!(store.get(body).unwrap().refcount, 4);
    }

    #[test]
    fn delta_reconstructs_correctly_xor() {
        let mut store = AtomStore::new();
        let template = store.put(AtomKind::ImageFrame, vec![10, 20, 30, 40, 50]);
        // Delta changes byte at pos 2 from 30 → 35 (XOR = 0x21)
        let delta_bytes = vec![0, 0, 0x21 ^ 0, 0, 0];
        let delta = store.put_delta(template, delta_bytes, DiffMethod::XorBytes, 5);
        let reconstructed = store.reconstruct(delta).unwrap();
        assert_eq!(reconstructed, vec![10, 20, 30 ^ 0x21, 40, 50]);
    }

    #[test]
    fn delta_reconstructs_sparse() {
        let mut store = AtomStore::new();
        let template = store.put(AtomKind::ImageFrame, vec![0u8; 100]);
        // Sparse diff: position 5 = 0xFF, position 50 = 0x80
        let mut delta_bytes = Vec::new();
        delta_bytes.extend_from_slice(&5u32.to_le_bytes());
        delta_bytes.push(0xFF);
        delta_bytes.extend_from_slice(&50u32.to_le_bytes());
        delta_bytes.push(0x80);
        let delta = store.put_delta(template, delta_bytes, DiffMethod::Sparse, 100);
        let r = store.reconstruct(delta).unwrap();
        assert_eq!(r[5], 0xFF);
        assert_eq!(r[50], 0x80);
        assert_eq!(r[0], 0);
    }

    #[test]
    fn outgoing_returns_all_edges() {
        let mut store = AtomStore::new();
        let a = store.put(AtomKind::Concept, b"A".to_vec());
        let b = store.put(AtomKind::Concept, b"B".to_vec());
        let c = store.put(AtomKind::Concept, b"C".to_vec());
        store.link(a, b, rel::HAS_PART, 50, 0);
        store.link(a, c, rel::IS_A, 80, 0);
        let out = store.outgoing(a);
        assert_eq!(out.len(), 2);
    }

    #[test]
    fn compression_ratio_computed() {
        let mut store = AtomStore::new();
        let payload: Vec<u8> = (0..1000).map(|i| (i % 256) as u8).collect();
        for _ in 0..10 {
            store.put(AtomKind::ImageFrame, payload.clone());
        }
        let s = store.stats();
        // Storage of 1000 bytes actually, 9000 saved
        assert_eq!(s.atom_count, 1);
        assert_eq!(s.bytes_saved_dedup, 9000);
        assert!(s.compression_ratio() >= 9.0);
    }

    #[test]
    fn delta_to_self_returns_template() {
        let mut store = AtomStore::new();
        let t = store.put(AtomKind::ImageFrame, vec![1, 2, 3, 4]);
        let d = store.put_delta(t, vec![], DiffMethod::XorBytes, 4);
        assert_eq!(d, t, "empty delta should collapse to template");
    }
}
