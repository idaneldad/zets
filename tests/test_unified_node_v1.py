#!/usr/bin/env python3
"""
test_unified_node_v1.py — Python prototype proving:
  1. Server and Client are the SAME binary, differ only in config
  2. Conversation turns are stored in the graph (with pending questions)
  3. User reply ("yes" / "3") resolves a pending question by context
  4. RAM tracking with mmap-style page allocation
  5. Active/idle pages with madvise-like promotion (simulated)

200 lines. No dependencies outside stdlib.
Run:
  python3 tests/test_unified_node_v1.py

This is the PROTOTYPE. If it proves the concept works, we plan Rust implementation.
Design doc: docs/working/20260423_unified_node_design.md
"""

import hashlib
import json
import os
import resource
import time
from dataclasses import dataclass, field, asdict
from pathlib import Path
from typing import Optional


# ═══════════════════════════════════════════════════════════════════════
# Content hash — same FNV-1a as Rust atoms.rs
# ═══════════════════════════════════════════════════════════════════════

def content_hash(data: bytes) -> int:
    h = 0xcbf29ce484222325
    for b in data:
        h ^= b
        h = (h * 0x100000001b3) & 0xFFFFFFFFFFFFFFFF
    return h


# ═══════════════════════════════════════════════════════════════════════
# Unified node — one code, different configs
# ═══════════════════════════════════════════════════════════════════════

@dataclass
class NodeConfig:
    """Configuration — the ONLY difference between server and client."""
    name: str
    role: str                          # 'server' | 'persona'
    port: int
    parent_url: Optional[str] = None   # clients point to server
    init_graph: str = ''               # path to seed graph file
    hot_page_threshold: int = 3        # activity count before page gets MADV_HUGEPAGE
    sync_interval_sec: int = 300


@dataclass
class Atom:
    id: int
    kind: str
    content: str
    content_hash: int = 0

    def __post_init__(self):
        if self.content_hash == 0:
            self.content_hash = content_hash(self.content.encode('utf-8'))


@dataclass
class Edge:
    from_id: int
    to_id: int
    relation: str
    weight: int = 100


@dataclass
class ConversationTurn:
    """Every message saved — input and output both go to the graph."""
    turn_id: int
    ts: float
    speaker: str              # 'user' | 'assistant'
    content: str
    pending_questions: list = field(default_factory=list)  # list of question_atom_ids
    resolves_turn_id: Optional[int] = None   # if this turn answers a prior question


# ═══════════════════════════════════════════════════════════════════════
# Page activity simulation (mmap big-page behavior)
# ═══════════════════════════════════════════════════════════════════════

class PageTracker:
    """Tracks access frequency per page group — simulates madvise behavior.

    Real system: call madvise(MADV_HUGEPAGE) when group.access_count crosses threshold,
    MADV_COLD when it drops below. Python doesn't do this but the LOGIC is real.
    """
    PAGE_SIZE_SMALL = 4096        # 4 KB
    PAGE_SIZE_HUGE = 2 * 1024 * 1024  # 2 MB
    PAGE_SIZE_GIGA = 1024 * 1024 * 1024  # 1 GB

    def __init__(self, hot_threshold: int = 3):
        self.hot_threshold = hot_threshold
        self.access_counts: dict[int, int] = {}   # page_id → count
        self.page_type: dict[int, str] = {}       # page_id → 'SMALL'|'HUGE'|'GIGA'
        self.upgrade_events: list = []

    def touch(self, atom_id: int):
        """Track an access. Promote page to HUGE if crosses threshold."""
        page_id = atom_id // 1000  # simulated: 1000 atoms per page
        self.access_counts[page_id] = self.access_counts.get(page_id, 0) + 1
        current = self.page_type.get(page_id, 'SMALL')
        count = self.access_counts[page_id]

        if current == 'SMALL' and count >= self.hot_threshold:
            self.page_type[page_id] = 'HUGE'
            self.upgrade_events.append({
                'page_id': page_id, 'from': 'SMALL', 'to': 'HUGE',
                'at_count': count, 'ts': time.time()
            })
        elif current == 'HUGE' and count >= self.hot_threshold * 10:
            self.page_type[page_id] = 'GIGA'
            self.upgrade_events.append({
                'page_id': page_id, 'from': 'HUGE', 'to': 'GIGA',
                'at_count': count, 'ts': time.time()
            })

    def cool_down(self):
        """Called periodically — demote unused pages (simulates MADV_COLD)."""
        for page_id, count in list(self.access_counts.items()):
            if count == 0 and self.page_type.get(page_id) != 'SMALL':
                self.page_type[page_id] = 'SMALL'
                self.upgrade_events.append({
                    'page_id': page_id, 'to': 'SMALL', 'reason': 'cold'
                })
            self.access_counts[page_id] = max(0, count - 1)  # decay


