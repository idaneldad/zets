#!/usr/bin/env python3
"""Consult 5 top architects on the 7-iteration methodology design."""
import os, json, time, subprocess, tempfile
from concurrent.futures import ThreadPoolExecutor, as_completed
from pathlib import Path

ROOT = Path('/home/dinio')
for line in (ROOT / '.env').read_text().splitlines():
    line = line.strip()
    if '=' in line and not line.startswith('#'):
        k, v = line.split('=', 1)
        os.environ[k] = v.strip().strip('"').strip("'")

PROMPT = """You are one of 5 top architects consulting on a meta-methodology design.

THE PROJECT: ZETS — deterministic graph-native AGI engine. 30-year foundational
spec. 142KB master document (AGI.md, 4530 lines) covering atoms, walks,
sefirot, kabbalah, Hebrew morphology, learning loops, federation.

THE TASK: We will run 7 iterations of consultation with 14 different AI models
(Claude Opus 4.7, GPT-5.5, Gemini 3.1 Pro, DeepSeek R1, Kimi K2.6, Qwen 3.5,
GLM 5.1, MiniMax M2.7, Cogito 671B, etc.) to refine the master document. Each
model will be asked to act as the "loving parent" of ZETS — give their best
specific advice.

Sessions are STATELESS — each call to each model is fresh, no memory of
previous iterations. So we must inject context smartly.

THE QUESTION: What's the best methodology for iteration N+1 to leverage
iteration N's wisdom WITHOUT either:
(a) flooding with raw responses (140KB per iteration of accumulated noise)
(b) over-synthesizing and losing diversity / disagreement signal

CURRENT PROPOSAL (please critique):

**Approach: Structured Synthesis with Angle-Variation per Iteration**

Each iteration N produces (after the 14 models respond):
1. CONSENSUS BLOCK (1K tokens): what all 14 agreed on
2. DISAGREEMENTS BLOCK (1K tokens): top 3-5 places models split, with quotes
3. OPEN QUESTIONS BLOCK (500 tokens): what wasn't resolved
4. POINTER TO RAW (just URL/path): for those wanting depth

Iteration N+1 prompt includes:
- Original AGI.md (35K tokens)
- Structured synthesis from iter N (2.5K tokens)
- THIS iteration's specific angle/focus

Iteration angles (different focus each round):
- Iter 1: Holistic survey
- Iter 2: Top 3 weakest gaps deep dive
- Iter 3: Forward-looking (5-30 years)
- Iter 4: Contradictions/consistency check
- Iter 5: Implementation feasibility
- Iter 6: Competitor differentiation (why ZETS > GPT/Claude/Gemini long-term)
- Iter 7: Final integration push to 10/10

After iter 7, I (Claude Opus 4.7) do final genius synthesis using ALL
accumulated context.

QUESTIONS FOR YOU:

1. Is this methodology sound? What's the BIGGEST FLAW?

2. The "structured synthesis" approach: should we preserve raw responses
   in the prompt, or trust the synthesis? Why?

3. Should iterations focus on DIFFERENT angles (as proposed) or SAME angle
   with progressive depth? Why?

4. Should we pair-up disagreeing models in iter 2 (adversarial debate) or
   keep all 14 independent?

5. Cost-saving idea: skip cheap models in deep iterations, use only top-7?
   Or always all 14?

6. Bias concern: the synthesizer (me, Claude) is one of the 14 models.
   Is that a fatal conflict of interest? How to mitigate?

7. What ONE addition to this methodology would push it from 8/10 to 10/10?

Provide rigorous, specific, no-flattery feedback. <500 words.
Output structure:

## Biggest Flaw
[the single most important problem]

## Q1-Q7 Answers
[brief, direct]

## Methodology Improvement
[the +1 thing that pushes to 10/10]

## My Rating: X/10
[for the proposed methodology]
"""

def call_via_file(url, headers_list, body_dict, timeout=180):
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
        return r.stdout
    except Exception as e:
        try: os.unlink(fname)
        except: pass
        return f"ERR: {e}"

