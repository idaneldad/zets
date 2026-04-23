#!/usr/bin/env python3
"""
ask_ai.py — consult external AIs about ZETS with mandatory context primer.

Usage:
    python3 mcp/ask_ai.py "Your question here"
    python3 mcp/ask_ai.py --file my_question.txt
    python3 mcp/ask_ai.py --model gemini "Your question"
    python3 mcp/ask_ai.py --model groq "Your question"
    python3 mcp/ask_ai.py "Your question"     # (default: ask BOTH)

Why this script exists:
    Gemini/Groq/other external AIs don't know ZETS internals. Asking them
    cold gets you generic recommendations that hallucinate solutions to
    problems already solved, or conflict with existing code.

    This script:
    1. Prepends `docs/AI_CONSULTATION_PRIMER.md` to every prompt.
    2. Prepends a required-format instruction.
    3. Logs the full prompt + response to `mcp/logs/ai_consults/` for audit.
    4. Flags responses that mention forbidden patterns (zstd-seekable,
       Louvain, Neo4j, etc. — things that contradict the primer).

Do not bypass. If the AI gives bad advice, update the primer first, then
re-ask — don't just ignore the primer.
"""

import argparse
import datetime as dt
import json
import os
import re
import sys
import time
import urllib.request
import urllib.error
from pathlib import Path

# ─── Keys ───────────────────────────────────────────────────────────────
# Keys come from environment only — never hardcoded.
# Export them before running:
#   export GEMINI_API_KEY='AIza...'
#   export GROQ_API_KEY='gsk_...'
GEMINI_KEY = os.environ.get('GEMINI_API_KEY', '').strip() or os.environ.get('GEMINI_KEY', '').strip()
GROQ_KEY = os.environ.get('GROQ_API_KEY', '').strip() or os.environ.get('GROQ_KEY', '').strip()
OPENAI_KEY = os.environ.get('OPENAI_API_KEY', '').strip() or os.environ.get('OPENAI_KEY', '').strip()

def _check_keys():
    missing = []
    if not GEMINI_KEY or GEMINI_KEY.startswith('YOUR'):
        missing.append('GEMINI_API_KEY')
    if not GROQ_KEY or GROQ_KEY.startswith('YOUR'):
        missing.append('GROQ_API_KEY')
    return missing

# ─── Paths ──────────────────────────────────────────────────────────────
ZETS_ROOT = Path(os.environ.get('ZETS_ROOT', '/home/dinio/zets'))
PRIMER_PATH = ZETS_ROOT / 'docs' / 'AI_CONSULTATION_PRIMER.md'
LOG_DIR = ZETS_ROOT / 'mcp' / 'logs' / 'ai_consults'

# ─── Red-flag patterns ──────────────────────────────────────────────────
# If an AI's response contains any of these, it likely ignored the primer.
RED_FLAGS = [
    (r'\bzstd[-_ ]?seekable\b', 'zstd-seekable is premature — mmap already lazy'),
    (r'\bLouvain\b', 'Louvain not needed — per-lang split exists'),
    (r'\bNeo4j\b', "don't recommend full graph DBs — ZETS is a custom binary format"),
    (r'\bNeptune\b', "don't recommend full graph DBs"),
    (r'\bgradient descent\b', 'ZETS has no neural nets — no gradient descent'),
    (r'\bembedding[s]? model\b', 'ZETS uses symbolic atoms, not embeddings'),
    (r'\btrain[a-z]* a model\b', 'ZETS is not trained — it ingests text deterministically'),
    (r'\bRAG\b', 'ZETS is the knowledge layer, not RAG-over-LLM'),
    (r'\bvector[\s-]?database\b', 'no vectors — symbolic atoms only'),
    (r'\btranscformer[s]? architecture\b', 'no transformers'),
    (r'\bchain[\s-]?of[\s-]?thought\b', 'no CoT — deterministic walks only'),
]


def load_primer() -> str:
    if not PRIMER_PATH.exists():
        print(f"ERROR: primer not found at {PRIMER_PATH}", file=sys.stderr)
        sys.exit(1)
    return PRIMER_PATH.read_text(encoding='utf-8')


