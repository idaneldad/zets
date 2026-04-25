## Top 5 Critical Issues

### ISS-01: Edge Storage Exceeds RAM Budget  
- **Section affected**: §20 Performance, §6 RAM + Disk Frugality  
- **Severity**: critical  
- **Confidence**: 100  
- **Claim**: Edge storage (1B edges × 6B = 6GB) consumes entire 6GB RAM budget, leaving no room for OS/working memory.  
- **Proposed patch**:  
```diff
- Edges (1B × 6B) | 6 GB (mmap, page in on demand)
+ Edges (500M × 4B VarInt) | 2 GB (delta-encoded CSR)
```  
- **Hidden assumption**: Edge distribution follows power law (sparse graphs).  
- **Self-objection**: VarInt decoding adds CPU overhead (10-20ns/edge).  
- **Validation test**: Measure CSR traversal latency with 500M-edge Hebrew Wikipedia corpus on 6GB device.  

### ISS-02: Missing Cold Start Optimization  
- **Section affected**: §20 Performance, §0.6 Hardware Target  
- **Severity**: important  
- **Confidence**: 95  
- **Claim**: <2s cold start impossible without precomputed Hopfield indices/CSR offsets.  
- **Proposed patch**: Add to §20:  
```markdown
**Cold start optimizations**:  
- Precompute Hopfield bank indexes during NightMode  
- Serialize CSR offsets to disk with O(1) load  
- Lazy-load string pool via mmap  
```  
- **Hidden assumption**: Disk I/O speed ≥200MB/s (SATA SSD).  
- **Self-objection**: Increases disk footprint by 10%.  
- **Validation test**: Measure startup latency on 5400RPM HDD vs SSD.  

### ISS-03: Undefined Deterministic Hash  
- **Section affected**: §2.2 Determinism  
- **Severity**: critical  
- **Confidence**: 100  
- **Claim**: No spec for `deterministic_hash(seed)` implementation.  
- **Proposed patch**: Add to §2.2:  
```rust
fn deterministic_hash(input: &[u8], seed: u64) -> u64 { 
    wyhash::wyhash(input, seed) // BSD-licensed, 3ns/byte
}
```  
- **Hidden assumption**: WyHash passes SMHasher tests.  
- **Self-objection**: WyHash collisions might bias walks.  
- **Validation test**: Chi-squared test on walk path distribution with 1M seeds.  

### ISS-04: Media Pipeline I/O Budget Missing  
- **Section affected**: §13 Media, §20 Performance  
- **Severity**: important  
- **Confidence**: 90  
- **Claim**: CLIP/Whisper I/O not budgeted for <100ms query cycle.  
- **Proposed patch**: Add to §20:  
```markdown
| Media embedding (CLIP) | 50-200 ms | External process |
| Audio transcription     | 100-300 ms | Whisper.cpp     |
```  
- **Hidden assumption**: CLIP/Whisper run as standalone binaries.  
- **Self-objection**: External processes add IPC overhead.  
- **Validation test**: Benchmark end-to-end "describe this image" query on Raspberry Pi 4.  

### ISS-05: Kabbalistic Mapping Not Implementable  
- **Section affected**: §3.1 שכבות = ספירות  
- **Severity**: important  
- **Confidence**: 85  
- **Claim**: Sefirot stages (Keter→Malkhut) lack concrete Rust module mappings.  
- **Proposed patch**: Add to §3.1:  
```markdown
| **חסד** (Chesed) | `spreading_activation::spread()` |
| **גבורה** (Gevurah) | `pruning::apply_threshold(0.6)` |
```  
- **Hidden assumption**: 10-stage pipeline is optimal.  
- **Self-objection**: Hardcoding limits future refactoring.  
- **Validation test**: Code audit verifying each Sefira has exactly 1 Rust module.  

## Top 3 Strengths  
1. **Atom encoding** (8-byte bitpacked) enables O(1) ops and fits 10M atoms in 80MB - revolutionary for low-mem AGI.  
2. **Hebrew-first semantics** with root triples and gematria provides linguistically grounded reasoning.  
3. **Deterministic walks** with explicit interference mechanics enables zero-hallucination guarantees.  

## Open Question for Iter 2-7  
**How to achieve 1B edge scalability within 6GB RAM?** Focus on VarInt encoding, delta compression, and tiered edge storage.  

## Final Score  
**7/10** - Revolutionary architecture with critical memory budget gaps for 6GB devices.