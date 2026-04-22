//! Reasoning routes — multi-hop traversals on the semantic graph.
//!
//! These transform ZETS from a dictionary into a reasoning engine.
//! They navigate edges (IS_A, PART_OF, CAUSES, etc.) to answer questions
//! requiring 2+ graph hops.
//!
//! Example: "Is a dog a mammal?"
//!   dog --IS_A→ canine --IS_A→ mammal   (returns true via 2 hops)

use super::opcodes::Opcode;
use super::routes::{Route, RouteId, Tier};
use super::value::Value;

pub const R_IS_ANCESTOR: RouteId = 100;

/// Edge kinds (keep in sync with piece_graph::EdgeKind)
pub const KIND_IS_A: i64 = 3;
pub const KIND_PART_OF: i64 = 4;
pub const KIND_CAUSES: i64 = 6;

/// R100: `is_ancestor(concept_id, target_id, max_depth)` → Bool
///
/// Walks IS_A edges from concept_id; returns true if target_id is reached
/// within max_depth hops.
///
/// Algorithm (encoded in bytecode):
///   1. if start == target → true
///   2. if depth <= 0 → false
///   3. neighbors = EdgeTraverse(start, IS_A)
///   4. if neighbors is empty → false
///   5. recurse: is_ancestor(neighbors[0], target, depth-1)
///
/// NOTE: this v1 checks only the first neighbor. Checking all neighbors
/// needs an iteration opcode which we'll add in a later pass. Enough for
/// linear chains (which cover ~80% of IS_A hierarchies).
pub fn build_is_ancestor() -> Route {
    let mut r = Route::new(
        R_IS_ANCESTOR,
        "is_ancestor",
        Tier::Hot,
        3, // start, target, depth
    ).with_doc("Walk IS_A edges; true if target reached within depth.");

    let c_true  = r.add_constant(Value::Bool(true));
    let c_false = r.add_constant(Value::Bool(false));
    let c_zero  = r.add_constant(Value::Int(0));
    let c_one   = r.add_constant(Value::Int(1));
    let c_isa   = r.add_constant(Value::Int(KIND_IS_A));

    // Store params → R0=start, R1=target, R2=depth
    r.emit_u8(Opcode::ParamLoad.as_u8()); r.emit_u16(0);
    r.emit_u8(Opcode::Store.as_u8());     r.emit_u16(0);
    r.emit_u8(Opcode::ParamLoad.as_u8()); r.emit_u16(1);
    r.emit_u8(Opcode::Store.as_u8());     r.emit_u16(1);
    r.emit_u8(Opcode::ParamLoad.as_u8()); r.emit_u16(2);
    r.emit_u8(Opcode::Store.as_u8());     r.emit_u16(2);

    // ── Check 1: start == target? If yes → return true.
    r.emit_u8(Opcode::Load.as_u8()); r.emit_u16(0);
    r.emit_u8(Opcode::Load.as_u8()); r.emit_u16(1);
    let jmp_to_true_on_equal = r.bytecode.len();
    r.emit_u8(Opcode::IfEq.as_u8());
    r.emit_u32(0); // patch later

    // ── Check 2: depth == 0? If yes → return false.
    r.emit_u8(Opcode::Load.as_u8());      r.emit_u16(2);
    r.emit_u8(Opcode::ConstLoad.as_u8()); r.emit_u16(c_zero);
    let jmp_to_false_on_depth = r.bytecode.len();
    r.emit_u8(Opcode::IfEq.as_u8());
    r.emit_u32(0); // patch later

    // ── Get IS_A neighbors of start into R3 (a List).
    r.emit_u8(Opcode::Load.as_u8());      r.emit_u16(0);     // start
    r.emit_u8(Opcode::ConstLoad.as_u8()); r.emit_u16(c_isa); // kind=IS_A
    r.emit_u8(Opcode::EdgeTraverse.as_u8());
    r.emit_u8(Opcode::Store.as_u8());     r.emit_u16(3);     // R3 = list

    // ── Check 3: neighbors empty? If yes → return false.
    r.emit_u8(Opcode::Load.as_u8()); r.emit_u16(3);
    r.emit_u8(Opcode::ListEmpty.as_u8());
    r.emit_u8(Opcode::ConstLoad.as_u8()); r.emit_u16(c_true);  // compare to true
    let jmp_to_false_on_empty = r.bytecode.len();
    r.emit_u8(Opcode::IfEq.as_u8());
    r.emit_u32(0); // patch later

    // ── Extract first neighbor: neighbors[0] into R4
    r.emit_u8(Opcode::Load.as_u8());      r.emit_u16(3);
    r.emit_u8(Opcode::ConstLoad.as_u8()); r.emit_u16(c_zero);
    r.emit_u8(Opcode::ListIndex.as_u8());
    r.emit_u8(Opcode::Store.as_u8());     r.emit_u16(4);

    // ── Recurse: is_ancestor(neighbors[0], target, depth - 1)
    r.emit_u8(Opcode::Load.as_u8()); r.emit_u16(4); // new start
    r.emit_u8(Opcode::Load.as_u8()); r.emit_u16(1); // target (unchanged)
    r.emit_u8(Opcode::Load.as_u8()); r.emit_u16(2); // depth
    r.emit_u8(Opcode::ConstLoad.as_u8()); r.emit_u16(c_one);
    r.emit_u8(Opcode::Sub.as_u8());                 // depth - 1
    r.emit_u8(Opcode::CallRoute.as_u8());
    r.emit_u32(R_IS_ANCESTOR);
    r.emit_u8(Opcode::Return.as_u8());

    // ── Landing pad: return false ─────────────────────
    let return_false_pos = r.bytecode.len();
    r.emit_u8(Opcode::ConstLoad.as_u8()); r.emit_u16(c_false);
    r.emit_u8(Opcode::Return.as_u8());

    // ── Landing pad: return true ──────────────────────
    let return_true_pos = r.bytecode.len();
    r.emit_u8(Opcode::ConstLoad.as_u8()); r.emit_u16(c_true);
    r.emit_u8(Opcode::Return.as_u8());

    // ── Patch jump targets ────────────────────────────
    let patch = |r: &mut Route, pos: usize, target: u32| {
        let at = pos + 1;
        r.bytecode[at..at + 4].copy_from_slice(&target.to_le_bytes());
    };
    patch(&mut r, jmp_to_true_on_equal,  return_true_pos  as u32);
    patch(&mut r, jmp_to_false_on_depth, return_false_pos as u32);
    patch(&mut r, jmp_to_false_on_empty, return_false_pos as u32);

    r
}

