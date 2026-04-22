//! Route — a sequence of opcodes that defines a procedure.
//!
//! A Route is the unit of execution in the system graph.
//! It has:
//!   - A unique route_id
//!   - A tier (Hot/Warm/Cold/Archive) for lazy loading
//!   - A bytecode buffer
//!   - A constant pool (strings, numbers used by opcodes)
//!   - Parameter signature (how many args it expects)

use super::value::Value;

/// Tier — determines when a route is loaded.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Tier {
    /// Always in RAM (~50-100 core routes, ~100 KB total)
    Hot = 0,
    /// Loaded on first use (~500 per-lang/domain extensions)
    Warm = 1,
    /// mmap-lazy (~5000 specialized routes)
    Cold = 2,
    /// Encrypted bundle, decrypted on demand (rare domains)
    Archive = 3,
}

impl Tier {
    pub fn from_u8(v: u8) -> Self {
        match v {
            0 => Self::Hot,
            1 => Self::Warm,
            2 => Self::Cold,
            _ => Self::Archive,
        }
    }
    pub fn as_u8(self) -> u8 {
        self as u8
    }
}

pub type RouteId = u32;

/// A Route — a procedure in the system graph.
#[derive(Debug, Clone)]
pub struct Route {
    pub id: RouteId,
    pub name: String,       // for human debugging
    pub tier: Tier,
    pub bytecode: Vec<u8>,
    pub constants: Vec<Value>,  // constant pool
    pub param_count: u8,
    pub doc: String,        // human-readable description
}

impl Route {
    pub fn new(id: RouteId, name: &str, tier: Tier, param_count: u8) -> Self {
        Self {
            id,
            name: name.to_string(),
            tier,
            bytecode: Vec::new(),
            constants: Vec::new(),
            param_count,
            doc: String::new(),
        }
    }

    pub fn with_doc(mut self, doc: &str) -> Self {
        self.doc = doc.to_string();
        self
    }

    pub fn add_constant(&mut self, v: Value) -> u16 {
        let idx = self.constants.len() as u16;
        self.constants.push(v);
        idx
    }

    pub fn emit_u8(&mut self, b: u8) {
        self.bytecode.push(b);
    }

    pub fn emit_u16(&mut self, v: u16) {
        self.bytecode.extend_from_slice(&v.to_le_bytes());
    }

    pub fn emit_u32(&mut self, v: u32) {
        self.bytecode.extend_from_slice(&v.to_le_bytes());
    }

    pub fn byte_count(&self) -> usize {
        self.bytecode.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tier_roundtrip() {
        for t in [Tier::Hot, Tier::Warm, Tier::Cold, Tier::Archive] {
            assert_eq!(Tier::from_u8(t.as_u8()), t);
        }
    }

    #[test]
    fn route_constants_have_indices() {
        let mut r = Route::new(1, "test", Tier::Hot, 0);
        let i0 = r.add_constant(Value::String("hello".into()));
        let i1 = r.add_constant(Value::Int(42));
        assert_eq!(i0, 0);
        assert_eq!(i1, 1);
        assert_eq!(r.constants.len(), 2);
    }

    #[test]
    fn route_emit_writes_bytecode() {
        let mut r = Route::new(1, "test", Tier::Hot, 0);
        r.emit_u8(0x42);
        r.emit_u16(0x1234);
        assert_eq!(r.bytecode, vec![0x42, 0x34, 0x12]);
    }
}