FORMAT_INSTRUCTIONS = """
# How to answer

You MUST:
1. Read the primer above first. Your recommendations will be judged against
   sections 4 (two data systems), 5 (existing modules), 6 (invariants).
2. When suggesting a change, name specific Rust modules/types from section 5
   that would be modified. Do not invent new module names without justification.
3. If your answer conflicts with the primer, STATE THE CONFLICT explicitly.
   Do not hide it. Example: "This conflicts with section 8's no-compression
   stance; here's why I still think it's needed: ..."
4. If you don't have enough info to answer, ask for what's missing. Do not
   invent facts.
5. Keep recommendations concrete. No "consider looking at X" — say "use X".

Forbidden (these contradict the primer):
- zstd-seekable, Louvain clustering, Neo4j/Neptune, embeddings, transformers,
  gradient descent, chain-of-thought, vector databases, RAG, training a model.
  If any of these truly are needed, explain WHY the primer is wrong.

# Question
"""


def check_red_flags(response: str) -> list:
    flags = []
    for pattern, reason in RED_FLAGS:
        if re.search(pattern, response, flags=re.IGNORECASE):
            flags.append((pattern, reason))
    return flags


def ask_gemini(prompt: str, model: str = 'gemini-2.5-flash', timeout: int = 180) -> dict:
    url = f'https://generativelanguage.googleapis.com/v1beta/models/{model}:generateContent?key={GEMINI_KEY}'
    body = {
        "contents": [{"parts": [{"text": prompt}]}],
        "generationConfig": {"maxOutputTokens": 32000, "temperature": 0.2},
    }
    req = urllib.request.Request(url, data=json.dumps(body).encode('utf-8'), method='POST')
    req.add_header('Content-Type', 'application/json')
    t0 = time.time()
    try:
        with urllib.request.urlopen(req, timeout=timeout) as r:
            d = json.loads(r.read())
            return {
                'model': model,
                'text': d['candidates'][0]['content']['parts'][0]['text'],
                'elapsed': time.time() - t0,
                'ok': True,
            }
    except urllib.error.HTTPError as e:
        return {'model': model, 'text': f'[HTTP {e.code}: {e.read().decode()[:500]}]',
                'elapsed': time.time() - t0, 'ok': False}
    except Exception as e:
        return {'model': model, 'text': f'[err: {str(e)[:500]}]',
                'elapsed': time.time() - t0, 'ok': False}


def ask_groq(prompt: str, model: str = 'meta-llama/llama-4-scout-17b-16e-instruct',
             timeout: int = 180) -> dict:
    body = {
        "model": model,
        "messages": [{"role": "user", "content": prompt}],
        "max_tokens": 16000,
        "temperature": 0.2,
    }
    req = urllib.request.Request('https://api.groq.com/openai/v1/chat/completions',
                                 data=json.dumps(body).encode('utf-8'), method='POST')
    req.add_header('Content-Type', 'application/json')
    req.add_header('Authorization', f'Bearer {GROQ_KEY}')
    req.add_header('User-Agent', 'Mozilla/5.0 ZETS-arch-discussion')
    t0 = time.time()
    try:
        with urllib.request.urlopen(req, timeout=timeout) as r:
            d = json.loads(r.read())
            return {
                'model': model,
                'text': d['choices'][0]['message']['content'],
                'elapsed': time.time() - t0,
                'ok': True,
            }
    except urllib.error.HTTPError as e:
        return {'model': model, 'text': f'[HTTP {e.code}: {e.read().decode()[:500]}]',
                'elapsed': time.time() - t0, 'ok': False}
    except Exception as e:
        return {'model': model, 'text': f'[err: {str(e)[:500]}]',
                'elapsed': time.time() - t0, 'ok': False}



def ask_chatgpt(prompt: str, model: str = 'gpt-4o',
                timeout: int = 180) -> dict:
    """Consult OpenAI ChatGPT. Uses /v1/chat/completions API."""
    if not OPENAI_KEY or OPENAI_KEY.startswith('YOUR'):
        return {'model': model, 'text': '[err: no OPENAI_API_KEY]',
                'elapsed': 0, 'ok': False}
    body = {
        "model": model,
        "messages": [{"role": "user", "content": prompt}],
        "max_tokens": 16000,
        "temperature": 0.2,
    }
    req = urllib.request.Request('https://api.openai.com/v1/chat/completions',
                                 data=json.dumps(body).encode('utf-8'), method='POST')
    req.add_header('Content-Type', 'application/json')
    req.add_header('Authorization', f'Bearer {OPENAI_KEY}')
    req.add_header('User-Agent', 'ZETS-arch-discussion/1.0')
    t0 = time.time()
    try:
        with urllib.request.urlopen(req, timeout=timeout) as r:
            d = json.loads(r.read())
            return {
                'model': model,
                'text': d['choices'][0]['message']['content'],
                'elapsed': time.time() - t0,
                'ok': True,
            }
    except urllib.error.HTTPError as e:
        return {'model': model, 'text': f'[HTTP {e.code}: {e.read().decode()[:500]}]',
                'elapsed': time.time() - t0, 'ok': False}
    except Exception as e:
        return {'model': model, 'text': f'[err: {str(e)[:500]}]',
                'elapsed': time.time() - t0, 'ok': False}


