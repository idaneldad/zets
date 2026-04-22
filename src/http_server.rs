//! Minimal HTTP/1.1 server for ZETS — zero external dependencies.
//!
//! Built on `std::net::TcpListener` + manual request parsing.
//! Supports: GET/POST, JSON bodies, Content-Length, simple path routing.
//! Does NOT support: keep-alive, chunked transfer, HTTP/2, TLS.
//!
//! This is enough for local multi-client peer-talking on localhost.
//!
//! Usage:
//! ```no_run
//! use zets::http_server::{HttpServer, Request, Response};
//!
//! let mut srv = HttpServer::bind("127.0.0.1:3251")?;
//! srv.on("GET", "/health", |_req| {
//!     Response::json(200, r#"{"ok":true}"#.to_string())
//! });
//! srv.on("POST", "/ask", |req| {
//!     // req.body has the JSON
//!     Response::json(200, format!(r#"{{"echo":{}}}"#, req.body.len()))
//! });
//! srv.run()?;
//! # Ok::<(), std::io::Error>(())
//! ```

use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::Arc;
use std::thread;

pub struct Request {
    pub method: String,
    pub path: String,
    pub query: String, // everything after '?'
    pub headers: Vec<(String, String)>,
    pub body: String,
}

impl Request {
    pub fn header(&self, name: &str) -> Option<&str> {
        let lower = name.to_ascii_lowercase();
        self.headers
            .iter()
            .find(|(k, _)| k.to_ascii_lowercase() == lower)
            .map(|(_, v)| v.as_str())
    }
}

pub struct Response {
    pub status: u16,
    pub body: String,
    pub content_type: String,
    pub extra_headers: Vec<(String, String)>,
}

impl Response {
    pub fn json(status: u16, body: String) -> Self {
        Self {
            status,
            body,
            content_type: "application/json; charset=utf-8".to_string(),
            extra_headers: Vec::new(),
        }
    }

    pub fn text(status: u16, body: String) -> Self {
        Self {
            status,
            body,
            content_type: "text/plain; charset=utf-8".to_string(),
            extra_headers: Vec::new(),
        }
    }

