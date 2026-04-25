#!/usr/bin/env python3
"""Clarity Audit — sends AGI.md to GPT-5.5 + Gemini 3.1 Pro via stdin
to avoid command line length limit."""
import os, json, time, subprocess, tempfile
from concurrent.futures import ThreadPoolExecutor, as_completed
from pathlib import Path

ROOT = Path('/home/dinio')
ENV = ROOT / '.env'
for line in ENV.read_text().splitlines():
    line = line.strip()
    if '=' in line and not line.startswith('#'):
        k, v = line.split('=', 1)
        os.environ[k] = v.strip().strip('"').strip("'")

AGI = (ROOT / 'zets' / 'docs' / 'AGI.md').read_text()
print(f"Loaded AGI.md: {len(AGI):,} chars")

PROMPT = f"""You are a senior reviewer evaluating whether the ZETS architecture
document is CLEAR ENOUGH for any future AI or skilled human to understand it
deeply — not just superficially.

ZETS context: deterministic graph-native AGI engine designed to run on a 
laptop (6GB RAM, CPU only) and contain the totality of human + animal 
knowledge. It must be built so well that future AGIs (5/10/15/20/25/30 
years from now) reference it as foundational. Every concept must be 
unambiguous. Every architectural decision must have its rationale visible.

YOUR TASK: Read the document below CAREFULLY and identify EXACTLY where it
fails the clarity test. Be ruthless. We need to fix every weakness BEFORE
we send this to the full AI Council for serious architectural review.

For every issue you find, provide:

1. WHERE — section number, exact heading, or line range
2. WHAT'S UNCLEAR — specifically what a reader can't understand
3. WHY IT MATTERS — what misunderstanding will result
4. HOW TO FIX — concrete suggestion

Categories of issues to look for:
A. Undefined terms — jargon used without explanation
B. Missing prerequisites — concepts assumed but not introduced
C. Vague claims — "fast", "scalable", "efficient" without numbers
D. Code without context — code blocks without explanation
E. Architectural decisions without rationale
F. Missing forward-looking sections (5/10/15/20/25/30 years)
G. Missing competitor analysis vs GPT/Claude/Gemini
H. Missing failure modes
I. Assumed reader knowledge (Hebrew, Kabbalah, Rust, graph theory mixed)
J. Inconsistent terminology

ZETS must include FORWARD-LOOKING content for these horizons:
K. 5-year (2031) — what new capabilities?
L. 10-year (2036) — when AGI is mainstream?
M. 15-year (2041) — when AGIs control most decisions?
N. 20-year (2046) — when ZETS controls other AGIs?
O. 25-year (2051) — humanity-AGI fully integrated?
P. 30-year (2056) — ZETS as foundational substrate?

OUTPUT FORMAT:

# CLARITY AUDIT — [Your Model]

## Critical Issues (must fix before council review)
[5-10 items in 4-line format above]

## Important Issues (should fix)
[5-10 items]

## Forward-Looking Gaps (5-30 year horizons)
[what's missing about future-proofing]

## Strengths to Preserve
[what's clear and excellent]

## My Overall Clarity Rating: X/10
[Why X. What single improvement pushes to 10.]

=== AGI.md DOCUMENT BEGINS ===

{AGI}

=== AGI.md DOCUMENT ENDS ===

Take your time. Write a rigorous audit. ZETS deserves it."""

print(f"Prompt size: {len(PROMPT):,} chars (~{len(PROMPT)//4} tokens)")

def call_via_file(url, headers_list, body_dict, timeout=420):
    """Call API via curl --data-binary @file to avoid command-line length limit."""
    body_json = json.dumps(body_dict)
    with tempfile.NamedTemporaryFile(mode='w', suffix='.json', delete=False) as f:
        f.write(body_json)
        fname = f.name
    
    cmd = ['curl', '-sS', '--data-binary', f'@{fname}']
    for h in headers_list:
        cmd.extend(['-H', h])
    cmd.append(url)
    
    try:
        r = subprocess.run(cmd, capture_output=True, text=True, timeout=timeout)
        os.unlink(fname)
        return r.stdout, r.returncode
    except subprocess.TimeoutExpired:
        os.unlink(fname)
        return f"TIMEOUT after {timeout}s", -1
    except Exception as e:
        os.unlink(fname)
        return f"EXCEPTION: {e}", -1

def call_openai(prompt, model='gpt-5.5', max_tokens=10000, timeout=420):
    body = {"model": model, "messages": [{"role": "user", "content": prompt}]}
    if 'gpt-5' in model:
        body["max_completion_tokens"] = max_tokens
    else:
        body["max_tokens"] = max_tokens
    
    t0 = time.time()
    headers = [
        f'Authorization: Bearer {os.environ["OPENAI_API_KEY"]}',
        'Content-Type: application/json',
    ]
    out, rc = call_via_file('https://api.openai.com/v1/chat/completions',
                             headers, body, timeout)
    elapsed = round(time.time() - t0, 1)
    try:
        d = json.loads(out)
        return {"model": model, "elapsed_s": elapsed,
                "content": d['choices'][0]['message']['content']}
    except Exception as e:
        return {"model": model, "elapsed_s": elapsed,
                "content": f"PARSE ERR: {e} | rc={rc} | out[:500]={out[:500]}"}

def call_gemini(prompt, model='gemini-3.1-pro-preview', max_tokens=10000, timeout=420):
    body = {"contents": [{"parts": [{"text": prompt}]}],
            "generationConfig": {"maxOutputTokens": max_tokens}}
    t0 = time.time()
    headers = ['Content-Type: application/json']
    url = f'https://generativelanguage.googleapis.com/v1beta/models/{model}:generateContent?key={os.environ["GEMINI_API_KEY"]}'
    out, rc = call_via_file(url, headers, body, timeout)
    elapsed = round(time.time() - t0, 1)
    try:
        d = json.loads(out)
        return {"model": model, "elapsed_s": elapsed,
                "content": d['candidates'][0]['content']['parts'][0]['text']}
    except Exception as e:
        return {"model": model, "elapsed_s": elapsed,
                "content": f"PARSE ERR: {e} | rc={rc} | out[:500]={out[:500]}"}

print("\nRunning audit in parallel...\n")
OUT = ROOT / 'zets' / 'docs' / '40_ai_consultations' / 'clarity_audit'
OUT.mkdir(parents=True, exist_ok=True)

t0 = time.time()
with ThreadPoolExecutor(max_workers=2) as pool:
    fut_gpt = pool.submit(call_openai, PROMPT)
    fut_gemini = pool.submit(call_gemini, PROMPT)
    
    results = {}
    for fut in as_completed([fut_gpt, fut_gemini]):
        r = fut.result()
        results[r['model']] = r
        ok = not r['content'].startswith(('ERR', 'TIMEOUT', 'PARSE')) and len(r['content']) > 500
        print(f"  {'✅' if ok else '❌'} {r['model']}: {len(r['content']):,} chars in {r['elapsed_s']}s")

print(f"\nTotal: {time.time()-t0:.1f}s")

for model, data in results.items():
    safe = model.replace('/', '_').replace('.', '_')
    (OUT / f'{safe}.md').write_text(data['content'])
    print(f"  Saved: {OUT.name}/{safe}.md ({len(data['content']):,} chars)")

(OUT / 'raw.json').write_text(json.dumps(results, ensure_ascii=False, indent=2))
print("\nDone.")