def log_consultation(question: str, primer_hash: str, responses: list) -> Path:
    LOG_DIR.mkdir(parents=True, exist_ok=True)
    ts = dt.datetime.now().strftime('%Y%m%d_%H%M%S')
    log_file = LOG_DIR / f'{ts}.md'
    lines = [
        f'# AI Consultation — {ts}',
        '',
        f'Primer hash: `{primer_hash}`',
        '',
        '## Question',
        '',
        question,
        '',
        '## Responses',
        '',
    ]
    for resp in responses:
        flags = check_red_flags(resp['text']) if resp['ok'] else []
        lines.append(f"### {resp['model']}  ({resp['elapsed']:.1f}s)")
        lines.append('')
        if flags:
            lines.append(f'**⚠ {len(flags)} red-flag pattern(s) detected:**')
            for pat, reason in flags:
                lines.append(f'  - `{pat}` — {reason}')
            lines.append('')
        lines.append(resp['text'])
        lines.append('')
        lines.append('---')
        lines.append('')
    log_file.write_text('\n'.join(lines), encoding='utf-8')
    return log_file


def main():
    ap = argparse.ArgumentParser(description='Consult external AIs about ZETS.')
    ap.add_argument('question', nargs='?', help='Question text (or use --file)')
    ap.add_argument('--file', help='Read question from this file')
    ap.add_argument('--model', choices=['gemini', 'groq', 'both'], default='both')
    ap.add_argument('--no-log', action='store_true', help='Skip logging to disk')
    args = ap.parse_args()

    if args.file:
        question = Path(args.file).read_text(encoding='utf-8')
    elif args.question:
        question = args.question
    else:
        ap.error('Provide a question or --file')

    primer = load_primer()
    # content hash for audit trail
    import hashlib
    primer_hash = hashlib.sha256(primer.encode('utf-8')).hexdigest()[:12]

    full_prompt = primer + '\n\n' + FORMAT_INSTRUCTIONS + '\n' + question

    print(f'━━━ ask_ai.py ━━━')
    print(f'  primer: {len(primer):,} chars (hash {primer_hash})')
    print(f'  question: {len(question):,} chars')
    print(f'  total prompt: {len(full_prompt):,} chars')
    print(f'  model(s): {args.model}')
    print()

    responses = []
    if args.model in ('gemini', 'both'):
        print('  asking Gemini...', flush=True)
        r = ask_gemini(full_prompt)
        flags = check_red_flags(r['text']) if r['ok'] else []
        print(f'    got {len(r["text"]):,} chars in {r["elapsed"]:.1f}s'
              f'{f"  [{len(flags)} RED FLAG(s)]" if flags else ""}')
        responses.append(r)

    if args.model in ('groq', 'both'):
        print('  asking Groq...', flush=True)
        r = ask_groq(full_prompt)
        flags = check_red_flags(r['text']) if r['ok'] else []
        print(f'    got {len(r["text"]):,} chars in {r["elapsed"]:.1f}s'
              f'{f"  [{len(flags)} RED FLAG(s)]" if flags else ""}')
        responses.append(r)

    if not args.no_log:
        log_file = log_consultation(question, primer_hash, responses)
        print()
        print(f'  logged to: {log_file}')

    print()
    print('━━━ RESPONSES ━━━')
    print()
    for r in responses:
        print(f'╔══ {r["model"]} ({r["elapsed"]:.1f}s) ═══')
        flags = check_red_flags(r['text']) if r['ok'] else []
        if flags:
            print(f'║ ⚠ {len(flags)} red-flag pattern(s):')
            for pat, reason in flags:
                print(f'║   - {pat}  →  {reason}')
        print(r['text'])
        print()


if __name__ == '__main__':
    main()
