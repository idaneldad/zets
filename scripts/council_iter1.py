#!/usr/bin/env python3
"""Iter 1 — Broad Survey: all 14 council models review AGI.md v2.0."""
import os, json, time, subprocess, tempfile
from concurrent.futures import ThreadPoolExecutor, as_completed
from pathlib import Path

ROOT = Path('/home/dinio')
for line in (ROOT / '.env').read_text().splitlines():
    line = line.strip()
    if '=' in line and not line.startswith('#'):
        k, v = line.split('=', 1)
        os.environ[k] = v.strip().strip('"').strip("'")

AGI_PATH = ROOT / 'zets' / 'docs' / 'AGI.md'
AGI = AGI_PATH.read_text()
print(f"AGI.md: {len(AGI):,} chars / {AGI.count(chr(10))} lines")

PROMPT = f"""You are one of 14 expert architects reviewing the ZETS AGI specification.

ZETS = deterministic graph-native AGI engine. 6GB RAM, CPU-only laptop.
Hebrew-first canonical. Walks for reasoning. Zero hallucination.

Your role: act as a LOVING PARENT of ZETS. Provide concrete, specific,
implementable feedback that pushes the spec toward 10/10.

ITERATION 1 FOCUS: BROAD HOLISTIC SURVEY
- What's right (briefly)
- What's wrong (specifically)
- What's missing (concretely)
- What conflicts internally (with line refs if possible)

REQUIRED OUTPUT (Mutation Protocol + Issue Ledger):

## Top 5 Critical Issues
For each:
- ISS-NN: [issue title]
- Section affected: §X.Y
- Severity: critical | important | nice-to-have
- Confidence: 0-100
- Claim: [what's wrong/missing in <30 words]
- Proposed patch: [concrete diff, replacement text, or addition]
- Hidden assumption: [what you're assuming]
- Strongest self-objection: [why your proposed fix might be wrong]
- Validation test: [how we'd know it works]

## Top 3 Strengths (briefly)

## Open Question for Iter 2-7
[The single most important thing future iterations should focus on]

## Final Score
[X/10 with one-sentence rationale]

CONSTRAINTS:
- Must work on 6GB RAM CPU-only
- Must respect Hebrew-canonical principles  
- Must support 30-year evolution
- Concrete numbers > vague claims

RESPOND IN ENGLISH. Be specific. Be brief. <800 words total.

═══════════════════════════════════════════════════════════════════
THE ZETS AGI SPECIFICATION (read carefully):
═══════════════════════════════════════════════════════════════════

{AGI}
"""

print(f"Prompt size: {len(PROMPT):,} chars")

def call_via_file(url, headers, body, timeout=240):
    with tempfile.NamedTemporaryFile(mode='w', suffix='.json', delete=False) as f:
        f.write(json.dumps(body))
        fname = f.name
    cmd = ['curl', '-sS', '--data-binary', f'@{fname}']
    for h in headers:
        cmd.extend(['-H', h])
    cmd.append(url)
    try:
        r = subprocess.run(cmd, capture_output=True, text=True, timeout=timeout)
        os.unlink(fname)
        return r.stdout
    except Exception as e:
        try: os.unlink(fname)
        except: pass
        return f"ERR: {e}"

def call_anthropic(model="claude-opus-4-5"):
    body = {"model": model, "max_tokens": 3500,
            "messages": [{"role": "user", "content": PROMPT}]}
    headers = [f'x-api-key: {os.environ["ANTHROPIC_API_KEY"]}',
               'anthropic-version: 2023-06-01', 'content-type: application/json']
    out = call_via_file('https://api.anthropic.com/v1/messages', headers, body, 240)
    try:
        return json.loads(out)['content'][0]['text']
    except Exception as e:
        return f"ERR: {e} | {out[:200]}"

def call_openai(model='gpt-5.5'):
    body = {"model": model, "messages": [{"role": "user", "content": PROMPT}]}
    if 'gpt-5' in model:
        body["max_completion_tokens"] = 3500
    else:
        body["max_tokens"] = 3500
    headers = [f'Authorization: Bearer {os.environ["OPENAI_API_KEY"]}',
               'Content-Type: application/json']
    out = call_via_file('https://api.openai.com/v1/chat/completions', headers, body, 240)
    try:
        return json.loads(out)['choices'][0]['message']['content']
    except Exception as e:
        return f"ERR: {e} | {out[:200]}"

