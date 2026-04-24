# ZETS Value Ranges — Master Reference

**Created:** 2026-04-24  
**Purpose:** Single source of truth for every variable's storage size, range, and decimal precision in ZETS.

---

## Executive Summary

ZETS uses **fixed-point integers exclusively** in storage — no floats on disk. Floats appear only transiently during decay computation, then quantize back to u8.

**Why fixed-point?**
- 8× smaller than float64
- Integer math = 1 CPU cycle vs Float = 4-7 cycles
- Deterministic (same input = same output, always)
- No NaN/Inf/denormal edge cases

---

## 1. ATOM (8 bytes = 64 bits, packed u64)

| Field | Bits | Range | Precision | Coverage |
|---|---|---|---|---|
| language | 4 | 0..15 | 16 discrete | 16 langs |
| atom_type | 4 | 0..15 | 16 discrete | 16 types |
| letters[11] | 55 | 0..31 each | 32 per slot | 11-letter words (99% Hebrew) |
| is_long_flag | 1 | 0 or 1 | binary | long-form lookup flag |

---

## 2. EDGE HOT (6 bytes = 48 bits)

| Field | Bits | Range | Precision | Coverage |
|---|---|---|---|---|
| dst_atom_index | 32 | 0..4.3 B | atom array index | 4.3B atoms |
| edge_type | 5 | 0..31 | 32 discrete | 21 used + 11 reserved |
| state_value | 4 | -8..+7 | step 0.125 in [-1, +1) | 16 bipolar levels |
| memory_strength | 4 | 0..15 | exponential 0..1 | 16 Ebbinghaus levels |
| flags | 3 | 0..7 | 3 boolean bits | ctx/state/del |

---

## 3. state_value Decimal Mapping (4 bits)

| Bucket | Binary | Decimal | Meaning |
|---|---|---|---|
| -8 | 1000 | -1.000 | התנגדות מוחלטת (anti) |
| -6 | 1010 | -0.750 | התנגדות חזקה |
| -4 | 1100 | -0.500 | התנגדות בינונית |
| -2 | 1110 | -0.250 | התנגדות חלשה |
| 0 | 0000 | 0.000 | ניטרלי |
| +2 | 0010 | +0.250 | תמיכה חלשה |
| +4 | 0100 | +0.500 | תמיכה בינונית |
| +6 | 0110 | +0.750 | תמיכה חזקה |
| +7 | 0111 | +0.875 | זהות מקסימלית |

**Step:** 0.125 (12.5% per bucket)  
**Why:** Semantic relations are not measurable beyond ~10% precision

---

## 4. memory_strength Decimal Mapping (4 bits, EXPONENTIAL)

| Bucket | Decimal | Meaning |
|---|---|---|
| 0 | 0.0000 | נשכח לחלוטין |
| 1 | 0.0016 | כמעט נשכח |
| 3 | 0.0040 | חלש מאוד |
| 5 | 0.0100 | חלש |
| 7 | 0.0251 | בינוני נמוך |
| 9 | 0.0631 | בינוני |
| 11 | 0.1585 | טרי-יחסית |
| 13 | 0.3981 | חזק |
| 15 | 1.0000 | מקסימלי (זה עתה) |

**Mapping:** `decimal = exp(bucket/15 × log(1000)) / 1000` (for bucket > 0)  
**Why exponential:** Matches Ebbinghaus forgetting curve scientifically

---

## 5. EDGE COLD Fields (lookup, ~5-10% of edges)

| Field | Bits | Range | Precision | Notes |
|---|---|---|---|---|
| confidence | 8 | 0..255 | step 0.0039 (0.39%) | sub-bucket of state |
| asymmetry_factor | 8 | 0..255 | step 0.0039 | 0=sym, 255=one-way |
| context_id | 32 | 0..4.3B | exact int | context tree node |
| state_axis_id | 32 | 0..4.3B | exact int | axis registry |
| active_range_min | 8 | 0..255 | step 0.0039 | axis range start |
| active_range_max | 8 | 0..255 | step 0.0039 | axis range end |
| created_at_days | 16 | 0..65535 | 1 day | 179 years coverage |
| last_used_days | 16 | 0..65535 | 1 day | 179 years coverage |
| use_count | 16 | 0..65535 | exact int | caps at 65K |
| source_type | 4 | 0..15 | 16 types | user/inference/etc |

---

## 6. SEFIROT VECTOR (intent classification, 11 bytes)