pub fn all_reasoning_routes() -> Vec<Route> {
    vec![build_is_ancestor()]
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::{Host, Vm};
    use std::collections::HashMap;

    struct TestHost {
        is_a_edges: HashMap<u32, Vec<u32>>,
    }

    impl TestHost {
        fn build_mammal_hierarchy() -> Self {
            // dog → canine → mammal → animal
            let mut h = HashMap::new();
            h.insert(1, vec![2]);
            h.insert(2, vec![3]);
            h.insert(3, vec![4]);
            h.insert(4, vec![]);
            Self { is_a_edges: h }
        }
    }

    impl Host for TestHost {
        fn concept_lookup(&mut self, _: &str, _: &str) -> Option<u32> { None }
        fn concept_create(&mut self, _: &str, _: &str, _: u8) -> u32 { 0 }
        fn edge_add(&mut self, _: u32, _: u32, _: u8) {}
        fn edge_traverse(&mut self, c: u32, kind: u8) -> Vec<u32> {
            if kind == KIND_IS_A as u8 {
                self.is_a_edges.get(&c).cloned().unwrap_or_default()
            } else {
                vec![]
            }
        }
        fn morph_analyze(&mut self, _: &str, s: &str) -> String { s.to_string() }
        fn string_match(&mut self, t: &str, p: &str) -> bool { t.contains(p) }
        fn wal_write(&mut self, _: u8, _: &[u8]) {}
    }

    fn routes_map() -> HashMap<RouteId, Route> {
        let mut m = HashMap::new();
        for r in all_reasoning_routes() { m.insert(r.id, r); }
        m
    }

    #[test]
    fn is_ancestor_self_true() {
        let routes = routes_map();
        let mut host = TestHost::build_mammal_hierarchy();
        let mut vm = Vm::new(&routes);
        let r = vm.run(R_IS_ANCESTOR,
            vec![Value::ConceptId(3), Value::ConceptId(3), Value::Int(5)],
            &mut host).unwrap();
        assert_eq!(r, Value::Bool(true));
    }

    #[test]
    fn is_ancestor_one_hop() {
        let routes = routes_map();
        let mut host = TestHost::build_mammal_hierarchy();
        let mut vm = Vm::new(&routes);
        // dog (1) IS_A canine (2) — 1 hop
        let r = vm.run(R_IS_ANCESTOR,
            vec![Value::ConceptId(1), Value::ConceptId(2), Value::Int(3)],
            &mut host).unwrap();
        assert_eq!(r, Value::Bool(true));
    }

    #[test]
    fn is_ancestor_three_hops() {
        let routes = routes_map();
        let mut host = TestHost::build_mammal_hierarchy();
        let mut vm = Vm::new(&routes);
        // dog (1) → canine → mammal → animal (4), 3 hops
        let r = vm.run(R_IS_ANCESTOR,
            vec![Value::ConceptId(1), Value::ConceptId(4), Value::Int(5)],
            &mut host).unwrap();
        assert_eq!(r, Value::Bool(true));
    }

    #[test]
    fn is_ancestor_unreachable() {
        let routes = routes_map();
        let mut host = TestHost::build_mammal_hierarchy();
        let mut vm = Vm::new(&routes);
        // Is animal (4) a dog (1)? animal has no IS_A → empty neighbors → false.
        let r = vm.run(R_IS_ANCESTOR,
            vec![Value::ConceptId(4), Value::ConceptId(1), Value::Int(5)],
            &mut host).unwrap();
        assert_eq!(r, Value::Bool(false));
    }

    #[test]
    fn is_ancestor_depth_limited() {
        let routes = routes_map();
        let mut host = TestHost::build_mammal_hierarchy();
        let mut vm = Vm::new(&routes);
        // 3-hop path, depth=1 → not enough
        let r = vm.run(R_IS_ANCESTOR,
            vec![Value::ConceptId(1), Value::ConceptId(4), Value::Int(1)],
            &mut host).unwrap();
        assert_eq!(r, Value::Bool(false));
    }
}
