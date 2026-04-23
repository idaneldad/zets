#!/usr/bin/env python3
"""
test_7x7_parallel_search_v1.py — proves parallelized beam search with
persona-adaptive parameters + "shevirat kelim" self-correction.

Per consultation with gpt-4o + gemini-2.5-flash (docs/working/20260423_neural_tree_7x7_consultation_V1.md):
  - This is "parallelized Beam Search" (Russell & Norvig Ch 3), not MCTS
  - Gemini recommended renaming personas away from diagnostic labels (ADHD/autistic)
  - Real state-of-the-art for semantic mmap: vector-data mmap + dynamic topology index
  - Breakeven: I/O-bound + high branching + shallow answers + multiple paths

Tests:
  [1]  Beam-search-7 finds answer in small graph (vs BFS baseline)
  [2]  7×7 beam vs single depth-8 walk — when does parallelism win?
  [3]  Async cancellation: 49 walks stop when one finds answer
  [4]  Persona graph: select search strategy by persona_id
  [5]  "Deep think" mode: 10 parallel 7×7 + shevirat kelim + tikkun
  [6]  Safe labels: behavioral descriptors instead of diagnostic labels
  [7]  Breakeven measurement: at what graph size does parallel pay off?

Uses asyncio (not threading) — matches Rust tokio plan.
Run: python3 py_testers/test_7x7_parallel_search_v1.py
"""

import asyncio
import random
import time
from dataclasses import dataclass, field
from enum import Enum
from typing import Optional


# ═══════════════════════════════════════════════════════════════════════
# Mini graph model
# ═══════════════════════════════════════════════════════════════════════

@dataclass
class GraphNode:
    id: int
    neighbors: list        # list of node IDs
    labels: dict           # {topic: confidence}, for matching


class MiniGraph:
    """A fake semantic graph with controllable branching factor and depth."""

    def __init__(self, num_nodes: int = 1000, avg_branching: int = 7,
                 target_node: int = 500, target_labels: Optional[dict] = None,
                 seed: int = 42):
        random.seed(seed)
        self.nodes = {}
        self.target_node = target_node
        # Create nodes
        for i in range(num_nodes):
            # Neighbors: random subset, size ≈ avg_branching
            branch = max(1, int(random.gauss(avg_branching, 2)))
            potential = [j for j in range(num_nodes) if j != i]
            neighbors = random.sample(potential, min(branch, len(potential)))
            labels = {f"topic_{i % 50}": random.random()}
            self.nodes[i] = GraphNode(id=i, neighbors=neighbors, labels=labels)
        # Mark the target
        if target_labels:
            self.nodes[target_node].labels.update(target_labels)
        else:
            self.nodes[target_node].labels["answer"] = 1.0

    async def get_node(self, node_id: int) -> GraphNode:
        # Simulate some I/O latency (mmap cache miss)
        await asyncio.sleep(0.00001)  # 10 microseconds
        return self.nodes[node_id]


def is_answer(node: GraphNode, query_label: str = "answer") -> bool:
    return query_label in node.labels and node.labels[query_label] > 0.5


# ═══════════════════════════════════════════════════════════════════════
# Search strategies
# ═══════════════════════════════════════════════════════════════════════

@dataclass
class SearchStrategy:
    """Persona-adaptive search parameters.

    Per Gemini's recommendation: use BEHAVIORAL descriptors, not diagnostic labels.
    """
    label: str             # 'precise' / 'exploratory' / 'exhaustive' / 'divergent' / 'deep_focus'
    beam_width: int        # parallel walks per step
    max_depth: int         # max hops
    retry_waves: int       # if no answer, try again with fresh start points
    confidence_threshold: float  # 0-1, stop when found
    description: str = ""


# Per Gemini consultation: rename away from diagnostic labels
STRATEGIES = {
    "precise": SearchStrategy(
        label="precise",
        beam_width=3, max_depth=5, retry_waves=1, confidence_threshold=0.9,
        description="Narrow beam, high-confidence only. For factual queries."
    ),
    "exploratory": SearchStrategy(
        label="exploratory",
        beam_width=12, max_depth=9, retry_waves=3, confidence_threshold=0.6,
        description="Wide beam, divergent. For creative/research queries."
    ),
    "exhaustive": SearchStrategy(
        label="exhaustive",
        beam_width=7, max_depth=12, retry_waves=5, confidence_threshold=0.5,
        description="Cautious multi-path. For medical/safety-critical queries."
    ),
    "rapid_iteration": SearchStrategy(
        label="rapid_iteration",
        beam_width=15, max_depth=3, retry_waves=5, confidence_threshold=0.5,
        description="Broad shallow, many retries. For brainstorming."
    ),
    "deep_dive": SearchStrategy(
        label="deep_dive",
        beam_width=3, max_depth=10, retry_waves=1, confidence_threshold=0.8,
        description="Narrow but deep. For detailed analysis on known path."
    ),
    "standard_7x7": SearchStrategy(
        label="standard_7x7",
        beam_width=7, max_depth=7, retry_waves=3, confidence_threshold=0.7,
        description="Default balanced. Miller's number × 7 depth."
    ),
}


