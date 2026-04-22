"""
Enhanced Hopfield PoC with:
1. Threshold-based activation (knows when "nothing matches")
2. Top-K retrieval (not just top-1)
3. Multi-bank decomposition with confidence
4. Realistic test: decompose scene where only SOME banks should activate
"""
import numpy as np
import time


class ModernHopfield:
    def __init__(self, beta: float = 8.0, activation_threshold: float = 0.4):
        self.patterns = []
        self.X = None
        self.beta = beta
        self.threshold = activation_threshold

    def store(self, pattern):
        p = pattern / (np.linalg.norm(pattern) + 1e-8)
        self.patterns.append(p)
        self.X = np.stack(self.patterns)

    def recall_top_k(self, cue, k: int = 3, threshold: float = None):
        """Return up to k (index, similarity) pairs above threshold.
        
        Returns empty list if no pattern exceeds threshold — 
        this is the key fix: Hopfield says 'I don't recognize anything'.
        """
        if self.X is None:
            return []
        t = threshold if threshold is not None else self.threshold
        xi = cue / (np.linalg.norm(cue) + 1e-8)
        scores = self.X @ xi
        # argsort descending
        order = np.argsort(-scores)
        hits = []
        for idx in order[:k]:
            sim = float(scores[idx])
            if sim >= t:
                hits.append((int(idx), sim))
            else:
                break  # sorted, no more above threshold
        return hits


def test_genesis_selective_activation():
    """Realistic scene: only 3 banks should fire.
    
    This is the ACTUAL use case Idan described:
    - Input: image of a child hugging a dog on a grassy field
    - Expected activations: day3_land (grass), day6_mammal (dog), 
                            day6_human (child), interaction (hug)
    - Should NOT activate: day4_luminary, day5_fish_bird, day2_sky
    """
    D = 64
    rng = np.random.default_rng(99)

    banks = {
        "day1_light":      (ModernHopfield(beta=6.0, activation_threshold=0.4), 100),
        "day2_sky":        (ModernHopfield(beta=6.0, activation_threshold=0.4), 100),
        "day3_land":       (ModernHopfield(beta=6.0, activation_threshold=0.4), 100),
        "day4_luminary":   (ModernHopfield(beta=6.0, activation_threshold=0.4), 100),
        "day5_fish_bird":  (ModernHopfield(beta=6.0, activation_threshold=0.4), 100),
        "day6_mammal":     (ModernHopfield(beta=6.0, activation_threshold=0.4), 100),
        "day6_human":      (ModernHopfield(beta=6.0, activation_threshold=0.4), 100),
        "interaction":     (ModernHopfield(beta=6.0, activation_threshold=0.4), 100),
    }

    bank_atoms = {}
    for name, (net, n) in banks.items():
        atoms = rng.standard_normal((n, D))
        for a in atoms:
            net.store(a)
        bank_atoms[name] = atoms

    # Scene: outdoor playground — child hugging dog on grass, with light
    # but NOT sky-dominant, no birds/fish, no sun-in-frame
    scene = (
        0.10 * bank_atoms["day1_light"][15]   # bright sunshine
        + 0.20 * bank_atoms["day3_land"][22]   # grass
        + 0.30 * bank_atoms["day6_human"][7]   # child
        + 0.30 * bank_atoms["day6_mammal"][13] # dog
        + 0.25 * bank_atoms["interaction"][42] # hug
    )
    scene_n = scene / np.linalg.norm(scene)

    print("=== Realistic scene: child hugs dog on grass ===")
    print("Expected activations: day1_light, day3_land, day6_mammal, day6_human, interaction")
    print("Expected NO activation: day2_sky, day4_luminary, day5_fish_bird\n")
    for name, (net, _) in banks.items():
        hits = net.recall_top_k(scene_n, k=3, threshold=0.4)
        if hits:
            hit_str = ", ".join(f"atom[{i}]={s:+.3f}" for i, s in hits)
            print(f"  ★ {name:<20} → {hit_str}")
        else:
            print(f"    {name:<20} → (silent)")

    # Determine correctness
    expected_on = {"day1_light", "day3_land", "day6_human", "day6_mammal", "interaction"}
    expected_off = {"day2_sky", "day4_luminary", "day5_fish_bird"}

    for threshold in [0.25, 0.30, 0.35, 0.40, 0.45, 0.50]:
        correct_on, correct_off = 0, 0
        for name, (net, _) in banks.items():
            hits = net.recall_top_k(scene_n, k=1, threshold=threshold)
            if name in expected_on and hits:
                correct_on += 1
            if name in expected_off and not hits:
                correct_off += 1
        total_correct = correct_on + correct_off
        total_all = len(expected_on) + len(expected_off)
        print(f"\n  threshold={threshold:.2f}: on={correct_on}/{len(expected_on)} off={correct_off}/{len(expected_off)} total={total_correct}/{total_all}")


