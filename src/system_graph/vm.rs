//! The VM — runs bytecode routes.
//!
//! Key features:
//!   - Stack-based execution (simple, fast dispatch)
//!   - Bounded recursion (max_call_depth = 32)
//!   - Bounded execution (max_ops_per_run = 10_000)
//!   - No dynamic allocation in hot path (registers are fixed-size)
//!
//! Safety: the VM CAN NOT escape into Rust code other than the designated
//! handlers (morphology, concept lookup, WAL write, etc). This guarantees
//! that bytecode loaded from an untrusted source can only do what the
//! opcodes allow.

use std::collections::HashMap;

use super::opcodes::Opcode;
use super::routes::{Route, RouteId};
use super::value::Value;

pub const MAX_CALL_DEPTH: usize = 32;
pub const MAX_OPS_PER_RUN: usize = 10_000;
pub const NUM_REGISTERS: usize = 16;

#[derive(Debug, Clone)]
pub enum VmError {
    StackUnderflow,
    StackOverflow,
    TypeError { expected: &'static str, got: &'static str },
    InvalidOpcode(u8),
    InvalidConstant(u16),
    InvalidRegister(u16),
    InvalidJumpTarget(u32),
    RouteNotFound(RouteId),
    CallDepthExceeded,
    OpLimitExceeded,
    MissingParam(u16),
    NotImplemented(Opcode),
}

impl std::fmt::Display for VmError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

/// Host callback — the VM delegates concept/morphology/etc to these.
/// This keeps the VM itself agnostic of the data graph.
pub trait Host {
    fn concept_lookup(&mut self, lang: &str, surface: &str) -> Option<u32>;
    fn concept_create(&mut self, anchor: &str, gloss: &str, pos: u8) -> u32;
    fn edge_add(&mut self, source: u32, target: u32, kind: u8);
    fn edge_traverse(&mut self, concept_id: u32, kind: u8) -> Vec<u32>;
    fn morph_analyze(&mut self, lang: &str, surface: &str) -> String;
    fn string_match(&mut self, text: &str, pattern: &str) -> bool;
    fn wal_write(&mut self, kind: u8, payload: &[u8]);
}

pub struct Vm<'a> {
    stack: Vec<Value>,
    registers: [Value; NUM_REGISTERS],
    call_depth: usize,
    ops_executed: usize,
    routes: &'a HashMap<RouteId, Route>,
}

impl<'a> Vm<'a> {
    pub fn new(routes: &'a HashMap<RouteId, Route>) -> Self {
        Self {
            stack: Vec::with_capacity(64),
            registers: std::array::from_fn(|_| Value::Nil),
            call_depth: 0,
            ops_executed: 0,
            routes,
        }
    }

    /// Execute a route with given params. Returns top-of-stack on Return.
    pub fn run<H: Host>(
        &mut self,
        route_id: RouteId,
        params: Vec<Value>,
        host: &mut H,
    ) -> Result<Value, VmError> {
        let route = self.routes.get(&route_id).ok_or(VmError::RouteNotFound(route_id))?;
        self.run_route(route, params, host)
    }

