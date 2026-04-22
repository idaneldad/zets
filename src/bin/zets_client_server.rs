//! `zets_client_server` — a full Rust ZETS instance serving a single persona.
//!
//! Each instance:
//!   - Loads its own AtomStore (persona-seeded + optional shared baseline)
//!   - Has style parameters that shape the response
//!   - Listens on its port, speaks HTTP/1.1
//!   - Supports peer-to-peer queries with TTL + seen-set (loop prevention)
//!
//! This is the Rust replacement for the Python multi_client prototype.
//!
//! Usage:
//!   cargo run --release --bin zets-client-server -- \
//!       --port 3251 \
//!       --atoms data/clients/idan.atoms \
//!       --persona data/personas/idan.persona.json
//!
//! The persona JSON file has a small schema (see PersonaConfig below).

use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use zets::atom_persist;
use zets::atoms::{AtomId, AtomStore};
use zets::http_server::{json_get_num, json_get_str, json_string, HttpServer, Response};
use zets::inference::infer_attributes;
use zets::learning_layer::ProvenanceLog;
use zets::relations;

// ═══════════════════════════════════════════════════════════════════════
// PersonaConfig — what a persona JSON file looks like
// ═══════════════════════════════════════════════════════════════════════
//
// Minimal schema (hand-parsed, no serde):
// {
//   "name": "Idan",
//   "name_he": "עידן",
//   "lang": "he+en",
//   "bio": "Architect and founder...",
//   "verbosity": 0.7,
//   "formality": 0.4,
//   "warmth": 0.55,
//   "emoji_rate": 0.1,
//   "ask_peer_threshold": 0.5,
//   "strict_verifier": 0.8,
//   "confidence_baseline": 0.7
// }

struct PersonaConfig {
    name: String,
    name_he: String,
    lang: String,
    bio: String,
    verbosity: f64,
    formality: f64,
    warmth: f64,
    emoji_rate: f64,
    ask_peer_threshold: f64,
    strict_verifier: f64,
    confidence_baseline: f64,
}

