"""ITER 2 — Focused validation of §28-§43 + §41 Rust code review.
14 models, 6 succeeding minimum target. Real critique requested.
"""
import os, json, time, sys
from pathlib import Path
from concurrent.futures import ThreadPoolExecutor, as_completed
import urllib.request, urllib.error

# Load environment
from dotenv import load_dotenv
load_dotenv('/home/dinio/.env')

ANTHROPIC = os.getenv('ANTHROPIC_API_KEY')
OPENAI    = os.getenv('OPENAI_API_KEY')
GEMINI    = os.getenv('GEMINI_API_KEY') or os.getenv('GEMINI_KEY')
TOGETHER  = os.getenv('TOGETHER_API_KEY') or 'tgp_v1_OomnteXcZqUhvRZO_hO03Uf_jJDXYG7rVHhPGIJ4AtI'

# Read sections §28-§43 only (avoid sending entire 6117 lines)
agi = Path('docs/AGI.md').read_text()
# Find §28 onwards
idx = agi.find('# §28')
spec = agi[idx:] if idx > 0 else agi[-80000:]
# Cap at ~70KB to fit context
spec = spec[:70000]

PROMPT = f"""You are a senior systems architect reviewing the ZETS ASI specification.

CONTEXT: ZETS = "computationally created being" — graph-native AGI/ASI built on
Sefer Yetzirah algorithm. Hebrew-canonical. 6GB RAM target. CPU-only. Deterministic.

This is ITER 2 of a 7-iteration council process. Iter 1 found 5 ABI blockers
(EdgeKind size, Atom Layout, Determinism, AtomKind enum, CSR mutation strategy).

YOUR TASK: Critique §28-§43 below. Specifically:

§28 — 30-year Roadmap + AAR self-improvement
§29 — Failure Modes (F1-F13)
§30 — Tri-Memory architecture
§31 — 13 sub-graphs cryptographic topology
§32 — Beit Midrash Federation (preserves contradictions, NOT CRDT merge)
§33 — Tensor/Graph boundary
§34 — NRNCh"Y 5 layers + §34.4 Yechida=37=Akedah
§35 — Hebrew as canonical thinking substrate
§36 — Storage alternatives (LSM/HTM/Hopfield/Tri-Memory)
§37-§39 — Source anchoring (engineering verdict)
§40 — Core Bootstrap Protocol (4-stage SY 1:9-12 + Isaac 6-step)
§41 — Code-as-Spec (Rust skeleton)
§42 — Bootstrap Content Filling (100K atoms in 7 days)
§43 — Affective Architecture: עונג/נגע inversion guard

REQUIRED OUTPUT FORMAT (be brutal, be specific):

## Architecture Verdict
[overall coherence 1-10, why]

## Top 3 Critical Issues
[concrete problems, line/section refs, suggested fixes]

## Top 3 Strengths Worth Preserving
[what's genuinely novel, why it works]

## §41 Code Review (Rust types)
[bugs, undefined behavior, memory unsafety, performance issues]

## §43 ענג/נגע Architecture Assessment
[Will the inversion guard actually prevent deception? Edge cases?
What attacker model breaks it? How would you defeat your own design?]

## §40 Bootstrap Protocol Assessment
[Is the 4-stage ordering achievable? What can fail mid-bootstrap?
Is verify_homoiconic_root() actually meaningful or circular?]

## Self-Rating
[Your confidence in this critique: 1-10, what would make it 10]

## Falsification Test
[One concrete benchmark that would prove/disprove core claims]

NO FLATTERY. Engineering rigor only. The user is exhausted by sycophancy.

SPEC:
---
{spec}
---
"""

def call_anthropic(model='claude-opus-4-5'):
    """Anthropic API."""
    url = "https://api.anthropic.com/v1/messages"
    body = json.dumps({
        "model": model,
        "max_tokens": 4000,
        "messages": [{"role": "user", "content": PROMPT}]
    }).encode()
    req = urllib.request.Request(url, data=body, headers={
        "x-api-key": ANTHROPIC,
        "anthropic-version": "2023-06-01",
        "Content-Type": "application/json"
    })
    with urllib.request.urlopen(req, timeout=180) as r:
        d = json.loads(r.read())
    return d['content'][0]['text']

