//! Value — the runtime values that flow on the VM stack.
//!
//! Deliberately small and typed: no dynamic objects, no heaps of junk.

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Nil,
    Int(i64),
    Float(f64),
    Bool(bool),
    String(String),
    ConceptId(u32),
    List(Vec<Value>),
}

impl Value {
    pub fn as_int(&self) -> Option<i64> {
        if let Self::Int(v) = self { Some(*v) } else { None }
    }

    pub fn as_string(&self) -> Option<&str> {
        if let Self::String(v) = self { Some(v.as_str()) } else { None }
    }

    pub fn as_concept_id(&self) -> Option<u32> {
        if let Self::ConceptId(v) = self { Some(*v) } else { None }
    }

    pub fn as_bool(&self) -> bool {
        match self {
            Self::Bool(b) => *b,
            Self::Nil => false,
            Self::Int(0) => false,
            Self::String(s) => !s.is_empty(),
            Self::ConceptId(0) => false,
            _ => true,
        }
    }

    pub fn is_nil(&self) -> bool {
        matches!(self, Self::Nil)
    }

    pub fn type_name(&self) -> &'static str {
        match self {
            Self::Nil => "nil",
            Self::Int(_) => "int",
            Self::Float(_) => "float",
            Self::Bool(_) => "bool",
            Self::String(_) => "string",
            Self::ConceptId(_) => "concept_id",
            Self::List(_) => "list",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn as_int_returns_some() {
        assert_eq!(Value::Int(42).as_int(), Some(42));
        assert_eq!(Value::String("foo".into()).as_int(), None);
    }

    #[test]
    fn bool_coercion() {
        assert!(Value::Bool(true).as_bool());
        assert!(!Value::Nil.as_bool());
        assert!(!Value::Int(0).as_bool());
        assert!(Value::Int(42).as_bool());
        assert!(Value::String("hi".into()).as_bool());
        assert!(!Value::String("".into()).as_bool());
    }
}