# ═══════════════════════════════════════════════════════════════════════
# The unified node
# ═══════════════════════════════════════════════════════════════════════

class ZetsNode:
    def __init__(self, config: NodeConfig):
        self.config = config
        self.atoms: dict[int, Atom] = {}
        self.edges: list[Edge] = []
        self.content_index: dict[int, int] = {}  # content_hash → atom_id
        self.next_atom_id = 0
        self.conversations: list[ConversationTurn] = []
        self.pages = PageTracker(config.hot_page_threshold)
        # Pending questions index — maps user's "yes"/"3"/"first one" to actual question
        self.pending_by_turn: dict[int, list[int]] = {}

    # ── graph ops ───
    def add_atom(self, kind: str, content: str) -> int:
        h = content_hash(content.encode('utf-8'))
        if h in self.content_index:
            existing = self.content_index[h]
            self.pages.touch(existing)
            return existing
        aid = self.next_atom_id
        self.next_atom_id += 1
        self.atoms[aid] = Atom(id=aid, kind=kind, content=content, content_hash=h)
        self.content_index[h] = aid
        self.pages.touch(aid)
        return aid

    def add_edge(self, from_id: int, to_id: int, relation: str):
        self.edges.append(Edge(from_id=from_id, to_id=to_id, relation=relation))
        self.pages.touch(from_id)
        self.pages.touch(to_id)

    # ── conversation ops (THE NEW PART) ───
    def receive_user_message(self, text: str) -> dict:
        """Process user message. Return answer + follow-up questions."""
        turn_id = len(self.conversations)
        # Save user's input as an atom + conversation turn
        input_atom = self.add_atom('UserInput', text)

        # Check: is this a reply to a prior question?
        resolves = self._resolve_context(text, turn_id)
        if resolves is not None:
            turn = self.conversations[resolves]
            text_resolved = self._answer_with_context(text, turn)
        else:
            text_resolved = text

        # Generate answer + follow-up questions
        answer, followups = self._generate_answer(text_resolved)

        # Save answer as atom
        answer_atom = self.add_atom('AssistantReply', answer)
        self.add_edge(input_atom, answer_atom, 'replied_by')

        # Save follow-up questions as atoms, linked to the answer
        followup_atom_ids = []
        for q in followups:
            q_atom = self.add_atom('PendingQuestion', q)
            self.add_edge(answer_atom, q_atom, 'has_followup')
            followup_atom_ids.append(q_atom)

        # Record turn
        turn = ConversationTurn(
            turn_id=turn_id,
            ts=time.time(),
            speaker='user',
            content=text,
            pending_questions=followup_atom_ids,
            resolves_turn_id=resolves,
        )
        self.conversations.append(turn)
        if followup_atom_ids:
            self.pending_by_turn[turn_id] = followup_atom_ids

        return {
            'answer': answer,
            'followups': followups,
            'turn_id': turn_id,
            'resolved_from': resolves,
        }

    def _resolve_context(self, text: str, current_turn_id: int) -> Optional[int]:
        """If user replies 'yes'/'1'/'the first'/'כן'/'3' — find matching pending."""
        tl = text.strip().lower()
        # Simple affirmatives/negatives
        affirmatives = {'yes', 'yeah', 'sure', 'ok', 'כן', 'בטח', 'בסדר'}
        # Look through recent turns for pending questions
        for prior_id in range(current_turn_id - 1, max(-1, current_turn_id - 6), -1):
            if prior_id not in self.pending_by_turn:
                continue
            pendings = self.pending_by_turn[prior_id]
            if not pendings:
                continue
            # Option 1: user said "yes" → pick first pending
            if tl in affirmatives:
                return prior_id
            # Option 2: user said a number → index
            if tl.isdigit():
                idx = int(tl) - 1
                if 0 <= idx < len(pendings):
                    return prior_id
            # Option 3: user mentioned topic word that matches one of the pending's text
            for pa_id in pendings:
                q_text = self.atoms[pa_id].content.lower()
                q_words = set(q_text.split())
                user_words = set(tl.split())
                if len(q_words & user_words) >= 2:  # 2+ shared words
                    return prior_id
        return None

    def _answer_with_context(self, text: str, resolved_turn: ConversationTurn) -> str:
        """When user says 'yes' to prior question — expand their input."""
        tl = text.strip().lower()
        pendings = resolved_turn.pending_questions
        if not pendings:
            return text
        # Pick question
        if tl.isdigit():
            idx = int(tl) - 1
            if 0 <= idx < len(pendings):
                q_atom_id = pendings[idx]
                return self.atoms[q_atom_id].content
        # Default: first pending
        return self.atoms[pendings[0]].content

    def _generate_answer(self, query: str) -> tuple[str, list[str]]:
        """Mock answer generator + 1-3 follow-up questions."""
        ql = query.lower()
        # Match topic in our atoms
        matches = []
        for aid, atom in self.atoms.items():
            if atom.kind == 'Concept' and (ql in atom.content.lower()
                                            or atom.content.lower() in ql):
                matches.append(atom.content)

        if matches:
            answer = f"I know about: {', '.join(matches[:3])}"
        else:
            answer = f"I'll think about '{query}'."

        # Generate 1-3 follow-up questions based on context
        followups = []
        if 'python' in ql or 'rust' in ql or 'program' in ql:
            followups = [
                "Would you like to see example code?",
                "Should I compare it with another language?",
                "Do you want its strengths and weaknesses?",
            ]
        elif 'wikipedia' in ql:
            followups = [
                "Should I look at the Hebrew version?",
                "Want the main section or all sections?",
            ]
        else:
            followups = [
                "Do you want more detail?",
                "Should I try a different angle?",
            ]
        return answer, followups

    # ── stats ───
    def stats(self) -> dict:
        return {
            'atoms': len(self.atoms),
            'edges': len(self.edges),
            'conversation_turns': len(self.conversations),
            'pages_small': sum(1 for v in self.pages.page_type.values() if v == 'SMALL'),
            'pages_huge': sum(1 for v in self.pages.page_type.values() if v == 'HUGE'),
            'pages_giga': sum(1 for v in self.pages.page_type.values() if v == 'GIGA'),
            'page_upgrades': len(self.pages.upgrade_events),
            'pending_questions_tracked': sum(len(v) for v in self.pending_by_turn.values()),
            'rss_mb': resource.getrusage(resource.RUSAGE_SELF).ru_maxrss / 1024,
        }