impl PersonaConfig {
    fn load_or_default(path: &PathBuf) -> Self {
        let default = Self {
            name: "anonymous".into(),
            name_he: "אנונימי".into(),
            lang: "en".into(),
            bio: "unnamed ZETS client".into(),
            verbosity: 0.5,
            formality: 0.5,
            warmth: 0.5,
            emoji_rate: 0.1,
            ask_peer_threshold: 0.4,
            strict_verifier: 0.5,
            confidence_baseline: 0.6,
        };
        let json = match fs::read_to_string(path) {
            Ok(s) => s,
            Err(_) => return default,
        };
        Self {
            name: json_get_str(&json, "name").unwrap_or(&default.name).to_string(),
            name_he: json_get_str(&json, "name_he")
                .unwrap_or(&default.name_he)
                .to_string(),
            lang: json_get_str(&json, "lang").unwrap_or(&default.lang).to_string(),
            bio: json_get_str(&json, "bio").unwrap_or(&default.bio).to_string(),
            verbosity: json_get_num(&json, "verbosity").unwrap_or(default.verbosity),
            formality: json_get_num(&json, "formality").unwrap_or(default.formality),
            warmth: json_get_num(&json, "warmth").unwrap_or(default.warmth),
            emoji_rate: json_get_num(&json, "emoji_rate").unwrap_or(default.emoji_rate),
            ask_peer_threshold: json_get_num(&json, "ask_peer_threshold")
                .unwrap_or(default.ask_peer_threshold),
            strict_verifier: json_get_num(&json, "strict_verifier")
                .unwrap_or(default.strict_verifier),
            confidence_baseline: json_get_num(&json, "confidence_baseline")
                .unwrap_or(default.confidence_baseline),
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════
// ClientState — per-instance mutable state
// ═══════════════════════════════════════════════════════════════════════

struct ClientState {
    persona: PersonaConfig,
    port: u16,
    store: AtomStore,
    prov: ProvenanceLog,
    // Peer registry: (port, name) — populated from CLI or default
    peers: Vec<(u16, String)>,
    // Query log
    queries_served: u64,
    peer_hits: u64,
    cloud_hits: u64,
}

impl ClientState {
    fn new(
        port: u16,
        persona: PersonaConfig,
        store: AtomStore,
        peers: Vec<(u16, String)>,
    ) -> Self {
        Self {
            persona,
            port,
            store,
            prov: ProvenanceLog::new(),
            peers,
            queries_served: 0,
            peer_hits: 0,
            cloud_hits: 0,
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════
// Core query pipeline
// ═══════════════════════════════════════════════════════════════════════

#[derive(Debug)]
struct QueryResult {
    raw_answer: String,
    confidence: f64,
    source: String, // "local" | "local+inference" | "peer:<port>" | "cloud" | "unknown"
    peer_port_used: Option<u16>,
}

/// Match query terms against atom data (case-insensitive substring).
fn local_lookup(store: &AtomStore, q: &str) -> (Vec<AtomId>, f64) {
    let q_lower = q.to_lowercase();
    let mut hits = Vec::new();
    let mut best_conf: f64 = 0.0;

    for id in 0..(store.atom_count() as AtomId) {
        if let Some(atom) = store.get(id) {
            let data = String::from_utf8_lossy(&atom.data);
            let data_lower = data.to_lowercase();
            // Exact match = high confidence
            if data_lower == q_lower {
                hits.push(id);
                best_conf = best_conf.max(0.95);
                continue;
            }
            // Substring match
            if data_lower.contains(&q_lower) || q_lower.contains(&data_lower) {
                if data.len() >= 3 {
                    hits.push(id);
                    best_conf = best_conf.max(0.65);
                }
            }
        }
    }
    (hits, best_conf)
}

fn render_local_answer(store: &AtomStore, hits: &[AtomId]) -> String {
    let mut parts = Vec::new();
    for &h in hits.iter().take(3) {
        if let Some(atom) = store.get(h) {
            let data = String::from_utf8_lossy(&atom.data).to_string();
            // Collect outgoing edges as context
            let edges = store.outgoing(h);
            let mut relations: Vec<String> = Vec::new();
            for e in edges.iter().take(3) {
                if let Some(target) = store.get(e.to) {
                    let rel_name = relations::get(e.relation)
                        .map(|r| r.name.to_string())
                        .unwrap_or_else(|| format!("rel{}", e.relation));
                    let target_text = String::from_utf8_lossy(&target.data).to_string();
                    relations.push(format!("{} {}", rel_name, target_text));
                }
            }
            if relations.is_empty() {
                parts.push(data);
            } else {
                parts.push(format!("{} ({})", data, relations.join(", ")));
            }
        }
    }
    if parts.is_empty() {
        "(no local match)".into()
    } else {
        parts.join(" | ")
    }
}

/// Try to add inference: if hits exist, infer attributes.
fn add_inference(state: &ClientState, hits: &[AtomId]) -> Option<String> {
    if let Some(&h) = hits.first() {
        let inferred = infer_attributes(&state.store, h, 3, Some(&state.prov));
        if !inferred.is_empty() {
            let parts: Vec<String> = inferred
                .iter()
                .take(3)
                .map(|inf| {
                    let attr = state
                        .store
                        .get(inf.attribute)
                        .map(|a| String::from_utf8_lossy(&a.data).to_string())
                        .unwrap_or_default();
                    format!("{} (inferred, conf {})", attr, inf.confidence)
                })
                .collect();
            if !parts.is_empty() {
                return Some(parts.join(", "));
            }
        }
    }
    None
}

// ═══════════════════════════════════════════════════════════════════════
// Persona style — reshape raw answer
// ═══════════════════════════════════════════════════════════════════════

fn style_response(raw: &str, persona: &PersonaConfig, seed: u64) -> String {
    let mut text = raw.trim().to_string();

    // Verbosity trim
    if persona.verbosity < 0.3 && text.len() > 80 {
        let cut = text.find('.').unwrap_or(std::cmp::min(80, text.len()));
        text.truncate(cut + 1);
    }

    // Warmth prefix (HE personas)
    if persona.warmth > 0.80 && persona.lang.starts_with("he") {
        text = format!("שלום! {}", text);
    } else if persona.warmth > 0.80 && persona.lang.starts_with("en") {
        text = format!("Hello! {}", text);
    }

    // Emoji — deterministic based on seed
    let rand_val = pseudo_rand(seed) as f64 / (u64::MAX as f64);
    if rand_val < persona.emoji_rate {
        if persona.emoji_rate > 0.7 {
            text.push_str(" 🌈");
        } else if persona.lang.contains("music") || persona.bio.contains("music") {
            text.push_str(" 🎵");
        } else {
            text.push_str(" ✨");
        }
    }

    // Verbose tail
    if persona.verbosity > 0.85 {
        if persona.lang.starts_with("he") {
            text.push_str(" — אגב, אפשר להעמיק בזה.");
        } else {
            text.push_str(" — by the way, we could go deeper on this.");
        }
    }

    text
}

fn pseudo_rand(seed: u64) -> u64 {
    // Splitmix64
    let mut x = seed.wrapping_add(0x9E3779B97F4A7C15);
    x = (x ^ (x >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
    x = (x ^ (x >> 27)).wrapping_mul(0x94D049BB133111EB);
    x ^ (x >> 31)
}

// ═══════════════════════════════════════════════════════════════════════
// Peer protocol — HTTP POST /peer_ask
// ═══════════════════════════════════════════════════════════════════════

fn pick_peer(peers: &[(u16, String)], q: &str, exclude: &HashSet<u16>, seed: u64) -> Option<u16> {
    let candidates: Vec<u16> = peers
        .iter()
        .filter(|(p, _)| !exclude.contains(p))
        .map(|(p, _)| *p)
        .collect();
    if candidates.is_empty() {
        return None;
    }
    // Weighted: for now, just deterministic pseudo-random
    let _ = q;
    let idx = (pseudo_rand(seed) as usize) % candidates.len();
    Some(candidates[idx])
}

/// HTTP client — POST to a peer (blocking, std::net only).
fn peer_request(
    port: u16,
    q: &str,
    ttl: u32,
    seen: &HashSet<u16>,
    asker: &str,
) -> Result<(String, f64, String), String> {
    use std::io::{Read, Write};
    use std::net::TcpStream;
    use std::time::Duration;

    let mut stream = TcpStream::connect(("127.0.0.1", port))
        .map_err(|e| format!("connect {}: {}", port, e))?;
    stream.set_read_timeout(Some(Duration::from_secs(15))).ok();
    stream.set_write_timeout(Some(Duration::from_secs(15))).ok();

    // Build body: {"q": "...", "ttl": N, "seen": [...], "asker": "..."}
    let seen_list: Vec<String> = seen.iter().map(|p| p.to_string()).collect();
    let body = format!(
        r#"{{"q":{},"ttl":{},"seen":[{}],"asker":{}}}"#,
        json_string(q),
        ttl,
        seen_list.join(","),
        json_string(asker)
    );

    let req = format!(
        "POST /peer_ask HTTP/1.1\r\nHost: localhost:{}\r\n\
         Content-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        port,
        body.as_bytes().len(),
        body
    );
    stream.write_all(req.as_bytes()).map_err(|e| e.to_string())?;

    // Read until EOF
    let mut response = Vec::new();
    stream.read_to_end(&mut response).map_err(|e| e.to_string())?;
    let text = String::from_utf8_lossy(&response).to_string();

    // Split headers from body at "\r\n\r\n"
    let body_start = text.find("\r\n\r\n").ok_or("no body separator")? + 4;
    let body_text = &text[body_start..];

    let answer = json_get_str(body_text, "answer")
        .unwrap_or("")
        .to_string();
    let conf = json_get_num(body_text, "confidence").unwrap_or(0.0);
    let source = json_get_str(body_text, "source")
        .unwrap_or("peer")
        .to_string();
    Ok((answer, conf, source))
}

// ═══════════════════════════════════════════════════════════════════════
// The ask pipeline
// ═══════════════════════════════════════════════════════════════════════

const MAX_TTL: u32 = 4;

fn run_ask(
    state: &mut ClientState,
    q: &str,
    ttl: u32,
    mut seen: HashSet<u16>,
    asker: &str,
) -> QueryResult {
    // Loop detection: if our port is in seen, this question cycled back to us.
    if seen.contains(&state.port) {
        return QueryResult {
            raw_answer: String::new(),
            confidence: 0.0,
            source: "loop_detected".into(),
            peer_port_used: None,
        };
    }
    seen.insert(state.port);

    state.queries_served += 1;

    // 1. Local lookup
    let (hits, mut confidence) = local_lookup(&state.store, q);
    let mut raw = render_local_answer(&state.store, &hits);
    let mut source = if hits.is_empty() {
        "local_miss".to_string()
    } else {
        "local".to_string()
    };

    // 2. If we have a hit, try to add inference for richness
    if !hits.is_empty() {
        if let Some(inf) = add_inference(state, &hits) {
            raw = format!("{} | inferred: {}", raw, inf);
            source = "local+inference".into();
            confidence = f64::min(confidence + 0.1, 1.0);
        }
    }

    // 3. If confidence low AND TTL left, ask a peer
    let mut peer_port_used: Option<u16> = None;
    if confidence < state.persona.ask_peer_threshold && ttl > 0 && !state.peers.is_empty() {
        let seed = now_micros().wrapping_add(state.port as u64);
        if let Some(peer_port) = pick_peer(&state.peers, q, &seen, seed) {
            peer_port_used = Some(peer_port);
            match peer_request(peer_port, q, ttl - 1, &seen, &state.persona.name) {
                Ok((peer_answer, peer_conf, _)) if !peer_answer.is_empty() => {
                    // Judge peer answer strictly
                    let adjusted = (peer_conf - (state.persona.strict_verifier - 0.5) * 0.3)
                        .clamp(0.0, 1.0);
                    if adjusted > confidence {
                        state.peer_hits += 1;
                        raw = format!("{} (via {}: {})", raw, peer_port, peer_answer);
                        confidence = adjusted;
                        source = format!("peer:{}", peer_port);
                    }
                }
                _ => {}
            }
        }
    }

    // 4. Format with persona
    // (done by caller — we return raw)

    QueryResult {
        raw_answer: raw,
        confidence,
        source,
        peer_port_used,
    }
}

fn now_micros() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_micros() as u64)
        .unwrap_or(0)
}

// ═══════════════════════════════════════════════════════════════════════
// HTTP handlers
// ═══════════════════════════════════════════════════════════════════════

fn mount_routes(state: Arc<Mutex<ClientState>>) -> HttpServer {
    let port = state.lock().unwrap().port;
    let mut srv = HttpServer::bind(&format!("127.0.0.1:{}", port))
        .expect("bind failed");

    // GET /health
    {
        let s = state.clone();
        srv.on("GET", "/health", move |_req| {
            let st = s.lock().unwrap();
            let body = format!(
                r#"{{"client":{},"client_he":{},"port":{},"lang":{},"bio":{},"atoms":{},"edges":{},"queries_served":{},"peer_hits":{},"cloud_hits":{}}}"#,
                json_string(&st.persona.name),
                json_string(&st.persona.name_he),
                st.port,
                json_string(&st.persona.lang),
                json_string(&st.persona.bio),
                st.store.atom_count(),
                st.store.edge_count(),
                st.queries_served,
                st.peer_hits,
                st.cloud_hits
            );
            Response::json(200, body)
        });
    }

    // POST /ask  — user query
    {
        let s = state.clone();
        srv.on("POST", "/ask", move |req| {
            let q = match json_get_str(&req.body, "q").or_else(|| json_get_str(&req.body, "question")) {
                Some(x) => x.to_string(),
                None => return Response::bad_request("q required"),
            };
            let result;
            let styled;
            {
                let mut st = s.lock().unwrap();
                result = run_ask(&mut st, &q, MAX_TTL, HashSet::new(), "user");
                styled = style_response(&result.raw_answer, &st.persona, now_micros());
            }
            let body = format!(
                r#"{{"answer":{},"raw":{},"confidence":{:.3},"source":{},"peer_port":{},"q":{}}}"#,
                json_string(&styled),
                json_string(&result.raw_answer),
                result.confidence,
                json_string(&result.source),
                result
                    .peer_port_used
                    .map(|p| p.to_string())
                    .unwrap_or("null".into()),
                json_string(&q)
            );
            Response::json(200, body)
        });
    }

    // POST /peer_ask — peer-to-peer
    {
        let s = state.clone();
        srv.on("POST", "/peer_ask", move |req| {
            let q = match json_get_str(&req.body, "q") {
                Some(x) => x.to_string(),
                None => return Response::bad_request("q required"),
            };
            let ttl = json_get_num(&req.body, "ttl").unwrap_or(0.0) as u32;
            let asker = json_get_str(&req.body, "asker").unwrap_or("?").to_string();
            // Parse seen array — simple: find "seen":[...]
            let mut seen = HashSet::new();
            if let Some(idx) = req.body.find("\"seen\":[") {
                let after = &req.body[idx + 8..];
                if let Some(end) = after.find(']') {
                    for tok in after[..end].split(',') {
                        if let Ok(n) = tok.trim().parse::<u16>() {
                            seen.insert(n);
                        }
                    }
                }
            }

            let result;
            {
                let mut st = s.lock().unwrap();
                result = run_ask(&mut st, &q, ttl, seen, &asker);
            }
            let body = format!(
                r#"{{"answer":{},"confidence":{:.3},"source":{},"q":{}}}"#,
                json_string(&result.raw_answer),
                result.confidence,
                json_string(&result.source),
                json_string(&q)
            );
            Response::json(200, body)
        });
    }

    srv
}

// ═══════════════════════════════════════════════════════════════════════
// CLI + main
// ═══════════════════════════════════════════════════════════════════════

fn parse_args() -> (u16, PathBuf, PathBuf, Vec<(u16, String)>) {
    let args: Vec<String> = std::env::args().collect();
    let mut port: u16 = 3251;
    let mut atoms_path = PathBuf::from("data/clients/default.atoms");
    let mut persona_path = PathBuf::from("data/personas/default.persona.json");
    let mut peers: Vec<(u16, String)> = Vec::new();

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--port" => {
                port = args[i + 1].parse().expect("bad port");
                i += 2;
            }
            "--atoms" => {
                atoms_path = PathBuf::from(&args[i + 1]);
                i += 2;
            }
            "--persona" => {
                persona_path = PathBuf::from(&args[i + 1]);
                i += 2;
            }
            "--peers" => {
                // Format: 3252:Rotem,3253:Bentz,...
                for part in args[i + 1].split(',') {
                    if let Some((p, n)) = part.split_once(':') {
                        if let Ok(pn) = p.parse() {
                            peers.push((pn, n.to_string()));
                        }
                    }
                }
                i += 2;
            }
            _ => i += 1,
        }
    }
    (port, atoms_path, persona_path, peers)
}

fn main() -> std::io::Result<()> {
    let (port, atoms_path, persona_path, peers) = parse_args();

    let persona = PersonaConfig::load_or_default(&persona_path);

    let store = if atoms_path.exists() {
        match atom_persist::load_from_file(&atoms_path) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("  ! atoms load failed: {:?}. Starting with empty store.", e);
                AtomStore::new()
            }
        }
    } else {
        eprintln!("  ! atoms not found at {:?}. Starting empty.", atoms_path);
        AtomStore::new()
    };

    let state = ClientState::new(port, persona, store, peers);
    let peer_count = state.peers.len();
    let atom_count = state.store.atom_count();
    let edge_count = state.store.edge_count();
    let name = state.persona.name.clone();
    let lang = state.persona.lang.clone();

    println!(
        "ZETS client server | {} :{} | lang={} | {} atoms, {} edges | {} peers",
        name, port, lang, atom_count, edge_count, peer_count
    );

    let srv = mount_routes(Arc::new(Mutex::new(state)));
    srv.run()
}