def test_noise_rejection():
    """Critical test: what if the cue is RANDOM NOISE? 
    A robust recall should return empty or very low scores."""
    D = 128
    N = 500
    rng = np.random.default_rng(77)

    net = ModernHopfield(beta=8.0, activation_threshold=0.4)
    patterns = rng.standard_normal((N, D))
    for p in patterns:
        net.store(p)

    print("\n=== Noise rejection test ===")
    # Test 1: a real pattern should recall correctly
    hits = net.recall_top_k(patterns[42], k=1, threshold=0.4)
    print(f"  True pattern: hits={len(hits)}, sim={hits[0][1]:.3f}" if hits else "FAIL")

    # Test 2: pure random noise should have weak scores
    noise = rng.standard_normal(D)
    hits = net.recall_top_k(noise, k=3, threshold=0.4)
    if hits:
        print(f"  FAIL: noise produced {len(hits)} hits: {hits}")
    else:
        # Report max similarity to see how close we got to firing
        xi = noise / np.linalg.norm(noise)
        max_sim = float((net.X @ xi).max())
        print(f"  Pure noise: correctly rejected. Max similarity = {max_sim:.3f} (below threshold 0.4)")


def test_hierarchical_composition():
    """Rex = Quadruped + Dog + Poodle + Color.
    Can Hopfield atom bank recall all layers given the full vector?"""
    D = 128
    rng = np.random.default_rng(11)

    # 4 banks matching ZETS prototype chain
    mammal_bank = ModernHopfield(beta=8.0, activation_threshold=0.3)
    quadruped_bank = ModernHopfield(beta=8.0, activation_threshold=0.3)
    species_bank = ModernHopfield(beta=8.0, activation_threshold=0.3)
    breed_bank = ModernHopfield(beta=8.0, activation_threshold=0.3)
    color_bank = ModernHopfield(beta=8.0, activation_threshold=0.3)

    # 10 patterns per bank
    mammals = rng.standard_normal((5, D))
    quadrupeds = rng.standard_normal((3, D))   # Canine, Bovine, Equine
    species = rng.standard_normal((20, D))     # 20 dog species, etc.
    breeds = rng.standard_normal((100, D))     # 100 breeds
    colors = rng.standard_normal((15, D))      # 15 coat colors

    for p in mammals: mammal_bank.store(p)
    for p in quadrupeds: quadruped_bank.store(p)
    for p in species: species_bank.store(p)
    for p in breeds: breed_bank.store(p)
    for p in colors: color_bank.store(p)

    # Rex = mammal[0] + quadruped[0] + species[5] + breed[42] + color[3]
    # (each weighted equally because they're orthogonal levels)
    rex = (mammals[0] + quadrupeds[0] + species[5] + breeds[42] + colors[3]) / 5.0
    rex_n = rex / np.linalg.norm(rex)

    print("\n=== Hierarchical decomposition: 'Rex' ===")
    print("Built from: mammal[0] + quadruped[0] + species[5] + breed[42] + color[3]")
    for name, bank, expected in [
        ("mammal", mammal_bank, 0),
        ("quadruped", quadruped_bank, 0),
        ("species", species_bank, 5),
        ("breed", breed_bank, 42),
        ("color", color_bank, 3),
    ]:
        hits = bank.recall_top_k(rex_n, k=1, threshold=0.3)
        if hits:
            idx, sim = hits[0]
            match = "✓" if idx == expected else "✗"
            print(f"  {match} {name:<10} → atom[{idx:>3}] sim={sim:+.3f}  (expected {expected})")
        else:
            print(f"  ✗ {name:<10} → no match above threshold")


if __name__ == "__main__":
    test_genesis_selective_activation()
    test_noise_rejection()
    test_hierarchical_composition()