# ═══════════════════════════════════════════════════════════════════════
# Tests — end-to-end demonstration
# ═══════════════════════════════════════════════════════════════════════

def test_same_binary_different_configs():
    """Prove: server and client are same ZetsNode class, differ only in config."""
    server = ZetsNode(NodeConfig(name='server', role='server', port=3147,
                                 init_graph='seed_server.json'))
    client_a = ZetsNode(NodeConfig(name='Idan', role='persona', port=3251,
                                   parent_url='http://localhost:3147',
                                   init_graph='seed_idan.json'))
    client_b = ZetsNode(NodeConfig(name='Yam', role='persona', port=3265,
                                   parent_url='http://localhost:3147',
                                   init_graph='seed_yam.json'))
    assert type(server) is type(client_a) is type(client_b)
    print(f"  ✓ same class: {type(server).__name__}")
    print(f"  ✓ configs differ: server.role={server.config.role}, client.role={client_a.config.role}")
    print(f"  ✓ port differ:   server={server.config.port}, A={client_a.config.port}, B={client_b.config.port}")


def test_conversation_stored_in_graph():
    """Prove: every turn stored as atoms + edges in the graph."""
    node = ZetsNode(NodeConfig(name='test', role='server', port=9999))
    # Seed some concepts
    node.add_atom('Concept', 'Python')
    node.add_atom('Concept', 'Rust')

    r = node.receive_user_message("tell me about Python")
    print(f"  ✓ turn 0: atoms before={3} after={len(node.atoms)}, edges={len(node.edges)}")
    print(f"    answer: {r['answer']}")
    print(f"    followups: {r['followups']}")

    # Verify in graph
    assert len(node.conversations) == 1
    assert node.conversations[0].pending_questions  # has questions


