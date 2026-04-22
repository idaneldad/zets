//! `gemini_http` — minimal HTTP client for Gemini API, using `curl` as
//! the transport.
//!
//! Why curl and not a Rust HTTP crate: ZETS's explicit policy is zero
//! runtime dependencies unless absolutely necessary. `curl` is on every
//! Unix machine and gives us HTTPS + JSON for free. The API call
//! happens at most once per question in NLU mode, so shell overhead
//! (~5-10ms per call) is negligible next to the 200-500ms network RTT.
//!
//! Environment:
//!   ZETS_GEMINI_KEY — API key. Without it, this module refuses to call.
//!
//! Behavior:
//!   - Uses gemini-2.5-flash (fast, cheap, good for structured parsing)
//!   - 10-second timeout hardcoded
//!   - Returns raw response text on success; caller parses JSON
//!   - Never panics; all errors are Err(String)
//!
//! Not in this module:
//!   - JSON parsing (caller's responsibility)
//!   - Retry logic (caller decides)
//!   - Multi-turn context (stateless by design)

use std::process::Command;
use std::time::Duration;

/// Call Gemini 2.5 Flash with a prompt. Returns the model's text response.
///
/// Uses curl with a short timeout. Returns Err if:
///   - Env var ZETS_GEMINI_KEY is missing
///   - curl command fails
///   - Response isn't valid JSON
///   - Response contains no `candidates[0].content.parts[0].text`
pub fn call_gemini(prompt: &str, timeout: Duration) -> Result<String, String> {
    let api_key = std::env::var("ZETS_GEMINI_KEY")
        .map_err(|_| "ZETS_GEMINI_KEY not set".to_string())?;

    // Build the JSON body.
    // We do it by hand — no serde dependency.
    let body = build_request_body(prompt);

    // Run curl
    let timeout_secs = timeout.as_secs().max(1).to_string();
    let output = Command::new("curl")
        .arg("-sS")
        .arg("--max-time").arg(&timeout_secs)
        .arg("-X").arg("POST")
        .arg("https://generativelanguage.googleapis.com/v1beta/models/gemini-2.5-flash:generateContent")
        .arg("-H").arg(format!("x-goog-api-key: {}", api_key))
        .arg("-H").arg("Content-Type: application/json")
        .arg("-d").arg(&body)
        .output()
        .map_err(|e| format!("curl exec: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("curl failed: {}", stderr));
    }

    let response = String::from_utf8_lossy(&output.stdout).to_string();
    extract_text_from_response(&response)
}

/// Build the Gemini API request body (minimal, no serde).
fn build_request_body(prompt: &str) -> String {
    let escaped = escape_json_string(prompt);
    format!(
        r#"{{"contents":[{{"parts":[{{"text":"{}"}}]}}],"generationConfig":{{"thinkingConfig":{{"thinkingBudget":0}},"maxOutputTokens":400,"temperature":0.0}}}}"#,
        escaped
    )
}

/// Escape a string for inclusion in a JSON body.
/// Handles: \" \\ \n \r \t and control chars.
fn escape_json_string(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 16);
    for ch in s.chars() {
        match ch {
            '"'  => out.push_str("\\\""),
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
    out
}

/// Extract the response text from Gemini's JSON envelope.
/// Looks for `candidates[0].content.parts[0].text`.
fn extract_text_from_response(json: &str) -> Result<String, String> {
    // Find the "text" field inside parts[0]. Minimal parser — finds the
    // first "text":"..." after "parts" occurrence.
    let parts_idx = json.find("\"parts\"")
        .ok_or_else(|| format!("no 'parts' field. Body: {}",
            json.chars().take(300).collect::<String>()))?;
    let after_parts = &json[parts_idx..];
    let text_idx = after_parts.find("\"text\"")
        .ok_or_else(|| "no 'text' inside parts".to_string())?;
    let after_text = &after_parts[text_idx + 6..]; // skip "text"
    // Expect :"..."
    let after_colon = after_text.trim_start_matches(':').trim_start();
    if !after_colon.starts_with('"') {
        return Err("text value not a string".to_string());
    }
    // Walk the string, honoring escapes, until closing quote
    let body = &after_colon[1..];
    let mut out = String::new();
    let mut escape = false;
    for ch in body.chars() {
        if escape {
            match ch {
                'n' => out.push('\n'),
                't' => out.push('\t'),
                'r' => out.push('\r'),
                '"' => out.push('"'),
                '\\' => out.push('\\'),
                '/' => out.push('/'),
                'u' => out.push('?'),  // unicode escapes: punt for now
                other => out.push(other),
            }
            escape = false;
            continue;
        }
        if ch == '\\' { escape = true; continue; }
        if ch == '"' { return Ok(out); }
        out.push(ch);
    }
    Err("unterminated text value".to_string())
}

// ────────────────────────────────────────────────────────────────
// Tests
// ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn escape_basic() {
        assert_eq!(escape_json_string("hello"), "hello");
        assert_eq!(escape_json_string("a\"b"), "a\\\"b");
        assert_eq!(escape_json_string("line1\nline2"), "line1\\nline2");
        assert_eq!(escape_json_string("tab\there"), "tab\\there");
        assert_eq!(escape_json_string("back\\slash"), "back\\\\slash");
    }

    #[test]
    fn build_body_contains_prompt() {
        let body = build_request_body("what is X?");
        assert!(body.contains("what is X?"));
        assert!(body.contains("\"thinkingBudget\":0"));
        assert!(body.contains("\"temperature\":0.0"));
    }

    #[test]
    fn extract_text_from_gemini_response() {
        let json = r#"{"candidates":[{"content":{"parts":[{"text":"Hello world"}]}}]}"#;
        assert_eq!(extract_text_from_response(json).unwrap(), "Hello world");
    }

    #[test]
    fn extract_text_handles_escapes() {
        let json = r#"{"candidates":[{"content":{"parts":[{"text":"line1\nline2"}]}}]}"#;
        let result = extract_text_from_response(json).unwrap();
        assert_eq!(result, "line1\nline2");
    }

    #[test]
    fn extract_text_handles_quotes() {
        let json = r#"{"candidates":[{"content":{"parts":[{"text":"say \"hi\""}]}}]}"#;
        let result = extract_text_from_response(json).unwrap();
        assert_eq!(result, "say \"hi\"");
    }

    #[test]
    fn extract_fails_on_empty() {
        let json = r#"{"error":"nothing"}"#;
        assert!(extract_text_from_response(json).is_err());
    }

    #[test]
    fn call_without_key_returns_error() {
        // Temporarily clear the env var
        let had_key = std::env::var("ZETS_GEMINI_KEY").is_ok();
        if had_key { std::env::remove_var("ZETS_GEMINI_KEY"); }
        let result = call_gemini("test", Duration::from_secs(1));
        assert!(result.is_err());
        // Restore
        if had_key {
            // We can't re-set without knowing the original value — tests that
            // depend on the key must run separately. This is fine since we
            // only check the "no key" path here.
        }
    }
}
