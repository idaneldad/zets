//! Minimal JSON parser — no external dependencies.
//!
//! Parses the subset of JSON used in JSONL calibration files:
//! objects, strings, arrays.

/// A parsed JSON value.
#[derive(Debug, Clone, PartialEq)]
pub enum JsonValue {
    Null,
    Bool(bool),
    Number(f64),
    Str(String),
    Array(Vec<JsonValue>),
    Object(Vec<(String, JsonValue)>),
}

impl JsonValue {
    /// Access a field of a JSON object by key.
    pub fn get(&self, key: &str) -> Option<&JsonValue> {
        if let JsonValue::Object(pairs) = self {
            pairs.iter().find(|(k, _)| k == key).map(|(_, v)| v)
        } else {
            None
        }
    }

    /// Return inner string, if this is a `Str`.
    pub fn as_str(&self) -> Option<&str> {
        if let JsonValue::Str(s) = self { Some(s) } else { None }
    }

    /// Return inner array, if this is an `Array`.
    pub fn as_array(&self) -> Option<&[JsonValue]> {
        if let JsonValue::Array(a) = self { Some(a) } else { None }
    }
}

// ── Parser ────────────────────────────────────────────────────────────────

struct Parser<'a> {
    src: &'a [u8],
    pos: usize,
}

impl<'a> Parser<'a> {
    fn new(s: &'a str) -> Self {
        Parser { src: s.as_bytes(), pos: 0 }
    }

    fn remaining(&self) -> usize {
        self.src.len() - self.pos
    }

    fn peek(&self) -> Option<u8> {
        self.src.get(self.pos).copied()
    }

    fn advance(&mut self) {
        self.pos += 1;
    }

    fn skip_ws(&mut self) {
        while let Some(b) = self.peek() {
            if b == b' ' || b == b'\t' || b == b'\n' || b == b'\r' {
                self.advance();
            } else {
                break;
            }
        }
    }

    fn expect(&mut self, byte: u8) -> Result<(), String> {
        self.skip_ws();
        match self.peek() {
            Some(b) if b == byte => { self.advance(); Ok(()) }
            Some(b) => Err(format!("expected '{}' got '{}' at pos {}", byte as char, b as char, self.pos)),
            None => Err(format!("expected '{}' but got EOF at pos {}", byte as char, self.pos)),
        }
    }

    fn parse_value(&mut self) -> Result<JsonValue, String> {
        self.skip_ws();
        match self.peek() {
            Some(b'"') => self.parse_string().map(JsonValue::Str),
            Some(b'{') => self.parse_object(),
            Some(b'[') => self.parse_array(),
            Some(b't') => { self.pos += 4; Ok(JsonValue::Bool(true)) }
            Some(b'f') => { self.pos += 5; Ok(JsonValue::Bool(false)) }
            Some(b'n') => { self.pos += 4; Ok(JsonValue::Null) }
            Some(b) if b == b'-' || (b'0'..=b'9').contains(&b) => self.parse_number(),
            Some(b) => Err(format!("unexpected char '{}' at pos {}", b as char, self.pos)),
            None => Err("unexpected EOF".to_string()),
        }
    }

    fn parse_string(&mut self) -> Result<String, String> {
        self.expect(b'"')?;
        let mut out = String::new();
        loop {
            match self.peek() {
                None => return Err("unterminated string".to_string()),
                Some(b'"') => { self.advance(); return Ok(out); }
                Some(b'\\') => {
                    self.advance();
                    match self.peek() {
                        Some(b'"')  => { out.push('"');  self.advance(); }
                        Some(b'\\') => { out.push('\\'); self.advance(); }
                        Some(b'/')  => { out.push('/');  self.advance(); }
                        Some(b'n')  => { out.push('\n'); self.advance(); }
                        Some(b't')  => { out.push('\t'); self.advance(); }
                        Some(b'r')  => { out.push('\r'); self.advance(); }
                        Some(b'u')  => {
                            // \uXXXX — decode 4 hex digits
                            self.advance();
                            if self.remaining() < 4 {
                                return Err("truncated \\u escape".to_string());
                            }
                            let hex: String = (0..4)
                                .map(|_| { let c = self.src[self.pos] as char; self.advance(); c })
                                .collect();
                            let code = u32::from_str_radix(&hex, 16)
                                .map_err(|_| format!("bad \\u escape: {hex}"))?;
                            let ch = char::from_u32(code)
                                .ok_or_else(|| format!("invalid codepoint {code}"))?;
                            out.push(ch);
                        }
                        Some(b) => { out.push(b as char); self.advance(); }
                        None => return Err("unterminated escape".to_string()),
                    }
                }
                Some(b) => {
                    // UTF-8: gather multi-byte sequences
                    if b < 0x80 {
                        out.push(b as char);
                        self.advance();
                    } else {
                        // Collect a full UTF-8 sequence
                        let mut buf = vec![b];
                        self.advance();
                        while let Some(next) = self.peek() {
                            if next >= 0x80 && next < 0xC0 {
                                buf.push(next);
                                self.advance();
                            } else {
                                break;
                            }
                        }
                        let s = std::str::from_utf8(&buf)
                            .map_err(|e| format!("utf8 error: {e}"))?;
                        out.push_str(s);
                    }
                }
            }
        }
    }

