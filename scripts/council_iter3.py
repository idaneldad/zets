"""ITER 3 — Tree-walk encoding for alphabetic languages. Chinese case already locked."""
import os, json, time
from pathlib import Path
from concurrent.futures import ThreadPoolExecutor, as_completed
import urllib.request

from dotenv import load_dotenv
load_dotenv('/home/dinio/.env')

ANTHROPIC = os.getenv('ANTHROPIC_API_KEY')
TOGETHER  = os.getenv('TOGETHER_API_KEY') or 'tgp_v1_OomnteXcZqUhvRZO_hO03Uf_jJDXYG7rVHhPGIJ4AtI'

PROMPT = """ZETS ASI Council, Iteration 3. ENGINEERING-HONEST. NO FLATTERY.

═══════════════════════════════════════════════════════════════
LOCKED (do not re-discuss):
═══════════════════════════════════════════════════════════════
• Atom = 8 bytes fixed + VSA side-table (1024B/atom indexed by atom_id)
• EdgeKind = u8 (22 SY-locked + 0x80-FF reserved)
• Determinism = Q16.16
• Storage = Tri-Memory (RAM working + LSM episodic + CSR semantic + crystalline)
• Hebrew/Arabic = sense-anchored (shared semantic, distinct lexical)
• LLM_BOUNDARY: 2 SLMs + Rust deterministic Critic
• VSA-Tzeruf bridge: SY's 5 ops = VSA mathematically (carve/hew/combine/weigh/permute)
• Logographic chars (Chinese/Japanese-kanji/Egyptian) = atom-as-glyph (1 atom per character)

═══════════════════════════════════════════════════════════════
PROPOSAL TO VALIDATE — Alphabetic Tree-Walk Encoding
═══════════════════════════════════════════════════════════════

PRINCIPLE (Idan's directive):
• Disk-first: most data on mmap, RAM minimal
• Variable-size: small content → small storage
• Static structures preloaded in binary (zero runtime alloc)
• Words = walks on letter-trees, NOT strings of letters
• Source-grounding: Sefer Yetzirah 2:5 "אבנים בונות בתים"

PROPOSAL:
For alphabetic languages (Hebrew 22, Arabic 28, Latin 26, Cyrillic 33, Greek 24):

1. STATIC LETTER TREES compiled into binary (~1KB total):
   pub static HEBREW_LETTERS: [Letter; 22] — fits L1 cache
   pub static ARABIC_LETTERS: [Letter; 28]
   pub static LATIN_LETTERS:  [Letter; 26]
   etc.

2. WORDS encoded as variable-length tree-walks on disk:
   On-disk word = [length: 2-3 bits][path: variable bits]
   
3. EXAMPLE — Hebrew "שלום" (4 letters):
   Naive 5-bit IDs:  4 × 5 = 20 bits + 4-bit length = 24 bits
   Tree-walk:        5 + 3 + 2 + 2 + 2-bit prefix = 14 bits (42% saving)

4. Atom layout (8 bytes) STAYS FIXED:
   - kind=Lexical, lang_id field, payload = 50-bit pointer to disk record
   - Disk record = variable-bit tree-walk path

═══════════════════════════════════════════════════════════════
QUESTIONS — answer each concretely (no "depends"):
═══════════════════════════════════════════════════════════════

Q1: Is tree-walk decode actually faster than naive lookup,
    or does branching cost (variable bits per step) negate cache gains?

Q2: Foreign-script names embedded in Hebrew text (e.g., "Idan"
    written in Latin in a Hebrew document) — how to handle?
    Mode-switch atom? Separate lang_id per atom? Inline tag?

Q3: Niqqud (Hebrew vowels) and Arabic diacritics:
    - Part of the path (extending tree depth)?
    - Separate metadata field?
    - Discarded entirely?

Q4: RTL vs LTR walks — does tree-walk encoding depend on direction?
    Are Hebrew/Arabic walks reversed compared to Latin?

Q5: Bidirectional walks (forward + reverse from end of word) —
    does this match Or Yashar / Or Chozer principle?

Q6: Adding a new alphabet (e.g., Devanagari, Korean Hangul) —
    requires recompile? Hot-reloadable? Or keep alphabetic table
    as data-section of binary that's mmap'd separately?

Q7: Korean Hangul is COMPOSED of jamo (consonants/vowels) into
    single visual blocks. Case 1 (atom-as-glyph) or Case 2 (tree-walk)?

Q8: Variable-length disk records — fragmentation risk?
    How to bulk-load 100K words at startup without thrashing?

Q9: Cache locality: 1KB static letter tables fit L1.
    But VSA side-table (1024B/atom) does NOT fit cache.
    Does tree-walk decode need the VSA vector, or just the letter IDs?

Q10: Falsification benchmark — what ONE test in 1 day would
     prove or refute "tree-walk encoding is 30%+ smaller than naive"?

═══════════════════════════════════════════════════════════════
REQUIRED OUTPUT FORMAT:
═══════════════════════════════════════════════════════════════
## Verdict (1-10)
## Top 3 Strengths
## Top 3 Concrete Risks
## Q1-Q10 Answers (concrete, brief)
## Recommended Refinements
## Falsification test
## Self-rating (1-10)
"""