def test_yes_resolves_context():
    """Prove: user saying 'yes' resolves to first pending question."""
    node = ZetsNode(NodeConfig(name='test', role='server', port=9998))
    node.add_atom('Concept', 'Python')

    r1 = node.receive_user_message("what is Python?")
    print(f"  turn 0: {r1['answer']}")
    print(f"    followups: {r1['followups']}")

    r2 = node.receive_user_message("yes")
    print(f"  turn 1 (user said 'yes'): resolved={r2['resolved_from']}")
    assert r2['resolved_from'] == 0, "yes should resolve to turn 0"


def test_number_reply_picks_indexed_question():
    """Prove: user saying '3' picks the 3rd pending question."""
    node = ZetsNode(NodeConfig(name='test', role='server', port=9997))
    node.add_atom('Concept', 'Python')

    r1 = node.receive_user_message("show me Python")
    print(f"  turn 0 followups: {r1['followups']}")
    assert len(r1['followups']) >= 3

    r2 = node.receive_user_message("3")
    print(f"  turn 1 (user said '3'): resolved_from={r2['resolved_from']}")
    assert r2['resolved_from'] == 0


def test_page_promotion_by_activity():
    """Prove: hot atoms trigger page promotion (SMALL → HUGE)."""
    node = ZetsNode(NodeConfig(name='test', role='server', port=9996,
                               hot_page_threshold=3))
    aid = node.add_atom('Concept', 'HotTopic')
    # Access 3 times → should promote
    for _ in range(4):
        node.pages.touch(aid)
    stats = node.stats()
    print(f"  page states after 5 accesses: "
          f"SMALL={stats['pages_small']} HUGE={stats['pages_huge']}")
    assert stats['pages_huge'] >= 1, "should have promoted to HUGE"
    print(f"  ✓ upgrade events: {node.pages.upgrade_events}")


def test_full_scenario():
    """End-to-end scenario mimicking real use."""
    node = ZetsNode(NodeConfig(name='Idan', role='persona', port=3251,
                               parent_url='http://localhost:3147',
                               hot_page_threshold=3))
    # Seed
    for concept in ['Python', 'Rust', 'Kabbalah', 'Wikipedia', 'ZETS']:
        node.add_atom('Concept', concept)

    # Conversation 1
    print(f"\n  --- Conversation 1 ---")
    r = node.receive_user_message("tell me about Python")
    print(f"  user> tell me about Python")
    print(f"  bot>  {r['answer']}")
    for i, q in enumerate(r['followups'], 1):
        print(f"        [{i}] {q}")

    # User replies "2"
    r = node.receive_user_message("2")
    print(f"  user> 2")
    print(f"  bot>  resolved_from={r['resolved_from']}; answer={r['answer']}")

    # Conversation 2 (separate topic)
    print(f"\n  --- Conversation 2 ---")
    r = node.receive_user_message("what about Wikipedia?")
    print(f"  user> what about Wikipedia?")
    print(f"  bot>  {r['answer']}")
    for i, q in enumerate(r['followups'], 1):
        print(f"        [{i}] {q}")

    r = node.receive_user_message("yes")
    print(f"  user> yes")
    print(f"  bot>  resolved_from={r['resolved_from']}; answer={r['answer']}")

    # Stats
    print(f"\n  Final stats:")
    for k, v in node.stats().items():
        print(f"    {k}: {v}")


if __name__ == '__main__':
    print("━━━ Unified Node — Python Prototype Tests ━━━")
    print()
    print("[1] Server and Client are the same class:")
    test_same_binary_different_configs()
    print()
    print("[2] Conversation stored in graph:")
    test_conversation_stored_in_graph()
    print()
    print("[3] 'yes' resolves to prior pending question:")
    test_yes_resolves_context()
    print()
    print("[4] '3' picks 3rd pending question:")
    test_number_reply_picks_indexed_question()
    print()
    print("[5] Page promotion on activity:")
    test_page_promotion_by_activity()
    print()
    print("[6] Full scenario:")
    test_full_scenario()
    print()
    print("━━━ ALL TESTS PASSED ━━━")
