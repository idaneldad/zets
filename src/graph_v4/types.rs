//! graph_v4::types — Atoms + Edges + Graph מבנה.
//!
//! מודל:
//!   Atom = Word | Phrase | Sentence | Article
//!   Edge = (from, to, relation, weight, pos)
//!
//! Relations:
//!   FillsSlot   — word/phrase ב-position N בתוך sentence (מסלול מדויק)
//!   Next        — unit → הבא אחריו במשפט
//!   PartOf      — word part_of phrase (member-of)
//!   HasPart     — phrase → words שמרכיבים אותו
//!   ContainedIn — sentence → article
//!   HasSentence — article → sentence (reverse)
//!   CoOccurs    — word ↔ word באותו משפט (רחב, לא-מכוון)
//!
//! ראה: docs/50_working/v4_path_graph.py (reference implementation)

use std::collections::HashMap;

pub type AtomId = u32;

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AtomKind {
    Word = 0,
    Phrase = 1,
    Sentence = 2,
    Article = 3,
}

impl AtomKind {
    pub fn from_byte(b: u8) -> Option<AtomKind> {
        match b {
            0 => Some(AtomKind::Word),
            1 => Some(AtomKind::Phrase),
            2 => Some(AtomKind::Sentence),
            3 => Some(AtomKind::Article),
            _ => None,
        }
    }
    pub fn name(self) -> &'static str {
        match self {
            AtomKind::Word => "word",
            AtomKind::Phrase => "phrase",
            AtomKind::Sentence => "sentence",
            AtomKind::Article => "article",
        }
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Relation {
    FillsSlot = 0,
    Next = 1,
    PartOf = 2,
    HasPart = 3,
    ContainedIn = 4,
    HasSentence = 5,
    CoOccurs = 6,
    PartOfBackref = 7,
}

impl Relation {
    pub fn from_byte(b: u8) -> Option<Relation> {
        match b {
            0 => Some(Relation::FillsSlot),
            1 => Some(Relation::Next),
            2 => Some(Relation::PartOf),
            3 => Some(Relation::HasPart),
            4 => Some(Relation::ContainedIn),
            5 => Some(Relation::HasSentence),
            6 => Some(Relation::CoOccurs),
            7 => Some(Relation::PartOfBackref),
            _ => None,
        }
    }
    pub fn name(self) -> &'static str {
        match self {
            Relation::FillsSlot => "fills_slot",
            Relation::Next => "next",
            Relation::PartOf => "part_of",
            Relation::HasPart => "has_part",
            Relation::ContainedIn => "contained_in",
            Relation::HasSentence => "has_sentence",
            Relation::CoOccurs => "co_occurs",
            Relation::PartOfBackref => "part_of_backref",
        }
    }
}

/// Atom payload: תוכן יצוגי (string key) + אופציונלית טקסט מקורי (sentences).
#[derive(Debug, Clone)]
pub struct Atom {
    pub kind: AtomKind,
    pub key: String,       // 'gravity' / 'albert einstein' / 'Gravity:0' / 'Gravity'
    pub text: Option<String>, // רק ל-sentences: הטקסט המקורי המלא
    pub count: u32,        // רק ל-phrases: כמה פעמים הופיע ב-corpus
}

/// Edge: (from, to) עם relation, weight, ו-position.
#[derive(Debug, Clone, Copy)]
pub struct Edge {
    pub from: AtomId,
    pub to: AtomId,
    pub relation: Relation,
    pub weight: u8,
    pub pos: u16,
}

/// הגרף — idempotent atom registry + list of edges + indexes (ייבנו אחרי build).
pub struct Graph {
    // atom storage
    pub atoms: Vec<Atom>,
    // idempotent lookup: (kind, key) → atom_id
    pub by_key: HashMap<(AtomKind, String), AtomId>,
    // edge storage
    pub edges: Vec<Edge>,
    // indexes (None עד שקוראים ל-build_indexes)
    pub out_by_rel: Option<Vec<Vec<Vec<(AtomId, u8, u16)>>>>, // [atom_id][rel as usize] → [(to, w, pos)]
    pub in_by_rel: Option<Vec<Vec<Vec<(AtomId, u8, u16)>>>>,
}

impl Graph {
    pub fn new() -> Self {
        Graph {
            atoms: Vec::new(),
            by_key: HashMap::new(),
            edges: Vec::new(),
            out_by_rel: None,
            in_by_rel: None,
        }
    }

    /// idempotent — אם atom עם (kind, key) קיים, מחזיר את ה-id הקיים.
    pub fn atom(&mut self, kind: AtomKind, key: &str) -> AtomId {
        let k = (kind, key.to_string());
        if let Some(&id) = self.by_key.get(&k) {
            return id;
        }
        let id = self.atoms.len() as AtomId;
        self.atoms.push(Atom {
            kind,
            key: key.to_string(),
            text: None,
            count: 0,
        });
        self.by_key.insert(k, id);
        id
    }

    pub fn set_text(&mut self, id: AtomId, text: String) {
        self.atoms[id as usize].text = Some(text);
    }

    pub fn inc_count(&mut self, id: AtomId) {
        self.atoms[id as usize].count += 1;
    }

    pub fn edge(&mut self, from: AtomId, to: AtomId, rel: Relation, weight: u8, pos: u16) {
        self.edges.push(Edge { from, to, relation: rel, weight, pos });
    }

    pub fn atom_count(&self) -> usize { self.atoms.len() }
    pub fn edge_count(&self) -> usize { self.edges.len() }

    /// מחשב adjacency indexes — חובה לקרוא לפני retrieval.
    pub fn build_indexes(&mut self) {
        let n = self.atoms.len();
        let num_rels = 8_usize;
        let mut out: Vec<Vec<Vec<(AtomId, u8, u16)>>> =
            (0..n).map(|_| (0..num_rels).map(|_| Vec::new()).collect()).collect();
        let mut inn: Vec<Vec<Vec<(AtomId, u8, u16)>>> =
            (0..n).map(|_| (0..num_rels).map(|_| Vec::new()).collect()).collect();
        for e in &self.edges {
            out[e.from as usize][e.relation as usize].push((e.to, e.weight, e.pos));
            inn[e.to as usize][e.relation as usize].push((e.from, e.weight, e.pos));
        }
        self.out_by_rel = Some(out);
        self.in_by_rel = Some(inn);
    }

    pub fn stats(&self) -> Stats {
        let mut by_kind = [0usize; 4];
        for a in &self.atoms {
            by_kind[a.kind as usize] += 1;
        }
        let mut by_rel = [0usize; 8];
        for e in &self.edges {
            by_rel[e.relation as usize] += 1;
        }
        Stats {
            atoms_total: self.atoms.len(),
            by_kind,
            edges_total: self.edges.len(),
            by_rel,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Stats {
    pub atoms_total: usize,
    pub by_kind: [usize; 4],
    pub edges_total: usize,
    pub by_rel: [usize; 8],
}

impl Stats {
    pub fn print(&self) {
        println!("  atoms: {:>10}", self.atoms_total);
        for k in 0..4 {
            let name = AtomKind::from_byte(k as u8).unwrap().name();
            println!("    {:12}: {:>10}", name, self.by_kind[k]);
        }
        println!("  edges: {:>10}", self.edges_total);
        for r in 0..8 {
            let name = Relation::from_byte(r as u8).unwrap().name();
            println!("    {:16}: {:>10}", name, self.by_rel[r]);
        }
    }
}