def call_gemini(model='gemini-3.1-pro-preview'):
    body = {"contents": [{"parts": [{"text": PROMPT}]}],
            "generationConfig": {"maxOutputTokens": 3500}}
    headers = ['Content-Type: application/json']
    url = f'https://generativelanguage.googleapis.com/v1beta/models/{model}:generateContent?key={os.environ["GEMINI_API_KEY"]}'
    out = call_via_file(url, headers, body, 240)
    try:
        d = json.loads(out)
        return d['candidates'][0]['content']['parts'][0]['text']
    except Exception as e:
        return f"ERR: {e} | {out[:200]}"

def call_together(model, max_tokens=3500, timeout=300):
    body = {"model": model, "messages": [{"role": "user", "content": PROMPT}],
            "max_tokens": max_tokens}
    headers = [f'Authorization: Bearer {os.environ["TOGETHER_API_KEY"]}',
               'Content-Type: application/json',
               'User-Agent: Mozilla/5.0']
    out = call_via_file('https://api.together.xyz/v1/chat/completions', headers, body, timeout)
    try:
        return json.loads(out)['choices'][0]['message']['content']
    except Exception as e:
        return f"ERR: {e} | {out[:200]}"

# 14 council members
COUNCIL = [
    ('claude_opus47',  lambda: call_anthropic('claude-opus-4-5')),
    ('gpt55',          lambda: call_openai('gpt-5.5')),
    ('gemini31pro',    lambda: call_gemini('gemini-3.1-pro-preview')),
    ('deepseek_r1',    lambda: call_together('deepseek-ai/DeepSeek-R1-0528', 4000)),
    ('deepseek_v4pro', lambda: call_together('deepseek-ai/DeepSeek-V4-Pro', 3500)),
    ('deepseek_v31',   lambda: call_together('deepseek-ai/DeepSeek-V3.1', 3500)),
    ('kimi_k26',       lambda: call_together('moonshotai/Kimi-K2.6-Instruct', 3500)),
    ('kimi_k25',       lambda: call_together('moonshotai/Kimi-K2.5-Instruct', 3500)),
    ('qwen35',         lambda: call_together('Qwen/Qwen3.5-397B-Instruct', 3500)),
    ('qwen3_coder',    lambda: call_together('Qwen/Qwen3-Coder-480B-A35B-Instruct', 3500)),
    ('glm51',          lambda: call_together('zai-org/GLM-5.1-Air', 3500)),
    ('minimax_m27',    lambda: call_together('MiniMaxAI/MiniMax-M2.7', 3500)),
    ('cogito_671b',    lambda: call_together('deepcogito/cogito-v2-1-671b', 3500, 360)),
    ('llama33_70b',    lambda: call_together('meta-llama/Llama-3.3-70B-Instruct-Turbo', 3500)),
]

OUT_DIR = ROOT / 'zets' / 'docs' / '40_ai_consultations' / 'master_council' / 'iter_1'
OUT_DIR.mkdir(parents=True, exist_ok=True)

print(f"\nCalling {len(COUNCIL)} council members in parallel...")
print(f"Output: {OUT_DIR}/")

t0 = time.time()
results = {}
with ThreadPoolExecutor(max_workers=14) as pool:
    futures = {pool.submit(fn): name for name, fn in COUNCIL}
    for f in as_completed(futures):
        name = futures[f]
        try:
            resp = f.result()
        except Exception as e:
            resp = f"FATAL: {e}"
        results[name] = resp
        ok = not resp.startswith('ERR') and not resp.startswith('FATAL') and len(resp) > 500
        print(f"  {'OK' if ok else 'FAIL'} {name}: {len(resp):,} chars [{time.time()-t0:.0f}s]")
        # save immediately
        (OUT_DIR / f'{name}.md').write_text(resp)

print(f"\nTotal time: {time.time()-t0:.0f}s")
(OUT_DIR / 'all_responses.json').write_text(json.dumps(results, ensure_ascii=False, indent=2))
print(f"Saved to {OUT_DIR}/")

# Quick stats
ok_count = sum(1 for r in results.values() if not r.startswith(('ERR', 'FATAL')) and len(r) > 500)
print(f"\nSuccess: {ok_count}/{len(COUNCIL)}")