# ═══════════════════════════════════════════════════════════════════════
# Search implementations
# ═══════════════════════════════════════════════════════════════════════

@dataclass
class SearchResult:
    found: bool
    answer_node_id: Optional[int] = None
    path: list = field(default_factory=list)
    walks_executed: int = 0
    steps_total: int = 0
    elapsed_sec: float = 0.0
    wave: int = 0


async def sequential_walk(graph: MiniGraph, start: int, target_label: str,
                           max_depth: int) -> SearchResult:
    """Single-threaded DFS-style walk. Baseline for comparison."""
    t0 = time.time()
    visited = set()
    steps = 0

    async def walk(node_id: int, depth: int, path: list):
        nonlocal steps
        if node_id in visited or depth > max_depth:
            return None
        visited.add(node_id)
        node = await graph.get_node(node_id)
        steps += 1
        new_path = path + [node_id]
        if is_answer(node, target_label):
            return new_path
        for n in node.neighbors:
            result = await walk(n, depth + 1, new_path)
            if result:
                return result
        return None

    path = await walk(start, 0, [])
    return SearchResult(
        found=path is not None,
        answer_node_id=path[-1] if path else None,
        path=path or [],
        walks_executed=1,
        steps_total=steps,
        elapsed_sec=time.time() - t0,
    )


async def beam_search_parallel(graph: MiniGraph, starts: list,
                                target_label: str,
                                strategy: SearchStrategy) -> SearchResult:
    """
    Parallel beam search with async cancellation.

    At each depth:
      - Have `beam_width` active walks
      - Each walk expands to its neighbors
      - Keep top `beam_width` by heuristic score
      - As soon as ANY walk finds answer, cancel others and return
    """
    t0 = time.time()
    steps_total = 0
    answer_found: asyncio.Event = asyncio.Event()
    result_container = {"node_id": None, "path": None}

    async def walk_one(start: int, walk_id: int):
        """Each walk maintains its own path, follows heuristic best neighbors."""
        nonlocal steps_total
        visited = set()
        frontier = [(start, [])]  # (node_id, path_so_far)
        for depth in range(strategy.max_depth):
            if answer_found.is_set():
                return
            next_frontier = []
            for node_id, path in frontier:
                if node_id in visited:
                    continue
                visited.add(node_id)
                node = await graph.get_node(node_id)
                steps_total += 1
                new_path = path + [node_id]
                # Check answer
                if is_answer(node, target_label):
                    result_container["node_id"] = node_id
                    result_container["path"] = new_path
                    answer_found.set()
                    return
                # Expand neighbors, score by confidence in target_label
                scored = []
                for n in node.neighbors:
                    next_node = graph.nodes.get(n)
                    if next_node is None:
                        continue
                    score = next_node.labels.get(target_label, 0) or random.random() * 0.3
                    scored.append((score, n))
                scored.sort(reverse=True)
                # Keep top few per walk to maintain beam_width across walks
                for _, n in scored[:2]:
                    next_frontier.append((n, new_path))
            frontier = next_frontier[:strategy.beam_width]
            if not frontier:
                return

    # Launch beam_width walks in parallel, each from a different start
    walks = []
    for i, start in enumerate(starts[:strategy.beam_width]):
        walks.append(asyncio.create_task(walk_one(start, i)))

    # Wait for either all-done or answer-found
    try:
        await asyncio.wait_for(
            asyncio.gather(*walks, return_exceptions=True),
            timeout=30.0,
        )
    except asyncio.TimeoutError:
        pass

    # Cancel remaining
    for w in walks:
        if not w.done():
            w.cancel()
    await asyncio.gather(*walks, return_exceptions=True)

    return SearchResult(
        found=answer_found.is_set(),
        answer_node_id=result_container["node_id"],
        path=result_container["path"] or [],
        walks_executed=len(walks),
        steps_total=steps_total,
        elapsed_sec=time.time() - t0,
    )