    fn parse_object(&mut self) -> Result<JsonValue, String> {
        self.expect(b'{')?;
        let mut pairs = Vec::new();
        self.skip_ws();
        if self.peek() == Some(b'}') {
            self.advance();
            return Ok(JsonValue::Object(pairs));
        }
        loop {
            self.skip_ws();
            let key = self.parse_string()?;
            self.expect(b':')?;
            let val = self.parse_value()?;
            pairs.push((key, val));
            self.skip_ws();
            match self.peek() {
                Some(b',') => { self.advance(); }
                Some(b'}') => { self.advance(); break; }
                Some(b) => return Err(format!("expected ',' or '}}' got '{}' at {}", b as char, self.pos)),
                None => return Err("unterminated object".to_string()),
            }
        }
        Ok(JsonValue::Object(pairs))
    }

    fn parse_array(&mut self) -> Result<JsonValue, String> {
        self.expect(b'[')?;
        let mut items = Vec::new();
        self.skip_ws();
        if self.peek() == Some(b']') {
            self.advance();
            return Ok(JsonValue::Array(items));
        }
        loop {
            let val = self.parse_value()?;
            items.push(val);
            self.skip_ws();
            match self.peek() {
                Some(b',') => { self.advance(); }
                Some(b']') => { self.advance(); break; }
                Some(b) => return Err(format!("expected ',' or ']' got '{}' at {}", b as char, self.pos)),
                None => return Err("unterminated array".to_string()),
            }
        }
        Ok(JsonValue::Array(items))
    }

    fn parse_number(&mut self) -> Result<JsonValue, String> {
        let start = self.pos;
        if self.peek() == Some(b'-') { self.advance(); }
        while matches!(self.peek(), Some(b) if b.is_ascii_digit()) { self.advance(); }
        if self.peek() == Some(b'.') {
            self.advance();
            while matches!(self.peek(), Some(b) if b.is_ascii_digit()) { self.advance(); }
        }
        if matches!(self.peek(), Some(b'e') | Some(b'E')) {
            self.advance();
            if matches!(self.peek(), Some(b'+') | Some(b'-')) { self.advance(); }
            while matches!(self.peek(), Some(b) if b.is_ascii_digit()) { self.advance(); }
        }
        let s = std::str::from_utf8(&self.src[start..self.pos])
            .map_err(|e| e.to_string())?;
        let n: f64 = s.parse().map_err(|e: std::num::ParseFloatError| e.to_string())?;
        Ok(JsonValue::Number(n))
    }
}

/// Parse a JSON value from a string slice.
pub fn parse(s: &str) -> Result<JsonValue, String> {
    let mut p = Parser::new(s);
    p.parse_value()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_object() {
        let v = parse(r#"{"id":"easy_001","text":"What?","difficulty":"Easy"}"#).unwrap();
        assert_eq!(v.get("id").and_then(|v| v.as_str()), Some("easy_001"));
        assert_eq!(v.get("text").and_then(|v| v.as_str()), Some("What?"));
    }

    #[test]
    fn test_parse_nested_object() {
        let v = parse(r#"{"expected":{"type":"Exact","value":"Paris"}}"#).unwrap();
        let exp = v.get("expected").unwrap();
        assert_eq!(exp.get("type").and_then(|v| v.as_str()), Some("Exact"));
        assert_eq!(exp.get("value").and_then(|v| v.as_str()), Some("Paris"));
    }

    #[test]
    fn test_parse_array_values() {
        let v = parse(r#"{"values":["a","b","c"]}"#).unwrap();
        let arr = v.get("values").and_then(|v| v.as_array()).unwrap();
        assert_eq!(arr.len(), 3);
        assert_eq!(arr[0].as_str(), Some("a"));
    }

    #[test]
    fn test_parse_hebrew_string() {
        let v = parse(r#"{"text":"מה הבירה של ישראל?"}"#).unwrap();
        assert_eq!(
            v.get("text").and_then(|v| v.as_str()),
            Some("מה הבירה של ישראל?")
        );
    }

    #[test]
    fn test_parse_refuse() {
        let v = parse(r#"{"expected":{"type":"Refuse"}}"#).unwrap();
        let exp = v.get("expected").unwrap();
        assert_eq!(exp.get("type").and_then(|v| v.as_str()), Some("Refuse"));
    }

    #[test]
    fn test_parse_empty_object() {
        let v = parse("{}").unwrap();
        assert!(matches!(v, JsonValue::Object(ref p) if p.is_empty()));
    }

    #[test]
    fn test_parse_escape_sequences() {
        let v = parse(r#"{"a":"line1\nline2"}"#).unwrap();
        assert_eq!(v.get("a").and_then(|v| v.as_str()), Some("line1\nline2"));
    }
}