    fn run_route<H: Host>(
        &mut self,
        route: &Route,
        params: Vec<Value>,
        host: &mut H,
    ) -> Result<Value, VmError> {
        if self.call_depth >= MAX_CALL_DEPTH {
            return Err(VmError::CallDepthExceeded);
        }
        self.call_depth += 1;

        let mut pc = 0usize;
        let bc = &route.bytecode;
        let local_params = params;

        while pc < bc.len() {
            self.ops_executed += 1;
            if self.ops_executed > MAX_OPS_PER_RUN {
                self.call_depth -= 1;
                return Err(VmError::OpLimitExceeded);
            }

            let op_byte = bc[pc];
            pc += 1;
            let op = Opcode::from_u8(op_byte).ok_or(VmError::InvalidOpcode(op_byte))?;

            match op {
                Opcode::Noop => {}
                Opcode::Return => {
                    let top = self.stack.pop().unwrap_or(Value::Nil);
                    self.call_depth -= 1;
                    return Ok(top);
                }
                Opcode::ConstLoad => {
                    let idx = u16::from_le_bytes([bc[pc], bc[pc + 1]]);
                    pc += 2;
                    let v = route.constants.get(idx as usize)
                        .ok_or(VmError::InvalidConstant(idx))?.clone();
                    self.push(v)?;
                }
                Opcode::ParamLoad => {
                    let idx = u16::from_le_bytes([bc[pc], bc[pc + 1]]);
                    pc += 2;
                    let v = local_params.get(idx as usize)
                        .ok_or(VmError::MissingParam(idx))?.clone();
                    self.push(v)?;
                }
                Opcode::Store => {
                    let idx = u16::from_le_bytes([bc[pc], bc[pc + 1]]);
                    pc += 2;
                    if (idx as usize) >= NUM_REGISTERS {
                        return Err(VmError::InvalidRegister(idx));
                    }
                    self.registers[idx as usize] = self.pop()?;
                }
                Opcode::Load => {
                    let idx = u16::from_le_bytes([bc[pc], bc[pc + 1]]);
                    pc += 2;
                    if (idx as usize) >= NUM_REGISTERS {
                        return Err(VmError::InvalidRegister(idx));
                    }
                    self.push(self.registers[idx as usize].clone())?;
                }
                Opcode::Jump => {
                    let tgt = u32::from_le_bytes([bc[pc], bc[pc + 1], bc[pc + 2], bc[pc + 3]]);
                    pc = tgt as usize;
                }
                Opcode::IfEq => {
                    let tgt = u32::from_le_bytes([bc[pc], bc[pc + 1], bc[pc + 2], bc[pc + 3]]);
                    pc += 4;
                    let b = self.pop()?;
                    let a = self.pop()?;
                    if a == b {
                        pc = tgt as usize;
                    }
                }
                Opcode::IfNe => {
                    let tgt = u32::from_le_bytes([bc[pc], bc[pc + 1], bc[pc + 2], bc[pc + 3]]);
                    pc += 4;
                    let b = self.pop()?;
                    let a = self.pop()?;
                    if a != b {
                        pc = tgt as usize;
                    }
                }
                Opcode::CallRoute => {
                    let target_id = u32::from_le_bytes([bc[pc], bc[pc + 1], bc[pc + 2], bc[pc + 3]]);
                    pc += 4;
                    let target = self.routes.get(&target_id)
                        .ok_or(VmError::RouteNotFound(target_id))?;
                    // Pop params from stack (target.param_count items)
                    let n = target.param_count as usize;
                    if self.stack.len() < n {
                        return Err(VmError::StackUnderflow);
                    }
                    let split_at = self.stack.len() - n;
                    let call_params: Vec<Value> = self.stack.split_off(split_at);
                    let result = self.run_route(target, call_params, host)?;
                    self.push(result)?;
                }
                Opcode::ConceptLookup => {
                    let surface = self.pop_string()?;
                    let lang = self.pop_string()?;
                    let id = host.concept_lookup(&lang, &surface).unwrap_or(0);
                    self.push(Value::ConceptId(id))?;
                }
                Opcode::ConceptCreate => {
                    let pos = self.pop_int()? as u8;
                    let gloss = self.pop_string()?;
                    let anchor = self.pop_string()?;
                    let id = host.concept_create(&anchor, &gloss, pos);
                    self.push(Value::ConceptId(id))?;
                }
                Opcode::EdgeAdd => {
                    let kind = self.pop_int()? as u8;
                    let target = self.pop_concept()?;
                    let source = self.pop_concept()?;
                    host.edge_add(source, target, kind);
                }
                Opcode::EdgeTraverse => {
                    let kind = self.pop_int()? as u8;
                    let cid = self.pop_concept()?;
                    let targets = host.edge_traverse(cid, kind);
                    self.push(Value::List(
                        targets.into_iter().map(Value::ConceptId).collect(),
                    ))?;
                }
                Opcode::MorphAnalyze => {
                    let surface = self.pop_string()?;
                    let lang = self.pop_string()?;
                    let lemma = host.morph_analyze(&lang, &surface);
                    self.push(Value::String(lemma))?;
                }
                Opcode::StringMatch => {
                    let pat = self.pop_string()?;
                    let txt = self.pop_string()?;
                    let ok = host.string_match(&txt, &pat);
                    self.push(Value::Bool(ok))?;
                }
                Opcode::StringSplit => {
                    let sep = self.pop_string()?;
                    let txt = self.pop_string()?;
                    let parts: Vec<Value> = txt.split(sep.as_str())
                        .map(|s| Value::String(s.to_string())).collect();
                    self.push(Value::List(parts))?;
                }
                Opcode::WalWrite => {
                    let payload_val = self.pop()?;
                    let kind = self.pop_int()? as u8;
                    let payload = match payload_val {
                        Value::String(s) => s.into_bytes(),
                        _ => vec![],
                    };
                    host.wal_write(kind, &payload);
                }
                Opcode::ConfidenceSet | Opcode::TierCheck => {
                    // Metadata ops — pop value, ignore for now (used for reflection)
                    let _ = self.pop()?;
                }

                // ── Stack manipulation (NEW) ────────────────────────────
                Opcode::Dup => {
                    let top = self.stack.last().ok_or(VmError::StackUnderflow)?.clone();
                    self.push(top)?;
                }
                Opcode::Swap => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    self.push(b)?;
                    self.push(a)?;
                }
                Opcode::Pop => {
                    let _ = self.pop()?;
                }

                // ── List operations (NEW) ───────────────────────────────
                Opcode::ListIndex => {
                    let n = self.pop_int()?;
                    let list_val = self.pop()?;
                    match list_val {
                        Value::List(items) => {
                            let idx = n as usize;
                            if n < 0 || idx >= items.len() {
                                self.push(Value::Nil)?;
                            } else {
                                self.push(items[idx].clone())?;
                            }
                        }
                        _ => self.push(Value::Nil)?,
                    }
                }
                Opcode::ListLen => {
                    let v = self.pop()?;
                    let len = match v {
                        Value::List(items) => items.len() as i64,
                        Value::String(s) => s.chars().count() as i64,
                        _ => 0,
                    };
                    self.push(Value::Int(len))?;
                }
                Opcode::ListEmpty => {
                    let v = self.pop()?;
                    let empty = match v {
                        Value::List(items) => items.is_empty(),
                        Value::String(s) => s.is_empty(),
                        Value::Nil => true,
                        _ => false,
                    };
                    self.push(Value::Bool(empty))?;
                }

                // ── Arithmetic (NEW) ────────────────────────────────────
                Opcode::Add => {
                    let b = self.pop_int().unwrap_or(0);
                    let a = self.pop_int().unwrap_or(0);
                    self.push(Value::Int(a.wrapping_add(b)))?;
                }
                Opcode::Sub => {
                    let b = self.pop_int().unwrap_or(0);
                    let a = self.pop_int().unwrap_or(0);
                    self.push(Value::Int(a.wrapping_sub(b)))?;
                }
                Opcode::Lt => {
                    let b = self.pop_int().unwrap_or(0);
                    let a = self.pop_int().unwrap_or(0);
                    self.push(Value::Bool(a < b))?;
                }
                Opcode::Gt => {
                    let b = self.pop_int().unwrap_or(0);
                    let a = self.pop_int().unwrap_or(0);
                    self.push(Value::Bool(a > b))?;
                }

                // ── Boolean (NEW) ───────────────────────────────────────
                Opcode::Not => {
                    let v = self.pop()?;
                    self.push(Value::Bool(!v.as_bool()))?;
                }
                Opcode::And => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    self.push(Value::Bool(a.as_bool() && b.as_bool()))?;
                }
                Opcode::Or => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    self.push(Value::Bool(a.as_bool() || b.as_bool()))?;
                }
            }
        }

        // Route ended without Return — return top of stack or Nil
        self.call_depth -= 1;
        Ok(self.stack.pop().unwrap_or(Value::Nil))
    }

    fn push(&mut self, v: Value) -> Result<(), VmError> {
        if self.stack.len() >= 1024 {
            return Err(VmError::StackOverflow);
        }
        self.stack.push(v);
        Ok(())
    }

    fn pop(&mut self) -> Result<Value, VmError> {
        self.stack.pop().ok_or(VmError::StackUnderflow)
    }

    fn pop_string(&mut self) -> Result<String, VmError> {
        let v = self.pop()?;
        match v {
            Value::String(s) => Ok(s),
            other => Err(VmError::TypeError { expected: "string", got: other.type_name() }),
        }
    }

    fn pop_int(&mut self) -> Result<i64, VmError> {
        let v = self.pop()?;
        match v {
            Value::Int(i) => Ok(i),
            other => Err(VmError::TypeError { expected: "int", got: other.type_name() }),
        }
    }

    fn pop_concept(&mut self) -> Result<u32, VmError> {
        let v = self.pop()?;
        match v {
            Value::ConceptId(c) => Ok(c),
            Value::Int(i) => Ok(i as u32),
            other => Err(VmError::TypeError { expected: "concept_id", got: other.type_name() }),
        }
    }

    pub fn ops_executed(&self) -> usize {
        self.ops_executed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Mock host for tests
    struct MockHost {
        concepts: HashMap<(String, String), u32>,
        next_id: u32,
        edges: Vec<(u32, u32, u8)>,
        wal_records: Vec<(u8, Vec<u8>)>,
    }

    impl MockHost {
        fn new() -> Self {
            let mut c = HashMap::new();
            c.insert(("en".to_string(), "dog".to_string()), 100);
            c.insert(("en".to_string(), "molecule".to_string()), 200);
            Self {
                concepts: c,
                next_id: 1000,
                edges: Vec::new(),
                wal_records: Vec::new(),
            }
        }
    }

    impl Host for MockHost {
        fn concept_lookup(&mut self, lang: &str, surface: &str) -> Option<u32> {
            self.concepts.get(&(lang.to_string(), surface.to_string())).copied()
        }
        fn concept_create(&mut self, _anchor: &str, _gloss: &str, _pos: u8) -> u32 {
            let id = self.next_id; self.next_id += 1; id
        }
        fn edge_add(&mut self, s: u32, t: u32, k: u8) {
            self.edges.push((s, t, k));
        }
        fn edge_traverse(&mut self, _c: u32, _k: u8) -> Vec<u32> { vec![] }
        fn morph_analyze(&mut self, _lang: &str, surface: &str) -> String {
            // Trivial: strip trailing 's'
            if surface.ends_with('s') && surface.len() > 3 {
                surface[..surface.len() - 1].to_string()
            } else {
                surface.to_string()
            }
        }
        fn string_match(&mut self, text: &str, pattern: &str) -> bool {
            text.contains(pattern)
        }
        fn wal_write(&mut self, k: u8, p: &[u8]) {
            self.wal_records.push((k, p.to_vec()));
        }
    }

    fn build_routes(r: Route) -> HashMap<RouteId, Route> {
        let mut m = HashMap::new();
        m.insert(r.id, r);
        m
    }

    #[test]
    fn vm_return_const() {
        let mut r = Route::new(1, "test", super::super::routes::Tier::Hot, 0);
        let c = r.add_constant(Value::Int(42));
        r.emit_u8(Opcode::ConstLoad.as_u8());
        r.emit_u16(c);
        r.emit_u8(Opcode::Return.as_u8());
        let routes = build_routes(r);
        let mut host = MockHost::new();
        let mut vm = Vm::new(&routes);
        let result = vm.run(1, vec![], &mut host).unwrap();
        assert_eq!(result, Value::Int(42));
    }

    #[test]
    fn vm_concept_lookup() {
        // Route: lookup("en", "dog") -> returns concept_id
        let mut r = Route::new(1, "lookup_dog", super::super::routes::Tier::Hot, 0);
        let c_lang = r.add_constant(Value::String("en".into()));
        let c_surf = r.add_constant(Value::String("dog".into()));
        r.emit_u8(Opcode::ConstLoad.as_u8()); r.emit_u16(c_lang);
        r.emit_u8(Opcode::ConstLoad.as_u8()); r.emit_u16(c_surf);
        r.emit_u8(Opcode::ConceptLookup.as_u8());
        r.emit_u8(Opcode::Return.as_u8());
        let routes = build_routes(r);
        let mut host = MockHost::new();
        let mut vm = Vm::new(&routes);
        let result = vm.run(1, vec![], &mut host).unwrap();
        assert_eq!(result, Value::ConceptId(100));
    }

    #[test]
    fn vm_params_are_accessible() {
        let mut r = Route::new(1, "identity", super::super::routes::Tier::Hot, 1);
        r.emit_u8(Opcode::ParamLoad.as_u8()); r.emit_u16(0);
        r.emit_u8(Opcode::Return.as_u8());
        let routes = build_routes(r);
        let mut host = MockHost::new();
        let mut vm = Vm::new(&routes);
        let result = vm.run(1, vec![Value::String("hello".into())], &mut host).unwrap();
        assert_eq!(result, Value::String("hello".into()));
    }

    #[test]
    fn vm_call_route_composes() {
        // Inner route: takes param, returns it
        let mut inner = Route::new(2, "identity", super::super::routes::Tier::Hot, 1);
        inner.emit_u8(Opcode::ParamLoad.as_u8()); inner.emit_u16(0);
        inner.emit_u8(Opcode::Return.as_u8());

        // Outer route: push 42, call inner(42), return result
        let mut outer = Route::new(1, "caller", super::super::routes::Tier::Hot, 0);
        let c42 = outer.add_constant(Value::Int(42));
        outer.emit_u8(Opcode::ConstLoad.as_u8()); outer.emit_u16(c42);
        outer.emit_u8(Opcode::CallRoute.as_u8()); outer.emit_u32(2);
        outer.emit_u8(Opcode::Return.as_u8());

        let mut routes = HashMap::new();
        routes.insert(1, outer);
        routes.insert(2, inner);
        let mut host = MockHost::new();
        let mut vm = Vm::new(&routes);
        let result = vm.run(1, vec![], &mut host).unwrap();
        assert_eq!(result, Value::Int(42));
    }

    #[test]
    fn vm_enforces_call_depth() {
        // Route that calls itself — should hit MAX_CALL_DEPTH
        let mut r = Route::new(1, "recurse", super::super::routes::Tier::Hot, 0);
        r.emit_u8(Opcode::CallRoute.as_u8()); r.emit_u32(1);
        r.emit_u8(Opcode::Return.as_u8());
        let routes = build_routes(r);
        let mut host = MockHost::new();
        let mut vm = Vm::new(&routes);
        let result = vm.run(1, vec![], &mut host);
        assert!(matches!(result, Err(VmError::CallDepthExceeded)));
    }

    #[test]
    fn vm_hearst_pattern_match() {
        // Given: "DNA is a molecule that carries genetic info"
        // Route: test if "is a molecule" is in the text
        let mut r = Route::new(1, "hearst_match", super::super::routes::Tier::Hot, 1);
        let pat = r.add_constant(Value::String("is a molecule".into()));
        r.emit_u8(Opcode::ParamLoad.as_u8()); r.emit_u16(0);
        r.emit_u8(Opcode::ConstLoad.as_u8()); r.emit_u16(pat);
        r.emit_u8(Opcode::StringMatch.as_u8());
        r.emit_u8(Opcode::Return.as_u8());
        let routes = build_routes(r);
        let mut host = MockHost::new();
        let mut vm = Vm::new(&routes);
        let result = vm.run(1, vec![Value::String("DNA is a molecule that carries info".into())], &mut host).unwrap();
        assert_eq!(result, Value::Bool(true));
    }
    #[test]
    fn vm_arithmetic_add() {
        let mut r = Route::new(1, "add", super::super::routes::Tier::Hot, 2);
        r.emit_u8(Opcode::ParamLoad.as_u8()); r.emit_u16(0);
        r.emit_u8(Opcode::ParamLoad.as_u8()); r.emit_u16(1);
        r.emit_u8(Opcode::Add.as_u8());
        r.emit_u8(Opcode::Return.as_u8());
        let routes = build_routes(r);
        let mut host = MockHost::new();
        let mut vm = Vm::new(&routes);
        let result = vm.run(1, vec![Value::Int(3), Value::Int(4)], &mut host).unwrap();
        assert_eq!(result, Value::Int(7));
    }

    #[test]
    fn vm_list_index() {
        let mut r = Route::new(1, "idx", super::super::routes::Tier::Hot, 0);
        let c_list = r.add_constant(Value::List(vec![
            Value::Int(10), Value::Int(20), Value::Int(30),
        ]));
        let c_idx = r.add_constant(Value::Int(1));
        r.emit_u8(Opcode::ConstLoad.as_u8()); r.emit_u16(c_list);
        r.emit_u8(Opcode::ConstLoad.as_u8()); r.emit_u16(c_idx);
        r.emit_u8(Opcode::ListIndex.as_u8());
        r.emit_u8(Opcode::Return.as_u8());
        let routes = build_routes(r);
        let mut host = MockHost::new();
        let mut vm = Vm::new(&routes);
        let result = vm.run(1, vec![], &mut host).unwrap();
        assert_eq!(result, Value::Int(20));
    }

    #[test]
    fn vm_list_index_out_of_range() {
        let mut r = Route::new(1, "idx", super::super::routes::Tier::Hot, 0);
        let c_list = r.add_constant(Value::List(vec![Value::Int(10)]));
        let c_idx = r.add_constant(Value::Int(5));
        r.emit_u8(Opcode::ConstLoad.as_u8()); r.emit_u16(c_list);
        r.emit_u8(Opcode::ConstLoad.as_u8()); r.emit_u16(c_idx);
        r.emit_u8(Opcode::ListIndex.as_u8());
        r.emit_u8(Opcode::Return.as_u8());
        let routes = build_routes(r);
        let mut host = MockHost::new();
        let mut vm = Vm::new(&routes);
        let result = vm.run(1, vec![], &mut host).unwrap();
        assert!(result.is_nil());
    }

    #[test]
    fn vm_list_len() {
        let mut r = Route::new(1, "len", super::super::routes::Tier::Hot, 0);
        let c_list = r.add_constant(Value::List(vec![
            Value::Int(1), Value::Int(2), Value::Int(3), Value::Int(4),
        ]));
        r.emit_u8(Opcode::ConstLoad.as_u8()); r.emit_u16(c_list);
        r.emit_u8(Opcode::ListLen.as_u8());
        r.emit_u8(Opcode::Return.as_u8());
        let routes = build_routes(r);
        let mut host = MockHost::new();
        let mut vm = Vm::new(&routes);
        let result = vm.run(1, vec![], &mut host).unwrap();
        assert_eq!(result, Value::Int(4));
    }

    #[test]
    fn vm_list_empty() {
        let mut r = Route::new(1, "empty", super::super::routes::Tier::Hot, 0);
        let c_list = r.add_constant(Value::List(vec![]));
        r.emit_u8(Opcode::ConstLoad.as_u8()); r.emit_u16(c_list);
        r.emit_u8(Opcode::ListEmpty.as_u8());
        r.emit_u8(Opcode::Return.as_u8());
        let routes = build_routes(r);
        let mut host = MockHost::new();
        let mut vm = Vm::new(&routes);
        let result = vm.run(1, vec![], &mut host).unwrap();
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn vm_comparison_lt() {
        let mut r = Route::new(1, "lt", super::super::routes::Tier::Hot, 2);
        r.emit_u8(Opcode::ParamLoad.as_u8()); r.emit_u16(0);
        r.emit_u8(Opcode::ParamLoad.as_u8()); r.emit_u16(1);
        r.emit_u8(Opcode::Lt.as_u8());
        r.emit_u8(Opcode::Return.as_u8());
        let routes = build_routes(r);
        let mut host = MockHost::new();
        let mut vm = Vm::new(&routes);
        let result = vm.run(1, vec![Value::Int(3), Value::Int(5)], &mut host).unwrap();
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn vm_dup_doubles_top() {
        let mut r = Route::new(1, "dup", super::super::routes::Tier::Hot, 1);
        r.emit_u8(Opcode::ParamLoad.as_u8()); r.emit_u16(0);
        r.emit_u8(Opcode::Dup.as_u8());
        r.emit_u8(Opcode::Add.as_u8());
        r.emit_u8(Opcode::Return.as_u8());
        let routes = build_routes(r);
        let mut host = MockHost::new();
        let mut vm = Vm::new(&routes);
        let result = vm.run(1, vec![Value::Int(7)], &mut host).unwrap();
        assert_eq!(result, Value::Int(14));
    }

    #[test]
    fn vm_swap_reorders() {
        let mut r = Route::new(1, "swap", super::super::routes::Tier::Hot, 2);
        r.emit_u8(Opcode::ParamLoad.as_u8()); r.emit_u16(0);
        r.emit_u8(Opcode::ParamLoad.as_u8()); r.emit_u16(1);
        r.emit_u8(Opcode::Swap.as_u8());
        r.emit_u8(Opcode::Sub.as_u8());
        r.emit_u8(Opcode::Return.as_u8());
        let routes = build_routes(r);
        let mut host = MockHost::new();
        let mut vm = Vm::new(&routes);
        // params: 10, 3 -> swap -> stack is [3, 10] -> sub -> 3 - 10 = -7
        let result = vm.run(1, vec![Value::Int(10), Value::Int(3)], &mut host).unwrap();
        assert_eq!(result, Value::Int(-7));
    }

    #[test]
    fn vm_not_inverts_bool() {
        let mut r = Route::new(1, "not", super::super::routes::Tier::Hot, 1);
        r.emit_u8(Opcode::ParamLoad.as_u8()); r.emit_u16(0);
        r.emit_u8(Opcode::Not.as_u8());
        r.emit_u8(Opcode::Return.as_u8());
        let routes = build_routes(r);
        let mut host = MockHost::new();
        let mut vm = Vm::new(&routes);
        let result = vm.run(1, vec![Value::Bool(true)], &mut host).unwrap();
        assert_eq!(result, Value::Bool(false));
    }
}
