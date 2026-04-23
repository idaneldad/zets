//! zets_v4_server — HTTP server שטוען v4 snapshot ומשרת /ask.
//!
//! Usage: zets_v4_server <snapshot.zv4> [--port 3148]
//!
//! Endpoints:
//!   GET  /health        → {"ok":true,"atoms":N,"edges":N,"snapshot":"..."}
//!   POST /ask           → body: {"q":"question"} → {top_articles, top_sentences, time_ms}
//!   GET  /stats         → detailed stats

use std::sync::Arc;
use std::time::Instant;
use zets::graph_v4::{answer, compute_idf, load, phrases_from_graph, AtomKind, Graph};
use zets::graph_v4::retrieve::{IdfTable};
use zets::graph_v4::phrase::PhraseMap;
use zets::http_server::{HttpServer, Response};

struct Context {
    g: Graph,
    idf: IdfTable,
    phrases: PhraseMap,
    snapshot_path: String,
}

fn json_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 16);
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

/// Extract string from JSON like {"q":"..."}. זהה ל-wiki_reader::extract_string_field.
fn json_get(body: &str, key: &str) -> Option<String> {
    let k = format!("\"{}\"", key);
    let key_idx = body.find(&k)?;
    let after = &body[key_idx + k.len()..];
    let colon = after.find(':')?;
    let mut rest = &after[colon+1..];
    while let Some(c) = rest.chars().next() {
        if c.is_whitespace() { rest = &rest[c.len_utf8()..]; } else { break; }
    }
    if !rest.starts_with('"') { return None; }
    rest = &rest[1..];
    let mut out = String::new();
    let mut chars = rest.chars();
    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.next()? {
                '"' => out.push('"'),
                '\\' => out.push('\\'),
                'n' => out.push('\n'),
                't' => out.push('\t'),
                'r' => out.push('\r'),
                'u' => {
                    let hex: String = chars.by_ref().take(4).collect();
                    if let Ok(code) = u32::from_str_radix(&hex, 16) {
                        if let Some(ch) = char::from_u32(code) { out.push(ch); }
                    }
                }
                other => out.push(other),
            }
        } else if c == '"' { return Some(out); }
        else { out.push(c); }
    }
    None
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let av: Vec<String> = std::env::args().collect();
    if av.len() < 2 {
        eprintln!("Usage: zets_v4_server <snapshot.zv4> [--port N]");
        std::process::exit(1);
    }
    let path = av[1].clone();
    let mut port: u16 = 3148;
    let mut i = 2;
    while i < av.len() {
        if av[i] == "--port" && i + 1 < av.len() {
            port = av[i+1].parse().unwrap_or(3148);
            i += 2;
        } else { i += 1; }
    }

    println!("loading snapshot: {}", path);
    let t = Instant::now();
    let mut g = load(&path)?;
    g.build_indexes();
    let idf = compute_idf(&g);
    let phrases = phrases_from_graph(&g);
    println!("  loaded {} atoms, {} edges in {:.1}s",
             g.atom_count(), g.edge_count(), t.elapsed().as_secs_f32());

    let ctx = Arc::new(Context { g, idf, phrases, snapshot_path: path });

    let mut srv = HttpServer::bind(&format!("0.0.0.0:{}", port))?;

    // ─── /health ───
    let c = ctx.clone();
    srv.on("GET", "/health", move |_| {
        let s = c.g.stats();
        let body = format!(
            r#"{{"ok":true,"atoms":{},"edges":{},"articles":{},"phrases":{},"sentences":{},"snapshot":{}}}"#,
            s.atoms_total, s.edges_total,
            s.by_kind[AtomKind::Article as usize],
            s.by_kind[AtomKind::Phrase as usize],
            s.by_kind[AtomKind::Sentence as usize],
            json_escape(&c.snapshot_path),
        );
        Response::json(200, body)
    });

    // ─── /stats ───
    let c = ctx.clone();
    srv.on("GET", "/stats", move |_| {
        let s = c.g.stats();
        let mut body = String::from("{");
        body.push_str(&format!(r#""atoms_total":{},"edges_total":{},"#, s.atoms_total, s.edges_total));
        body.push_str(r#""by_kind":{"#);
        for k in 0..4 {
            let name = AtomKind::from_byte(k as u8).unwrap().name();
            body.push_str(&format!(r#""{}":{}"#, name, s.by_kind[k]));
            if k < 3 { body.push(','); }
        }
        body.push_str("},\"by_rel\":{");
        for r in 0..9 {
            let name = zets::graph_v4::Relation::from_byte(r as u8).unwrap().name();
            body.push_str(&format!(r#""{}":{}"#, name, s.by_rel[r]));
            if r < 7 { body.push(','); }
        }
        body.push_str("}}");
        Response::json(200, body)
    });

    // ─── /ask ───
    let c = ctx.clone();
    srv.on("POST", "/ask", move |req| {
        let q = match json_get(&req.body, "q").or_else(|| json_get(&req.body, "question")) {
            Some(x) => x,
            None => return Response::bad_request("q or question required"),
        };
        let t = Instant::now();
        let ans = answer(&q, &c.g, &c.idf, &c.phrases, 5, 5);
        let dt_ms = t.elapsed().as_millis();

        // build JSON response
        let mut body = String::from("{");
        body.push_str(&format!(r#""question":{},"#, json_escape(&q)));
        body.push_str(&format!(r#""time_ms":{},"#, dt_ms));

        body.push_str(r#""tokens":["#);
        for (i, t) in ans.tokens.iter().enumerate() {
            body.push_str(&json_escape(t));
            if i + 1 < ans.tokens.len() { body.push(','); }
        }
        body.push_str("],");

        body.push_str(r#""seeds":["#);
        for (i, (k, key)) in ans.seeds.iter().enumerate() {
            body.push_str(&format!(r#"{{"kind":"{}","key":{}}}"#, k.name(), json_escape(key)));
            if i + 1 < ans.seeds.len() { body.push(','); }
        }
        body.push_str("],");

        body.push_str(r#""top_articles":["#);
        for (i, (name, score)) in ans.top_articles.iter().enumerate() {
            body.push_str(&format!(r#"{{"name":{},"score":{:.3}}}"#, json_escape(name), score));
            if i + 1 < ans.top_articles.len() { body.push(','); }
        }
        body.push_str("],");

        body.push_str(r#""top_sentences":["#);
        for (i, (text, score, key)) in ans.top_sentences.iter().enumerate() {
            body.push_str(&format!(r#"{{"text":{},"score":{:.3},"sentence_key":{}}}"#,
                                   json_escape(text), score, json_escape(key)));
            if i + 1 < ans.top_sentences.len() { body.push(','); }
        }
        body.push_str("]}");

        Response::json(200, body)
    });

    println!("listening on 0.0.0.0:{}", port);
    println!("  GET  /health");
    println!("  GET  /stats");
    println!("  POST /ask    body: {{\"q\":\"your question\"}}");
    srv.run()?;
    Ok(())
}