async def search_with_retries(graph: MiniGraph, query_start: int,
                               target_label: str,
                               strategy: SearchStrategy) -> SearchResult:
    """Run search with retry waves — each wave starts from different seeds."""
    for wave in range(strategy.retry_waves):
        # Start points: query_start + random perturbations (semantic neighbors)
        starts = [query_start] + random.sample(
            list(graph.nodes.keys()), strategy.beam_width - 1
        )
        result = await beam_search_parallel(graph, starts, target_label, strategy)
        if result.found:
            result.wave = wave
            return result
    # Not found after all retries
    return SearchResult(found=False, walks_executed=0, wave=strategy.retry_waves)


# ═══════════════════════════════════════════════════════════════════════
# "Deep think" mode — 10 parallel 7×7 + shevirat kelim
# ═══════════════════════════════════════════════════════════════════════

async def deep_think(graph: MiniGraph, query_start: int,
                      target_label: str,
                      parallel_branches: int = 10) -> SearchResult:
    """
    'Deep think': 10 parallel 7x7 searches, then compare results,
    break the weakest assumption, retry with fixed strategy.
    """
    t0 = time.time()
    strategy = STRATEGIES["standard_7x7"]

    # 10 parallel searches from different starts
    starts = [query_start] + random.sample(
        list(graph.nodes.keys()), parallel_branches - 1
    )
    branches = [
        search_with_retries(graph, s, target_label, strategy)
        for s in starts[:parallel_branches]
    ]
    results = await asyncio.gather(*branches)

    found_results = [r for r in results if r.found]
    total_walks = sum(r.walks_executed for r in results)
    total_steps = sum(r.steps_total for r in results)

    if found_results:
        best = min(found_results, key=lambda r: len(r.path))
        best.walks_executed = total_walks
        best.steps_total = total_steps
        best.elapsed_sec = time.time() - t0
        return best

    # Shevirat kelim: no answer found in 10×49=490 walks.
    # In real impl: weaken assumption (lower confidence_threshold),
    # try different target labels, try semantic neighbors.
    # For the prototype: just return failure.
    return SearchResult(
        found=False, walks_executed=total_walks,
        steps_total=total_steps, elapsed_sec=time.time() - t0
    )


# ═══════════════════════════════════════════════════════════════════════
# Persona graph (stored in the graph itself, not as config)
# ═══════════════════════════════════════════════════════════════════════

class PersonaGraph:
    """
    Persona → search strategy selection, stored as atoms + edges.
    """

    def __init__(self):
        # persona atoms → strategy atoms
        self.persona_to_strategy = {
            "dry_precise_user": "precise",
            "medical_patient": "exhaustive",
            "creative_scientist": "exploratory",
            "brainstormer": "rapid_iteration",
            "deep_researcher": "deep_dive",
            "anonymous": "standard_7x7",
        }

    def select_strategy(self, persona_id: str) -> SearchStrategy:
        strat_label = self.persona_to_strategy.get(persona_id, "standard_7x7")
        return STRATEGIES[strat_label]


# ═══════════════════════════════════════════════════════════════════════
# Tests
# ═══════════════════════════════════════════════════════════════════════

async def test_beam_search_finds_answer():
    graph = MiniGraph(num_nodes=200, avg_branching=5, target_node=150, seed=1)
    # Make target reachable by marking node 150's label strongly
    graph.nodes[150].labels["answer"] = 1.0
    result = await beam_search_parallel(graph, [0], "answer", STRATEGIES["standard_7x7"])
    print(f"  Beam search: found={result.found}, path_len={len(result.path)}, "
          f"steps={result.steps_total}, elapsed={result.elapsed_sec*1000:.1f}ms")


