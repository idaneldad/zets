"""
Modern Hopfield Network as 'recall engine' for pattern completion.

Tests:
1. Can store N patterns of dimension D
2. Can recall full pattern from partial cue
3. Throughput (patterns/sec for storage + lookup)
4. Capacity (how many patterns before catastrophic interference)

Reference: Ramsauer et al. 2020 "Hopfield Networks is All You Need"
Key insight: attention IS modern Hopfield. The update rule is softmax(beta * X @ xi) @ X.
"""
import numpy as np
import time


class ModernHopfield:
    """Modern Hopfield Network — exponential capacity, continuous patterns.

    Update rule (Ramsauer 2020):
        xi_new = X.T @ softmax(beta * X @ xi)

    where X = matrix of stored patterns (N x D), xi = query (D,), beta = inverse temperature.
    """

    def __init__(self, beta: float = 8.0):
        self.patterns = []
        self.X = None
        self.beta = beta

    def store(self, pattern):
        """Store a pattern (unit-normalized)."""
        p = pattern / (np.linalg.norm(pattern) + 1e-8)
        self.patterns.append(p)
        self.X = np.stack(self.patterns)

    def recall(self, cue, n_iter=1):
        """Recall — one iteration suffices for well-separated patterns."""
        if self.X is None:
            return cue
        xi = cue / (np.linalg.norm(cue) + 1e-8)
        for _ in range(n_iter):
            scores = self.beta * self.X @ xi
            scores -= scores.max()
            probs = np.exp(scores)
            probs /= probs.sum()
            xi = self.X.T @ probs
            xi = xi / (np.linalg.norm(xi) + 1e-8)
        return xi

    def recall_index(self, cue):
        """Which stored pattern is the cue most similar to?"""
        xi = cue / (np.linalg.norm(cue) + 1e-8)
        scores = self.X @ xi
        best = int(scores.argmax())
        return best, float(scores[best])


def test_capacity_d512():
    """How many patterns can we store in D=512 before recall breaks?"""
    D = 512
    print("=== Capacity test @ D=512, 30% mask ===")
    rng = np.random.default_rng(42)
    for N in [10, 100, 1000, 5000]:
        net = ModernHopfield(beta=8.0)
        patterns = rng.standard_normal((N, D))
        for p in patterns:
            net.store(p)

        correct = 0
        test_n = min(N, 100)
        for i in range(test_n):
            original = patterns[i]
            cue = original.copy()
            mask_idx = rng.choice(D, size=D // 3, replace=False)
            cue[mask_idx] = 0
            best, _ = net.recall_index(cue)
            if best == i:
                correct += 1
        acc = correct / test_n
        print(f"  N={N:>6}: recall accuracy = {acc:.1%}  ({correct}/{test_n})")


def test_partial_cue_retrieval():
    """Simulate 'dog ear → full dog head' pattern completion."""
    D = 256
    rng = np.random.default_rng(123)

    N = 1000
    heads = rng.standard_normal((N, D))
    poodle_head = heads[42].copy()

    net = ModernHopfield(beta=10.0)
    for h in heads:
        net.store(h)

    cue = np.zeros(D)
    cue[:64] = poodle_head[:64]

    t0 = time.perf_counter()
    recalled = net.recall(cue, n_iter=3)
    dt = time.perf_counter() - t0

    sim = np.dot(recalled, poodle_head / np.linalg.norm(poodle_head))
    print(f"\n=== Partial-cue recall (1/4 of D visible) ===")
    print(f"  Stored: {N} head patterns, D={D}")
    print(f"  Cue:    64/256 dims = 'ear region'")
    print(f"  Recall time: {dt*1000:.2f}ms")
    print(f"  Recall similarity to true poodle_head: {sim:.3f}  (1.0 = perfect)")

    t0 = time.perf_counter()
    n_queries = 1000
    for _ in range(n_queries):
        net.recall_index(poodle_head)
    dt = time.perf_counter() - t0
    print(f"  Throughput: {n_queries/dt:.0f} lookups/sec")


def test_composition():
    """Hopfield atoms composed: 'dog head' + 'labrador body' = 'labrador dog'?"""
    D = 128
    rng = np.random.default_rng(7)

    head_atoms = rng.standard_normal((50, D))
    body_atoms = rng.standard_normal((30, D))

    labrador = head_atoms[3] + body_atoms[7]
    labrador_n = labrador / np.linalg.norm(labrador)

    head_net = ModernHopfield(beta=8.0)
    for h in head_atoms:
        head_net.store(h)
    body_net = ModernHopfield(beta=8.0)
    for b in body_atoms:
        body_net.store(b)

    recalled_head, sim_h = head_net.recall_index(labrador_n)
    recalled_body, sim_b = body_net.recall_index(labrador_n)
    print(f"\n=== Decomposition test (dog = head + body) ===")
    print(f"  Input:  'labrador' = head[3] + body[7]")
    print(f"  Head network recalled index {recalled_head} (expected 3), sim={sim_h:.3f}")
    print(f"  Body network recalled index {recalled_body} (expected 7), sim={sim_b:.3f}")
    print(f"  Correct? head={recalled_head==3}, body={recalled_body==7}")


def test_genesis_order_decomposition():
    """Simulate Genesis-ordered image decomposition.

    Each 'day' is a Hopfield atom bank. Decomposition proceeds in order.
    """
    D = 64
    rng = np.random.default_rng(99)

    # 7 banks corresponding to creation days
    banks = {
        "day1_light":    ModernHopfield(beta=6.0),  # lighting patterns
        "day2_sky":      ModernHopfield(beta=6.0),  # sky/atmosphere
        "day3_land":     ModernHopfield(beta=6.0),  # terrain, vegetation
        "day4_luminary": ModernHopfield(beta=6.0),  # sun/moon/star in frame
        "day5_fish_bird": ModernHopfield(beta=6.0),
        "day6_mammal":   ModernHopfield(beta=6.0),
        "day6_human":    ModernHopfield(beta=6.0),
        "interaction":   ModernHopfield(beta=6.0),  # hug, gaze, holding
    }
    # Fill each bank with sample "atoms"
    n_per_bank = 100
    bank_atoms = {}
    for name, net in banks.items():
        atoms = rng.standard_normal((n_per_bank, D))
        for a in atoms:
            net.store(a)
        bank_atoms[name] = atoms

    # Construct a "scene embedding" as weighted sum of activated atoms from several banks
    scene = (
        0.15 * bank_atoms["day1_light"][5]   # bright lighting
        + 0.15 * bank_atoms["day3_land"][22]  # grass terrain
        + 0.30 * bank_atoms["day6_human"][7]  # one human
        + 0.25 * bank_atoms["day6_mammal"][13] # dog
        + 0.25 * bank_atoms["interaction"][42] # hug
    )
    scene_n = scene / np.linalg.norm(scene)

    print(f"\n=== Genesis-order scene decomposition ===")
    print(f"  Scene embedding dim: {D}")
    print(f"  Scene is weighted sum of 5 atoms across 5 banks")
    print(f"  Decomposition result:")
    for name, net in banks.items():
        idx, sim = net.recall_index(scene_n)
        marker = "★" if sim > 0.15 else " "
        print(f"    {marker} {name:<20} -> atom[{idx:>3}] sim={sim:+.3f}")

    print(f"  Expected activations: day1_light[5], day3_land[22], day6_human[7],")
    print(f"                         day6_mammal[13], interaction[42]")


if __name__ == "__main__":
    test_capacity_d512()
    test_partial_cue_retrieval()
    test_composition()
    test_genesis_order_decomposition()