| Sefira | Range | Precision | Default |
|---|---|---|---|
| keter | 0..255 ÷ 255 | step 0.0039 | 0 |
| chokhma | 0..255 ÷ 255 | step 0.0039 | 0 |
| bina | 0..255 ÷ 255 | step 0.0039 | 0 |
| daat | 0..255 ÷ 255 | step 0.0039 | 0 |
| chesed | 0..255 ÷ 255 | step 0.0039 | 0 |
| gevura | 0..255 ÷ 255 | step 0.0039 | 0 |
| tiferet | 0..255 ÷ 255 | step 0.0039 | 0 |
| netzach | 0..255 ÷ 255 | step 0.0039 | 0 |
| hod | 0..255 ÷ 255 | step 0.0039 | 0 |
| yesod | 0..255 ÷ 255 | step 0.0039 | 0 |
| malkhut | 76..255 ÷ 255 | step 0.0039 | 0.3 minimum |

**Total:** 11 × 8 = 88 bits = 11 bytes per query intent vector

---

## 7. STATE AXIS Values (per-concept dimensions)

| Axis Kind | Storage | Math Range | Example |
|---|---|---|---|
| Scalar | u8 | 0.0..1.0 | ripeness, freshness |
| Bipolar | u8 | -1.0..+1.0 | polarity |
| Cyclic | u8 | 0..360° | season, time of day |
| Discrete | u8 | categories | color category |
| Temporal | u32 days | epoch+ | age in days |

All scalar/bipolar/cyclic: step 0.39% (256 buckets)

---

## 8. DECAY τ (Time constant in days)

```
τ = 10 + 20 × context_depth + 30 × √use_count   [days]
```

| depth | use | τ (days) | half-life | Meaning |
|---|---|---|---|---|
| 0 | 0 | 10 | 7 | Public fact, never used |
| 0 | 10 | 105 | 73 | Public, used 10× |
| 1 | 0 | 30 | 21 | Personal, fresh |
| 2 | 100 | 350 | 243 | Specific personal, often used |
| 3 | 1000 | 1019 | 706 | Critical memory |
| 3 | 10000 | 3070 | 2128 (5.8 yrs) | Lifelong memory |

---

## 9. Type Cheat Sheet

| Type | Bytes | Range | Precision | Use |
|---|---|---|---|---|
| bit | 1/8 | 0 or 1 | — | flags |
| u3 | 3/8 | 0..7 | 3 booleans | edge flags |
| u4 | 1/2 | 0..15 | 16 levels | type, mem_strength |
| i4 | 1/2 | -8..+7 | 16 bipolar | state_value |
| u5 | 5/8 | 0..31 | 32 values | letter slot |
| u8 | 1 | 0..255 | 0.39% | confidence, sefirot |
| u16 | 2 | 0..65535 | integer | use_count, days |
| u32 | 4 | 0..4.3B | integer | atom indices |
| u64 | 8 | 0..1.8E19 | integer | atom encoding |

---

## 10. Total Memory Budget

**At 10M atoms × 100M edges:**

| Component | Size |
|---|---|
| Atoms (HOT, u64 each) | 80 MB |
| Edges (HOT, 6B each) | 600 MB |
| Offsets (fwd + rev) | 80 MB |
| **HOT TOTAL** | **760 MB** |
| Long atoms (1%) | 3 MB |
| Edge contexts (10%) | 60 MB |
| Edge state_deps (5%) | 50 MB |
| Atom features (5%) | 15 MB |
| Context tree | 1 MB |
| State axes registry | 1 MB |
| **COLD TOTAL** | **130 MB** |
| **GRAND TOTAL** | **890 MB** |

**Scaling:**
- 1M × 10M: 90 MB (laptop)
- 10M × 100M: 890 MB (workstation)
- 100M × 1B: 8.9 GB (server)
- 1B × 10B: 89 GB (still fits SSD)

---

## 11. Decimal Precision Where It Matters

**No float in storage. Float only at compute time.**

```rust
// state_value: i4 → f32 (only when needed)
fn state_to_float(v: i8) -> f32 { v as f32 / 8.0 }
fn float_to_state(f: f32) -> i8 { (f * 8.0).round().clamp(-8.0, 7.0) as i8 }

// memory_strength: u4 → f32 (exponential mapping)
fn memory_to_float(b: u8) -> f32 {
    if b == 0 { 0.0 } 
    else { (b as f32 / 15.0 * 1000.0_f32.ln()).exp() / 1000.0 }
}

// confidence/asymmetry/sefirot: u8 → f32 (linear)
fn u8_to_float(v: u8) -> f32 { v as f32 / 255.0 }
fn float_to_u8(f: f32) -> u8 { (f * 255.0).round().clamp(0.0, 255.0) as u8 }
```

---

## Summary

**Total per-edge bytes: 6 bytes hot + ~12 bytes cold (rare)**  
**Total per-atom bytes: 8 bytes (u64 self-describing)**  
**Total per-query overhead: 11 bytes (sefirot vector)**  
**Total at production scale: < 1 GB for 10M concepts × 100M edges**

This is what "lean" looks like.