    pub fn not_found() -> Self {
        Self::json(404, r#"{"error":"not found"}"#.to_string())
    }

    pub fn bad_request(msg: &str) -> Self {
        Self::json(
            400,
            format!(r#"{{"error":{}}}"#, json_string(msg)),
        )
    }

    fn write_to<W: Write>(&self, mut w: W) -> std::io::Result<()> {
        let reason = match self.status {
            200 => "OK",
            204 => "No Content",
            400 => "Bad Request",
            404 => "Not Found",
            500 => "Internal Server Error",
            _ => "OK",
        };
        let bytes = self.body.as_bytes();
        write!(
            w,
            "HTTP/1.1 {} {}\r\n\
             Content-Type: {}\r\n\
             Content-Length: {}\r\n\
             Access-Control-Allow-Origin: *\r\n\
             Connection: close\r\n",
            self.status,
            reason,
            self.content_type,
            bytes.len()
        )?;
        for (k, v) in &self.extra_headers {
            write!(w, "{}: {}\r\n", k, v)?;
        }
        w.write_all(b"\r\n")?;
        w.write_all(bytes)?;
        Ok(())
    }
}

/// Utility: escape a string as a JSON string literal (including surrounding quotes).
pub fn json_string(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => {
                out.push_str(&format!("\\u{:04x}", c as u32));
            }
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

type Handler = Box<dyn Fn(&Request) -> Response + Send + Sync + 'static>;

pub struct HttpServer {
    listener: TcpListener,
    routes: Vec<(String, String, Handler)>,
}

impl HttpServer {
    pub fn bind(addr: &str) -> std::io::Result<Self> {
        Ok(Self {
            listener: TcpListener::bind(addr)?,
            routes: Vec::new(),
        })
    }

    pub fn on<F>(&mut self, method: &str, path: &str, f: F)
    where
        F: Fn(&Request) -> Response + Send + Sync + 'static,
    {
        self.routes
            .push((method.to_string(), path.to_string(), Box::new(f)));
    }

    pub fn run(self) -> std::io::Result<()> {
        let routes = Arc::new(self.routes);
        for stream in self.listener.incoming() {
            let stream = match stream {
                Ok(s) => s,
                Err(_) => continue,
            };
            let routes = routes.clone();
            thread::spawn(move || {
                let _ = handle_connection(stream, routes);
            });
        }
        Ok(())
    }
}

fn handle_connection(
    mut stream: TcpStream,
    routes: Arc<Vec<(String, String, Handler)>>,
) -> std::io::Result<()> {
    stream.set_read_timeout(Some(std::time::Duration::from_secs(30)))?;
    stream.set_write_timeout(Some(std::time::Duration::from_secs(30)))?;

    let req = match parse_request(&mut stream) {
        Ok(r) => r,
        Err(e) => {
            let _ = Response::bad_request(&format!("parse error: {}", e)).write_to(&mut stream);
            return Ok(());
        }
    };

    // Route: match method+path; also allow /path/ to match /path
    let path_only = &req.path;
    let handler = routes
        .iter()
        .find(|(m, p, _)| m == &req.method && (p == path_only || format!("{}/", p) == *path_only));

    let response = match handler {
        Some((_, _, h)) => h(&req),
        None => Response::not_found(),
    };
    response.write_to(&mut stream)
}

fn parse_request(stream: &mut TcpStream) -> Result<Request, String> {
    let mut reader = BufReader::new(stream.try_clone().map_err(|e| e.to_string())?);

    // Request line
    let mut line = String::new();
    reader
        .read_line(&mut line)
        .map_err(|e| format!("read line: {}", e))?;
    let parts: Vec<&str> = line.trim().split_whitespace().collect();
    if parts.len() < 2 {
        return Err("malformed request line".into());
    }
    let method = parts[0].to_string();
    let (path, query) = match parts[1].split_once('?') {
        Some((p, q)) => (p.to_string(), q.to_string()),
        None => (parts[1].to_string(), String::new()),
    };

    // Headers
    let mut headers = Vec::new();
    let mut content_length: usize = 0;
    loop {
        let mut line = String::new();
        reader
            .read_line(&mut line)
            .map_err(|e| format!("read header: {}", e))?;
        let line = line.trim_end_matches(&['\r', '\n'][..]);
        if line.is_empty() {
            break;
        }
        if let Some((k, v)) = line.split_once(':') {
            let k = k.trim().to_string();
            let v = v.trim().to_string();
            if k.eq_ignore_ascii_case("content-length") {
                content_length = v.parse().unwrap_or(0);
            }
            headers.push((k, v));
        }
    }

    // Body
    let mut body = String::new();
    if content_length > 0 {
        // Cap at 10MB to prevent DoS
        let cap = std::cmp::min(content_length, 10 * 1024 * 1024);
        let mut buf = vec![0u8; cap];
        reader
            .read_exact(&mut buf)
            .map_err(|e| format!("read body: {}", e))?;
        body = String::from_utf8_lossy(&buf).to_string();
    }

    Ok(Request {
        method,
        path,
        query,
        headers,
        body,
    })
}

// ═══════════════════════════════════════════════════════════════════════
// Tiny JSON field extractor (no external deps)
// ═══════════════════════════════════════════════════════════════════════

/// Extract a string field from a JSON blob: `{"foo":"bar"}` → `Some("bar")`.
/// Naive — assumes well-formed JSON. Good enough for our known request shapes.
pub fn json_get_str<'a>(json: &'a str, key: &str) -> Option<&'a str> {
    let pat = format!("\"{}\"", key);
    let start = json.find(&pat)?;
    let after = &json[start + pat.len()..];
    // skip whitespace + colon
    let after = after.trim_start();
    let after = after.strip_prefix(':')?.trim_start();
    let after = after.strip_prefix('"')?;
    let end = find_unescaped_quote(after)?;
    Some(&after[..end])
}

/// Extract a numeric field.
pub fn json_get_num(json: &str, key: &str) -> Option<f64> {
    let pat = format!("\"{}\"", key);
    let start = json.find(&pat)?;
    let after = &json[start + pat.len()..];
    let after = after.trim_start();
    let after = after.strip_prefix(':')?.trim_start();
    let end = after
        .find(|c: char| c == ',' || c == '}' || c == ']' || c.is_whitespace())
        .unwrap_or(after.len());
    after[..end].parse().ok()
}

fn find_unescaped_quote(s: &str) -> Option<usize> {
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'\\' {
            i += 2;
            continue;
        }
        if bytes[i] == b'"' {
            return Some(i);
        }
        i += 1;
    }
    None
}

// ═══════════════════════════════════════════════════════════════════════
// Tests
// ═══════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn json_string_escapes() {
        assert_eq!(json_string("hello"), r#""hello""#);
        assert_eq!(json_string("a\"b"), r#""a\"b""#);
        assert_eq!(json_string("a\\b"), r#""a\\b""#);
        assert_eq!(json_string("a\nb"), r#""a\nb""#);
    }

    #[test]
    fn json_get_str_works() {
        let j = r#"{"name":"Idan","age":49}"#;
        assert_eq!(json_get_str(j, "name"), Some("Idan"));
        assert_eq!(json_get_str(j, "missing"), None);
    }

    #[test]
    fn json_get_num_works() {
        let j = r#"{"ttl":3,"conf":0.85}"#;
        assert_eq!(json_get_num(j, "ttl"), Some(3.0));
        assert_eq!(json_get_num(j, "conf"), Some(0.85));
    }

    #[test]
    fn json_get_str_with_hebrew() {
        let j = r#"{"name":"עידן","city":"בת ים"}"#;
        assert_eq!(json_get_str(j, "name"), Some("עידן"));
        assert_eq!(json_get_str(j, "city"), Some("בת ים"));
    }
}