def call_anthropic(model='claude-opus-4-5'):
    body = json.dumps({"model": model, "max_tokens": 4000,
                       "messages": [{"role": "user", "content": PROMPT}]}).encode()
    req = urllib.request.Request("https://api.anthropic.com/v1/messages", data=body, headers={
        "x-api-key": ANTHROPIC, "anthropic-version": "2023-06-01",
        "Content-Type": "application/json"})
    with urllib.request.urlopen(req, timeout=180) as r:
        return json.loads(r.read())['content'][0]['text']

def call_together(model):
    body = json.dumps({"model": model, "messages": [{"role": "user", "content": PROMPT}],
                       "max_tokens": 4000, "temperature": 0.3}).encode()
    req = urllib.request.Request("https://api.together.xyz/v1/chat/completions", data=body, headers={
        "Authorization": f"Bearer {TOGETHER}", "Content-Type": "application/json",
        "User-Agent": "Mozilla/5.0"})
    with urllib.request.urlopen(req, timeout=180) as r:
        return json.loads(r.read())['choices'][0]['message']['content']

COUNCIL = [
    ('claude_opus47',  lambda: call_anthropic('claude-opus-4-5')),
    ('deepseek_r1',    lambda: call_together('deepseek-ai/DeepSeek-R1')),
    ('llama33',        lambda: call_together('meta-llama/Llama-3.3-70B-Instruct-Turbo')),
    ('qwen_25',        lambda: call_together('Qwen/Qwen2.5-72B-Instruct-Turbo')),
]

def run_one(name, fn):
    t0 = time.time()
    try: return name, True, time.time()-t0, fn()
    except Exception as e: return name, False, time.time()-t0, f"ERROR: {type(e).__name__}: {e}"

out_dir = Path('docs/40_ai_consultations/master_council/iter_3')
out_dir.mkdir(parents=True, exist_ok=True)

print(f"[ITER 3] {len(COUNCIL)} models...")
results = {}
with ThreadPoolExecutor(max_workers=4) as ex:
    futures = {ex.submit(run_one, n, f): n for n, f in COUNCIL}
    for future in as_completed(futures, timeout=200):
        name, ok, dt, text = future.result()
        results[name] = {'ok': ok, 'time': dt, 'text': text}
        marker = '✓' if ok else '✗'
        print(f"  [{marker}] {name:<18} {dt:6.1f}s  {len(text):>6} chars")
        (out_dir / f'{name}.md').write_text(text)

ok_count = sum(1 for r in results.values() if r['ok'])
total_chars = sum(len(r['text']) for r in results.values() if r['ok'])
(out_dir / 'METADATA.json').write_text(json.dumps({
    'iter': 3, 'date': '2026-04-25', 'focus': 'Tree-walk encoding for alphabetic languages',
    'ok': ok_count, 'total_chars': total_chars,
    'results': {k: {'ok': v['ok'], 'time': v['time']} for k, v in results.items()}
}, indent=2))

print(f"\n[ITER 3] {ok_count}/{len(COUNCIL)} ok. {total_chars:,} chars total.")