def call_openai(model='gpt-5'):
    """OpenAI API."""
    url = "https://api.openai.com/v1/chat/completions"
    body = json.dumps({
        "model": model,
        "messages": [{"role": "user", "content": PROMPT}],
        "max_completion_tokens": 4000,
    }).encode()
    req = urllib.request.Request(url, data=body, headers={
        "Authorization": f"Bearer {OPENAI}",
        "Content-Type": "application/json"
    })
    with urllib.request.urlopen(req, timeout=180) as r:
        d = json.loads(r.read())
    return d['choices'][0]['message']['content']

def call_gemini(model='gemini-2.0-flash-exp'):
    """Google Gemini API."""
    url = f"https://generativelanguage.googleapis.com/v1beta/models/{model}:generateContent?key={GEMINI}"
    body = json.dumps({
        "contents": [{"parts": [{"text": PROMPT}]}],
        "generationConfig": {"maxOutputTokens": 4000}
    }).encode()
    req = urllib.request.Request(url, data=body, headers={"Content-Type": "application/json"})
    with urllib.request.urlopen(req, timeout=180) as r:
        d = json.loads(r.read())
    return d['candidates'][0]['content']['parts'][0]['text']

def call_together(model):
    """Together.ai API. Needs UA Mozilla."""
    url = "https://api.together.xyz/v1/chat/completions"
    body = json.dumps({
        "model": model,
        "messages": [{"role": "user", "content": PROMPT}],
        "max_tokens": 4000,
        "temperature": 0.3,
    }).encode()
    req = urllib.request.Request(url, data=body, headers={
        "Authorization": f"Bearer {TOGETHER}",
        "Content-Type": "application/json",
        "User-Agent": "Mozilla/5.0"
    })
    with urllib.request.urlopen(req, timeout=180) as r:
        d = json.loads(r.read())
    return d['choices'][0]['message']['content']

# Council members for Iter 2 — focus on models that worked in Iter 1
COUNCIL = [
    ('claude_opus47',     lambda: call_anthropic('claude-opus-4-5')),
    ('gpt5',              lambda: call_openai('gpt-5')),
    ('gpt5_mini',         lambda: call_openai('gpt-5-mini')),
    ('gemini_pro',        lambda: call_gemini('gemini-2.0-flash-thinking-exp-01-21')),
    ('deepseek_r1',       lambda: call_together('deepseek-ai/DeepSeek-R1')),
    ('llama33',           lambda: call_together('meta-llama/Llama-3.3-70B-Instruct-Turbo')),
    ('qwen_25_72b',       lambda: call_together('Qwen/Qwen2.5-72B-Instruct-Turbo')),
]

def run_one(name, fn):
    """Wrap with timing + error handling."""
    t0 = time.time()
    try:
        text = fn()
        dt = time.time() - t0
        return name, True, dt, text
    except Exception as e:
        dt = time.time() - t0
        return name, False, dt, f"ERROR: {type(e).__name__}: {e}"

# Execute in parallel
out_dir = Path('docs/40_ai_consultations/master_council/iter_2')
out_dir.mkdir(parents=True, exist_ok=True)

print(f"[ITER 2] Starting council with {len(COUNCIL)} models...")
print(f"[ITER 2] Spec size: {len(spec):,} chars")
results = {}

with ThreadPoolExecutor(max_workers=7) as ex:
    futures = {ex.submit(run_one, n, f): n for n, f in COUNCIL}
    for future in as_completed(futures, timeout=200):
        name, ok, dt, text = future.result()
        results[name] = {'ok': ok, 'time': dt, 'text': text}
        marker = '✓' if ok else '✗'
        print(f"  [{marker}] {name:<20} {dt:6.1f}s  {len(text):>6} chars")
        # Save
        (out_dir / f'{name}.md').write_text(text)

# Save metadata
ok_count = sum(1 for r in results.values() if r['ok'])
total_chars = sum(len(r['text']) for r in results.values() if r['ok'])

(out_dir / 'METADATA.json').write_text(json.dumps({
    'iter': 2,
    'date': '2026-04-25',
    'spec_chars': len(spec),
    'models': list(results.keys()),
    'ok_count': ok_count,
    'total_chars': total_chars,
    'focus': '§28-§43 — sections not validated in Iter 1',
    'results': {k: {'ok': v['ok'], 'time': v['time']} for k, v in results.items()}
}, indent=2))

print(f"\n[ITER 2] Done. Success: {ok_count}/{len(COUNCIL)}. Total response: {total_chars:,} chars.")
print(f"[ITER 2] Saved to {out_dir}/")
