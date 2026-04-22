#!/usr/bin/env python3
"""
build_persona_snapshots.py — convert Python persona definitions into:
  1. data/personas/<name>.persona.json     — config read by Rust server
  2. data/clients/<name>.atoms              — seeded AtomStore for Rust server

The atoms file contains:
  - An atom for the persona's own name (both HE and EN)
  - Atoms for each preferred topic
  - Edges: self likes topic, self speaks_language lang
  - A handful of bootstrap facts each persona 'knows'

The atoms file uses the existing atom_persist format — same as the wiki
benchmark snapshots. A Rust server can load it with atom_persist::load_from_file.

Since atom_persist is Rust-only, we shell out to a tiny Rust builder:
  cargo run --release --bin build_persona_atoms -- --persona personas/idan.persona.json --out data/clients/idan.atoms
"""

import json
import os
import subprocess
import sys
from pathlib import Path

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from personas import PERSONAS  # noqa

ZETS_ROOT = Path(os.environ.get('ZETS_ROOT', '/home/dinio/zets'))
PERSONAS_DIR = ZETS_ROOT / 'data' / 'personas'
CLIENTS_DIR = ZETS_ROOT / 'data' / 'clients'

PERSONAS_DIR.mkdir(parents=True, exist_ok=True)
CLIENTS_DIR.mkdir(parents=True, exist_ok=True)


def persona_to_json(p) -> dict:
    s = p.style
    return {
        'name': p.name,
        'name_he': p.name_he,
        'lang': p.lang,
        'age_hint': p.age_hint,
        'bio': p.bio,
        'cognitive_note': p.cognitive_note,
        'preferred_topics': s.preferred_topics,
        'avoided_topics': s.avoided_topics,
        'verbosity': s.verbosity,
        'formality': s.formality,
        'warmth': s.tone_warmth,
        'emoji_rate': s.emoji_rate,
        'ask_peer_threshold': s.asks_for_help_threshold,
        'strict_verifier': s.strict_verifier,
        'confidence_baseline': s.confidence_baseline,
        'attention_span': s.attention_span,
        'topic_switch_rate': s.topic_switch_rate,
        'proactive': s.proactive,
        'curiosity': s.curiosity,
    }


def persona_to_seed_facts(p) -> list:
    """Return list of (subject, relation, object) triples that seed the graph.

    Each persona gets:
    - self is_a human
    - self has_name <name>
    - self speaks_language <lang>
    - self likes <topic>  (for each preferred_topic)
    - human has_attribute <some basic traits>
    """
    facts = [
        (p.name.lower(), 'is_a', 'human'),
        (p.name.lower(), 'has_attribute', p.name),           # name as attribute
        (p.name.lower(), 'has_attribute', p.name_he),        # Hebrew name as attribute
        ('human', 'is_a', 'living_being'),
        ('living_being', 'has_attribute', 'alive'),
    ]
    # Language — existing relation speaks_language
    for lang in p.lang.split('+'):
        facts.append((p.name.lower(), 'speaks_language', lang))
    # Age hint — as attribute
    facts.append((p.name.lower(), 'has_attribute', p.age_hint))
    # Preferred topics — interest = has_attribute
    for topic in p.style.preferred_topics:
        facts.append((p.name.lower(), 'has_attribute', f'interest_{topic}'))
    # Bio hook — role via has_occupation
    for keyword in ['architect', 'entrepreneur', 'musician', 'teenager',
                   'child', 'senior', 'operations', 'creative', 'spiritual',
                   'parent', 'developer', 'student']:
        if keyword in p.bio.lower() or keyword in p.cognitive_note.lower():
            facts.append((p.name.lower(), 'has_occupation', keyword))
    return facts


def write_persona_configs():
    print("━━━ Persona configs (JSON) ━━━")
    for p in PERSONAS:
        cfg = persona_to_json(p)
        out_path = PERSONAS_DIR / f"{p.name.lower()}.persona.json"
        out_path.write_text(json.dumps(cfg, ensure_ascii=False, indent=2),
                            encoding='utf-8')
        print(f"  ✓ {out_path.name}")


def write_seed_jsonl():
    """One JSONL file per persona with their seed facts."""
    print("\n━━━ Seed fact files (JSONL) ━━━")
    for p in PERSONAS:
        facts = persona_to_seed_facts(p)
        out_path = PERSONAS_DIR / f"{p.name.lower()}.seed.jsonl"
        with out_path.open('w', encoding='utf-8') as f:
            for subj, rel, obj in facts:
                f.write(json.dumps({
                    'subject': subj, 'relation': rel, 'object': obj
                }, ensure_ascii=False) + '\n')
        print(f"  ✓ {out_path.name} ({len(facts)} facts)")


def build_atoms_via_rust():
    """Invoke the Rust builder binary to produce .atoms files."""
    print("\n━━━ Building .atoms files via Rust ━━━")
    # Ensure binary exists
    bin_path = ZETS_ROOT / 'target' / 'release' / 'build_persona_atoms'
    if not bin_path.exists():
        print(f"  building {bin_path.name}...")
        subprocess.run(
            ['cargo', 'build', '--release', '--bin', 'build_persona_atoms'],
            cwd=str(ZETS_ROOT), check=True,
        )
    for p in PERSONAS:
        seed_file = PERSONAS_DIR / f"{p.name.lower()}.seed.jsonl"
        atoms_out = CLIENTS_DIR / f"{p.name.lower()}.atoms"
        r = subprocess.run(
            [str(bin_path), '--seed', str(seed_file), '--out', str(atoms_out)],
            capture_output=True, text=True,
        )
        if r.returncode != 0:
            print(f"  ✗ {p.name}: {r.stderr.strip()}")
        else:
            size = atoms_out.stat().st_size
            print(f"  ✓ {p.name.lower()}.atoms  ({size} bytes)  {r.stdout.strip()}")


if __name__ == '__main__':
    write_persona_configs()
    write_seed_jsonl()
    build_atoms_via_rust()
    print("\n━━━ Done. ━━━")
    print(f"  Persona configs: {PERSONAS_DIR}")
    print(f"  Atoms:           {CLIENTS_DIR}")