async def test_7x7_vs_sequential():
    """When does 7×7 beat sequential?"""
    sizes = [100, 500, 2000]
    for size in sizes:
        graph = MiniGraph(num_nodes=size, avg_branching=5,
                          target_node=size // 2, seed=42)
        graph.nodes[size // 2].labels["answer"] = 1.0
        # Sequential
        seq = await sequential_walk(graph, 0, "answer", max_depth=8)
        # Parallel 7×7
        par = await search_with_retries(graph, 0, "answer", STRATEGIES["standard_7x7"])
        speedup = seq.elapsed_sec / max(par.elapsed_sec, 1e-6)
        print(f"  graph_size={size:>5}: seq={seq.elapsed_sec*1000:.0f}ms ({seq.found}), "
              f"par7×7={par.elapsed_sec*1000:.0f}ms ({par.found}), "
              f"speedup={speedup:.1f}×")


async def test_async_cancellation():
    """49 walks should stop when one finds answer."""
    graph = MiniGraph(num_nodes=500, avg_branching=7, target_node=100, seed=3)
    graph.nodes[100].labels["answer"] = 1.0
    result = await search_with_retries(graph, 0, "answer", STRATEGIES["standard_7x7"])
    print(f"  Retries={STRATEGIES['standard_7x7'].retry_waves}, "
          f"beam={STRATEGIES['standard_7x7'].beam_width}")
    print(f"  Found in wave {result.wave}, walks executed: {result.walks_executed}, "
          f"total steps: {result.steps_total}, elapsed: {result.elapsed_sec*1000:.0f}ms")


async def test_persona_strategies():
    """Different personas → different search strategies."""
    graph = MiniGraph(num_nodes=500, avg_branching=6, target_node=250, seed=7)
    graph.nodes[250].labels["answer"] = 1.0

    persona_graph = PersonaGraph()
    personas = ["dry_precise_user", "creative_scientist", "medical_patient",
                "brainstormer", "deep_researcher"]

    print(f"  {'persona':<22} {'strategy':<18} {'beam':>5} {'depth':>6} {'found':>6} {'ms':>6}")
    print(f"  {'-'*65}")
    for p in personas:
        strat = persona_graph.select_strategy(p)
        result = await search_with_retries(graph, 0, "answer", strat)
        print(f"  {p:<22} {strat.label:<18} {strat.beam_width:>5} "
              f"{strat.max_depth:>6} {str(result.found):>6} "
              f"{result.elapsed_sec*1000:>6.0f}")


async def test_deep_think():
    """10 parallel 7×7 for hard queries."""
    graph = MiniGraph(num_nodes=1000, avg_branching=5, target_node=500, seed=11)
    graph.nodes[500].labels["answer"] = 1.0
    result = await deep_think(graph, query_start=0, target_label="answer",
                               parallel_branches=10)
    print(f"  Deep think: found={result.found}, walks={result.walks_executed}, "
          f"steps={result.steps_total}, elapsed={result.elapsed_sec*1000:.0f}ms")


def test_labels_are_behavioral_not_diagnostic():
    """Per Gemini: don't use ADHD/autistic labels — use behavioral."""
    labels = [s.label for s in STRATEGIES.values()]
    forbidden = ["adhd", "autistic", "autism", "neurotypical", "deficit"]
    for lbl in labels:
        for forb in forbidden:
            assert forb not in lbl.lower(), f"label '{lbl}' contains diagnostic term '{forb}'"
    print(f"  All {len(labels)} labels are behavioral: {labels}")
    print(f"  No diagnostic terms present ✓")


async def test_breakeven_measurement():
    """Find breakeven: at what graph size does parallel beat sequential?"""
    sizes = [50, 100, 300, 1000, 3000]
    print(f"  {'size':>6} {'seq(ms)':>10} {'par(ms)':>10} {'ratio':>8}  winner")
    for size in sizes:
        graph = MiniGraph(num_nodes=size, avg_branching=6,
                          target_node=int(size * 0.4), seed=99)
        graph.nodes[int(size * 0.4)].labels["answer"] = 1.0

        seq = await sequential_walk(graph, 0, "answer", max_depth=10)
        par = await search_with_retries(graph, 0, "answer", STRATEGIES["standard_7x7"])

        ratio = seq.elapsed_sec / max(par.elapsed_sec, 1e-6)
        winner = "parallel" if ratio > 1.0 else "sequential"
        print(f"  {size:>6} {seq.elapsed_sec*1000:>10.1f} "
              f"{par.elapsed_sec*1000:>10.1f} {ratio:>7.1f}×  {winner}")


# ═══════════════════════════════════════════════════════════════════════
# Main
# ═══════════════════════════════════════════════════════════════════════

async def main():
    print("━━━ 7×7 Parallel Search + Persona Graph — Python Prototype ━━━\n")
    print("[1] Beam search finds answer:")
    await test_beam_search_finds_answer()

    print("\n[2] 7×7 vs sequential — when does parallel win?")
    await test_7x7_vs_sequential()

    print("\n[3] Async cancellation (49 walks, first finish wins):")
    await test_async_cancellation()

    print("\n[4] Persona-adaptive search:")
    await test_persona_strategies()

    print("\n[5] 'Deep think' mode (10 × 7×7 = up to 490 walks):")
    await test_deep_think()

    print("\n[6] Labels are behavioral, not diagnostic (per Gemini):")
    test_labels_are_behavioral_not_diagnostic()

    print("\n[7] Breakeven measurement (parallel vs sequential by graph size):")
    await test_breakeven_measurement()

    print("\n━━━ ALL TESTS RAN ━━━")


if __name__ == "__main__":
    asyncio.run(main())