def call_anthropic(prompt, model="claude-opus-4-5", max_tokens=2500, timeout=180):
    body = {"model": model, "max_tokens": max_tokens,
            "messages": [{"role": "user", "content": prompt}]}
    headers = [f'x-api-key: {os.environ["ANTHROPIC_API_KEY"]}',
               'anthropic-version: 2023-06-01', 'content-type: application/json']
    out = call_via_file('https://api.anthropic.com/v1/messages', headers, body, timeout)
    try:
        d = json.loads(out)
        return d['content'][0]['text']
    except Exception as e:
        return f"ERR: {e} | {out[:300]}"

def call_openai(prompt, model='gpt-5.5', max_tokens=2500, timeout=180):
    body = {"model": model, "messages": [{"role": "user", "content": prompt}]}
    if 'gpt-5' in model:
        body["max_completion_tokens"] = max_tokens
    else:
        body["max_tokens"] = max_tokens
    headers = [f'Authorization: Bearer {os.environ["OPENAI_API_KEY"]}',
               'Content-Type: application/json']
    out = call_via_file('https://api.openai.com/v1/chat/completions', headers, body, timeout)
    try:
        d = json.loads(out)
        return d['choices'][0]['message']['content']
    except Exception as e:
        return f"ERR: {e} | {out[:300]}"

def call_gemini(prompt, model='gemini-3.1-pro-preview', max_tokens=2500, timeout=180):
    body = {"contents": [{"parts": [{"text": prompt}]}],
            "generationConfig": {"maxOutputTokens": max_tokens}}
    headers = ['Content-Type: application/json']
    url = f'https://generativelanguage.googleapis.com/v1beta/models/{model}:generateContent?key={os.environ["GEMINI_API_KEY"]}'
    out = call_via_file(url, headers, body, timeout)
    try:
        d = json.loads(out)
        return d['candidates'][0]['content']['parts'][0]['text']
    except Exception as e:
        return f"ERR: {e} | {out[:300]}"

def call_together(prompt, model, max_tokens=3000, timeout=240):
    body = {"model": model, "messages": [{"role": "user", "content": prompt}],
            "max_tokens": max_tokens}
    headers = [f'Authorization: Bearer {os.environ["TOGETHER_API_KEY"]}',
               'Content-Type: application/json',
               'User-Agent: Mozilla/5.0']
    out = call_via_file('https://api.together.xyz/v1/chat/completions', headers, body, timeout)
    try:
        d = json.loads(out)
        return d['choices'][0]['message']['content']
    except Exception as e:
        return f"ERR: {e} | {out[:300]}"

# 5 architects (Claude is 6th — me — synthesizing this run)
COUNCIL = [
    ('claude_opus47', lambda: call_anthropic(PROMPT)),
    ('gpt55',         lambda: call_openai(PROMPT, 'gpt-5.5')),
    ('gemini31pro',   lambda: call_gemini(PROMPT, 'gemini-3.1-pro-preview')),
    ('deepseek_r1',   lambda: call_together(PROMPT, 'deepseek-ai/DeepSeek-R1-0528', max_tokens=4000, timeout=300)),
    ('cogito_671b',   lambda: call_together(PROMPT, 'deepcogito/cogito-v2-1-671b', max_tokens=3000, timeout=300)),
]

print(f"Consulting {len(COUNCIL)} architects on methodology design...")
print(f"Prompt: {len(PROMPT)} chars\n")

t0 = time.time()
results = {}
with ThreadPoolExecutor(max_workers=5) as pool:
    futures = {pool.submit(fn): name for name, fn in COUNCIL}
    for f in as_completed(futures):
        name = futures[f]
        resp = f.result()
        results[name] = resp
        ok = not resp.startswith('ERR') and len(resp) > 300
        print(f"  {'✅' if ok else '❌'} {name}: {len(resp):,} chars")

print(f"\nTotal: {time.time()-t0:.1f}s")

OUT = ROOT / 'zets' / 'docs' / '40_ai_consultations' / 'methodology'
OUT.mkdir(parents=True, exist_ok=True)
for name, resp in results.items():
    (OUT / f'{name}.md').write_text(resp)
(OUT / 'raw.json').write_text(json.dumps(results, ensure_ascii=False, indent=2))
print(f"\nSaved to {OUT}/")
