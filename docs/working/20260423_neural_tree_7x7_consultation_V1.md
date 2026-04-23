# Neural-tree mmap + 7×7 walks + persona search — AI Consultation

**Date:** 23.04.2026
**Context:** Idan's architectural proposal after seeing BPE fold takes 76s for 1.2MB.
Prompt asked for DIRECT critique of semantic-layout mmap, 7×7 parallel walks, and
persona-adaptive search parameters.

---

# gpt-4o (8.7s, 1848 tokens)

Your proposal for ZETS is ambitious and innovative, but there are several technical challenges and considerations that need to be addressed. Let's dive into each question:

### Q1: Semantic Layout in mmap (HNSW-style)
- **Feasibility**: HNSW is efficient for static or slowly changing datasets, but it struggles with dynamic updates due to the complexity of maintaining the hierarchical structure. As of 2025, dynamic semantic-ordered on-disk structures are still challenging. Techniques like dynamic graph embeddings or approximate nearest neighbor search with periodic re-indexing might be more suitable.
- **State-of-the-art**: Consider using hybrid approaches that combine HNSW with more flexible structures like graph neural networks (GNNs) that can adapt to changes more fluidly.

### Q2: 7×7 Parallel Walks
- **Comparison**: This approach resembles a beam search with a fixed beam width and depth, but with parallel exploration. It's not exactly MCTS, which involves probabilistic sampling and backpropagation of results.
- **Literature**: Beam search is well-documented in NLP and AI literature. Look into "Efficient Beam Search" techniques for further optimization.

### Q3: Async Cancellation Overhead
- **Overhead vs. Benefit**: The benefit of early termination is significant in large graphs where the solution is found quickly. However, the coordination overhead can be substantial, especially with 49 concurrent tasks. The payoff depends on the graph's size and the complexity of the query. Profiling and benchmarking are essential to determine the breakeven point.

### Q4: Persona-driven Search Parameters
- **Real Pattern**: Personalization is common in search engines, but typically involves user history and preferences rather than cognitive styles. The risk of filter bubbles is real and must be managed carefully.
- **Testing**: A/B testing and user feedback are crucial to validate that different parameters improve results for specific personas. Consider using metrics like precision, recall, and user satisfaction.

### Q5: ADHD/Autistic Cognitive Styles Metaphor
- **Grounding in Research**: While cognitive styles are recognized in psychology, applying them directly to search algorithms can be reductive and potentially stereotyping. It's important to ensure these metaphors are used as flexible guidelines rather than rigid categories.
- **Utility**: As an engineering abstraction, it can be useful if applied thoughtfully and inclusively.

### Q6: Realistic Implementation Path in Rust
- **Semantic Layout**: Look into Weaviate, Qdrant, and Milvus for inspiration. They often use vector embeddings and approximate nearest neighbor search.
- **Parallel Walks**: Use `tokio::spawn` for async tasks or `Rayon` for parallel iterators. Native threads can be used for heavy computation but may introduce more overhead.
- **Graph Mutation**: Consider using `RwLock` for read-heavy operations or MVCC for concurrent reads and writes. COW (Copy-On-Write) can be costly in terms of memory.

### Q7: 7×7 Intuition vs. Depth-8 Single-thread
- **Empirical Question**: Parallelism can outperform sequential search when the graph is large and queries are complex. The breakeven point depends on the hardware (CPU cores, memory bandwidth) and the graph's characteristics.
- **Benchmarking**: Conduct experiments to compare both approaches under various conditions.

### Q8: Missing Considerations
- **Feedback from Industry**: Lucene and Elasticsearch would emphasize the importance of indexing strategies and efficient query execution. Neo4j and TigerGraph might highlight the need for robust graph traversal algorithms and ACID compliance in graph databases.
- **Scalability and Fault Tolerance**: Ensure the system can handle large-scale data and recover gracefully from failures.
- **User Experience**: Consider the end-user experience and how the proposed changes impact usability and performance.

Overall, your proposal is promising but requires careful consideration of the technical challenges and potential pitfalls. Thorough testing, benchmarking, and user feedback will be crucial to its success.

---

# gemini-2.5-flash (35.3s, 7100 tokens)

Idan, this is a bold proposal, and it's good you're seeking a strong technical critique. Let's dive in.

## Critique of Idan's Proposal: Neural-tree mmap + 7×7 parallel walks + persona-adaptive search

### Q1. Is semantic layout in mmap (HNSW-style) FEASIBLE when the graph is constantly growing/mutating? HNSW has known issues with deletions and insertion reordering. What's the actual state-of-the-art in 2025 for dynamic semantic-ordered on-disk structures?

**Critique:** This is the most challenging part of your proposal. HNSW, while excellent for static or append-only vector indexes, is notoriously difficult to manage efficiently with frequent deletions and updates in-place.

*   **HNSW Mutation Challenges:**
    *   **Deletions:** "Lazy deletion" (marking nodes as deleted but not removing them from the graph) is common, but it bloats the index over time and degrades search performance as traversals hit dead ends. True deletion requires rebuilding parts of the graph or complex re-linking, which is expensive.
    *   **Insertions:** While HNSW supports insertions, maintaining optimal graph quality (short paths, high recall) requires careful selection of neighbors. Random insertions can degrade graph structure over time, leading to suboptimal search. Rebuilding the graph periodically is often necessary.
    *   **Reordering:** Reordering nodes in an HNSW graph is fundamentally a graph restructuring operation, not a simple array reordering. Moving a node means updating all its neighbors' adjacency lists, which is not cache-friendly or efficient for mmap-backed structures without significant overhead.

*   **State-of-the-Art for Dynamic Semantic-Ordered On-Disk Structures (2025 context):**
    *   **Vector Databases (Weaviate, Qdrant, Milvus):** These systems *do* use HNSW or similar structures (e.g., DiskANN, Vamana) for their core vector indexing. However, they typically handle mutations in specific ways:
        *   **Append-only segments:** Often, new data is written to new segments, and old segments are merged or compacted in the background. This is a common pattern for LSM-tree-like structures. Applying this to HNSW means building new HNSW indexes for new data and then merging them, which is non-trivial.
        *   **Memory-first, disk-backed:** Many keep the primary HNSW graph in memory for fast access, and use disk for persistence and larger-than-RAM datasets. When disk is involved, they often use memory-mapped files for the *vector data* and *adjacency lists*, but the *graph structure itself* (pointers, neighbor lists) is often managed in a way that allows for more dynamic updates, potentially with a separate indexing layer.
        *   **Batch updates/rebuilds:** For significant changes, they might rebuild parts of the index or the entire index in the background.
    *   **DiskANN/Vamana:** These are designed with disk access patterns in mind, often using techniques like "greedy graph traversal" and "robust neighbor selection" to minimize random disk I/O. They still face similar challenges with dynamic updates as HNSW.
    *   **LSM-Tree + HNSW:** A promising hybrid approach involves using an LSM-tree for recent updates and a more static HNSW index for older, stable data. New data is inserted into the LSM-tree, and periodically flushed/merged into the HNSW index. This adds complexity but offers better mutation performance.
    *   **Persistent Memory (PMEM/NVMe):** With the rise of PMEM, some research explores structures that can be modified in-place with transactional guarantees, but this is still bleeding edge for large-scale graph indexes.

*   **Feasibility for ZETS:**
    *   **Direct mmap of HNSW nodes for O(1) lookup + O(log n) navigation:** This is the core tension. If "adjacent positions in mmap = semantically close," then insertions/deletions *must* reorder the mmap. This is extremely expensive for a large mmap file. You'd need a sophisticated layer on top of mmap that provides logical contiguity while physical storage is fragmented, or accept significant performance degradation.
    *   **Alternative:** You could mmap the *vector data* and *adjacency lists* for HNSW, but the HNSW *graph structure itself* (the pointers/indices) would need to be managed by a more dynamic data structure (e.g., a B-tree or hash map for node IDs to mmap offsets) to handle mutations without full rebuilds. This loses the "adjacent positions = semantically close" property for the *nodes themselves*, only for their *vector representations*.
    *   **Conclusion:** Directly mapping a dynamically mutating HNSW graph to contiguous mmap regions for semantic proximity is **highly problematic and likely infeasible** without significant compromises or a complete re-think of the underlying storage model. You'd likely end up with something closer to a vector database's approach: mmap for raw data blocks, but a separate, more dynamic index structure managing the graph topology.

### Q2. The 7×7 parallel walk idea — is this equivalent to beam search? MCTS (Monte Carlo Tree Search)? Something new? Cite real literature.

**Critique:** This sounds very much like a variant of **Beam Search**, with some characteristics of **Limited Discrepancy Search (LDS)** or **Iterative Deepening Depth-First Search (IDDFS)** if the "retry waves" imply increasing beam width or depth. It is *not* MCTS.

*   **Beam Search:**
    *   **Definition:** Beam search is a heuristic search algorithm that explores a graph by expanding the most promising nodes in a limited set. It's an optimization of best-first search that reduces its memory requirements. At each level of the search tree, it generates all successors of the states at the current level, sorts them by heuristic value, and selects only the *k* best states (the "beam width") to proceed to the next level.
    *   **Your proposal:** "At any node: explore 7 neighbors in parallel (async)" and "cap SEARCH at depth 7, but with beam-width 7" perfectly aligns with beam search. You're exploring 7 paths concurrently, effectively maintaining a "beam" of 7 active search frontiers.
    *   **Literature:** Russell & Norvig, *Artificial Intelligence: A Modern Approach*, Chapter 3. Many papers on sequence generation (NLP, speech recognition) use beam search extensively.

*   **Not MCTS:**
    *   **MCTS** involves building a search tree by simulating random playouts (Monte Carlo simulations) from nodes to estimate their value, and then using these estimates to guide future expansions (e.g., UCT algorithm). Your proposal doesn't mention simulations or random playouts.

*   **Potential Nuances:**
    *   **"Deep think" mode (10 parallel 7×7 searches):** This sounds like running 10 *independent* beam searches concurrently, potentially with different starting points or slightly perturbed heuristics, to increase coverage. This is a common ensemble technique.
    *   **"3 retry waves of 7 walks":** This could imply increasing the beam width or depth in subsequent waves if the initial attempts fail, which is reminiscent of Iterative Deepening or Limited Discrepancy Search, where you systematically explore more options if simpler ones fail.

*   **Conclusion:** This is a **parallelized Beam Search**. It's a well-established and effective heuristic search strategy, especially when the search space is too large for exhaustive search but a good heuristic is available.

### Q3. Async cancellation of 49 concurrent walks when one finds answer — overhead of coordination (channel sends, atomic stops) vs benefit of early termination. At what query complexity does this pay off vs just doing sequential 1-walk-at-a-time?

**Critique:** This is a classic trade-off in parallel computing.

*   **Overhead:**
    *   **Coordination:** Yes, there will be overhead. `tokio::select!` or similar mechanisms for waiting on multiple futures and cancelling others involve atomic operations, channel sends/receives, and scheduler overhead. Each `tokio::task` has a memory footprint.
    *   **Context Switching:** With 49 concurrent tasks (or 4900 in "deep think"), the Rust async runtime (Tokio, async-std) will be doing a lot of context switching, especially if many tasks are ready to run. This can become a bottleneck if the individual "walk" steps are very short.
    *   **False Sharing:** If your walks frequently access and modify shared data structures (e.g., a visited set), cache coherence protocols can introduce significant overhead due to false sharing between CPU cores.

*   **Benefit of Early Termination:**
    *   **Best-case scenario:** If the "answer" is found very early in one of the parallel walks, the benefit is huge. You avoid the work of the other 48 walks.
    *   **Worst-case scenario:** If the answer is found late, or if all walks explore similar paths, the overhead might outweigh the benefit.

*   **Payoff Point:**
    *   **Query Complexity:** This pays off when:
        1.  **The search space is large and the answer can be found quickly by *some* path.** If the answer is always deep and requires exploring many paths, the benefit diminishes.
        2.  **The cost of a single "step" in a walk is significant.** If each step involves disk I/O (mmap access) or complex computation, then the overhead of coordination is amortized over more expensive work. If steps are trivial CPU operations, the overhead becomes more noticeable.
        3.  **The "answer" is well-defined and can be identified quickly.** If "finding an answer" requires further processing or aggregation, the early termination benefit is reduced.
        4.  **The underlying hardware has sufficient parallelism (CPU cores).** Running 49 tasks on 4 cores will lead to significant context switching and less true parallelism.
    *   **Sequential vs. Parallel:**
        *   **Sequential (1-walk-at-a-time):** Simpler, no coordination overhead. Good if answers are usually found quickly by a single, well-guided path, or if the graph traversal is CPU-bound and you want to avoid context switching.
        *   **Parallel (7x7):** Better for "needle in a haystack" scenarios where the answer might be on any of several promising paths, or when the graph traversal is I/O-bound (waiting on mmap reads), allowing other tasks to run.
    *   **Breakeven:** The breakeven point is highly dependent on:
        *   Average path length to an answer.
        *   Variance in path lengths to an answer.
        *   Cost of a single graph traversal step (CPU, I/O).
        *   Number of available CPU cores.
        *   Efficiency of the async runtime's scheduler and cancellation mechanism.

*   **Recommendation:** This needs **profiling and empirical testing**. Start with a simpler model (e.g., 7 parallel walks, not 49) and measure the overhead. Use tools like `perf` and Rust's `tracing` crate to understand where time is spent.

### Q4. Persona-driven search parameters:
   - Is this a real pattern in production search engines? (Google personalized results, but very different)
   - Risk: different personas getting different "truth"? Filter bubbles?
   - How do you test that "creative scientist" beam=12 actually gives better results than beam=7 for that persona?

**Critique:** This is a fascinating and potentially powerful idea, but comes with significant challenges.

*   **Real Pattern in Production Search Engines?**
    *   **Yes, but not explicitly as "personas" with hardcoded search parameters.** Personalized search *is* a major feature. Google, Amazon, Netflix, etc., all personalize results based on user history, location, device, implicit intent, etc.
    *   **How it's done:** This is typically achieved through:
        *   **Ranking models:** Machine learning models that re-rank results based on user features.
        *   **Query expansion/rewriting:** Modifying the query itself based on user context.
        *   **Content filtering:** Showing/hiding certain types of content.
        *   **Implicit feedback loops:** Users clicking on certain results reinforces preferences.
    *   **Your approach:** Explicitly setting `beam_width`, `depth`, `retry_policy` based on a `persona_id` is a more *declarative* and *rule-based* form of personalization. It's less common in large-scale general search engines because it's hard to maintain and generalize, but it's very common in *domain-specific* expert systems, recommender systems, or knowledge graphs where the "user intent" can be more clearly categorized.

*   **Risk: Different "truth"? Filter bubbles?**
    *   **Absolutely, this is a major risk.** If "Dry precision" gets a narrow, high-confidence answer and "Medical patient" gets a broad, cautious, multi-path answer, they are effectively seeing different "truths" or at least different facets of it.
    *   **Filter bubbles:** This is the definition of a filter bubble. By tailoring the search strategy, you are inherently filtering the information presented.
    *   **Ethical implications:** For a knowledge graph, especially one dealing with sensitive topics (like Wikipedia articles), presenting different "truths" based on persona can be problematic. This needs careful consideration and transparency. How do you ensure neutrality or at least expose the underlying search strategy?

*   **How to Test Persona Effectiveness?**
    *   **This is the hardest part.** You can't just A/B test `beam=12` vs `beam=7` in isolation.
    *   **User Studies/Human Evaluation:** The gold standard.
        *   Define clear metrics for each persona:
            *   **"Dry precision":** High precision@1, low recall, fast response.
            *   **"Medical patient":** High recall, diverse evidence, emphasis on safety/caution, maybe lower precision but higher confidence in presented facts.
            *   **"Creative scientist":** High diversity, serendipity, novel connections, potentially lower immediate relevance but high "inspiration" score.
        *   Recruit users representing these personas.
        *   Give them tasks.
        *   Have them rate results based on the persona's goals.
    *   **Offline Metrics (Proxy Metrics):**
        *   **Diversity metrics:** For "creative scientist," measure the semantic diversity of the top-N results (e.g., using embedding similarity).
        *   **Recall/Precision:** For "dry precision," measure standard precision/recall against a gold standard.
        *   **Path length/number of paths:** For "medical patient," ensure multiple, distinct paths to evidence are found.
        *   **Time to first answer:** For "ADHD-like."
    *   **Simulations:** If you have a good understanding of what each persona *wants*, you can simulate queries and evaluate the search paths and results against those criteria.
    *   **Conclusion:** This requires a robust evaluation framework, likely involving human judgment and carefully designed proxy metrics. It's not a simple engineering task.

### Q5. The "ADHD/autistic cognitive styles" metaphor:
   - Is this grounded in real cognitive psychology research?
   - Are we stereotyping in a problematic way?
   - Or is it a useful engineering abstraction?

**Critique:** This is a very sensitive area, and you need to tread extremely carefully.

*   **Grounded in Research?**
    *   **Partially, but highly generalized and simplified.** There is research into cognitive styles associated with neurodiversity (e.g., "broad attention" in ADHD, "systemizing" in autism). However, these are complex spectra, not monolithic "styles."
    *   **Example:** "Autistic-like deep focus" (narrow, very deep) might resonate with some aspects of autistic information processing (intense focus on specific interests, detailed processing). "ADHD-like broad divergent" (broad, shallow) might resonate with aspects of ADHD (difficulty sustaining attention, seeking novelty, rapid context switching).
    *   **However, these are vast oversimplifications.** Neurodiverse individuals have immense variability in their cognitive profiles. Reducing them to a few search parameters risks misrepresenting their experiences.

*   **Stereotyping in a Problematic Way?**
    *   **Yes, there is a significant risk of stereotyping and potentially causing offense.**
    *   **Why problematic:**
        1.  **Oversimplification:** You're reducing complex human cognitive profiles to a few search parameters. This ignores the vast diversity within neurodiverse communities.
        2.  **Pathologizing:** Using "ADHD-like" or "Autistic-like" as engineering abstractions can inadvertently reinforce stereotypes or imply that these are "deficiencies" that need a specific search "fix," rather than valid and diverse ways of processing information.
        3.  **Lack of Agency:** It implies that the system *knows* how someone with ADHD or autism *should* search, rather than empowering the individual to choose their own search strategy.
        4.  **Misattribution:** A user might have a "broad divergent" search need without having ADHD, or a "deep focus" need without being autistic. Attaching these labels can be misleading and unhelpful.

*   **Useful Engineering Abstraction?**
    *   **Potentially, but rename it immediately.** The *underlying cognitive needs* (e.g., "explore broadly and quickly," "focus deeply on a narrow topic," "seek diverse perspectives") are absolutely useful engineering abstractions.
    *   **Recommendation:** **Do NOT use diagnostic labels (ADHD, Autistic) for engineering abstractions.** Instead, abstract to the *desired search behavior* or *information processing style*.
        *   Instead of "ADHD-like," use "Broad Exploratory," "Rapid Iteration," or "Divergent Thinking."
        *   Instead of "Autistic-like deep focus," use "Deep Dive," "Focused Analysis," or "Exhaustive Detail."
        *   Instead of "Creative Scientist," use "Serendipitous Discovery" or "Interdisciplinary Exploration."
    *   This removes the problematic stereotyping while retaining the engineering utility of tailoring search strategies to specific user *needs* or *goals*.

### Q6. Realistic implementation path in Rust:
   - mmap with semantic layout: Weaviate, Qdrant, Milvus all have this. What approach do they use?
   - Parallel walks: tokio::spawn? Rayon? async-std? Native threads?
   - Graph mutation while walking: RwLock vs COW vs MVCC?

**Critique:** Good questions, let's break down the Rust specifics.

*   **mmap with semantic layout (HNSW):**
    *   **Approach of Vector DBs:** As discussed in Q1, they typically use mmap for the *raw vector data* and *adjacency lists*, but the *index structure itself* (the HNSW graph) is often managed in memory or with a more complex disk-backed structure that allows for dynamic updates.
    *   **Rust Implementation:**
        *   You'd use `memmap2` or `mmap-rs` for memory mapping.
        *   **Data Layout:** You'd define structs for your atoms and their associated vectors. These would be laid out contiguously in the mmap file.
        *   **HNSW Adjacency:** The adjacency lists (neighbor IDs) would also be stored in the mmap.
        *   **Dynamic Updates (The Hard Part):**
            *   **Append-only:** Easiest. New atoms are appended, new HNSW layers are built on top. Requires periodic compaction/merging.
            *   **In-place updates/deletions:** This is where it gets tricky. If you need to *reorder* or *remove* data in the mmap to maintain semantic contiguity, you're essentially rewriting parts of the file. This is slow and complex.
            *   **Alternative:** Use a separate B-tree or hash map (e.g., `sled`, `rocksdb` or a custom implementation) to map logical atom IDs to physical mmap offsets. This allows for dynamic updates but breaks the "adjacent in mmap = semantically close" property for *atoms themselves*, only for their vector representations.
            *   **Recommendation:** Start with an append-only model and lazy deletion. If true in-place mutation is critical, investigate LSM-tree-like compaction strategies or a hybrid approach with a separate index for dynamic data.

*   **Parallel Walks:**
    *   **`tokio::spawn` (or `async_std::task::spawn`):** This is the natural choice for I/O-bound tasks (like mmap reads) and for managing a large number of concurrent, cancellable operations.
        *   **Pros:** Excellent for concurrency, non-blocking I/O, efficient cancellation (`tokio::select!`).
        *   **Cons:** Requires `async`/`await` throughout your graph traversal logic. Can introduce overhead if tasks are purely CPU-bound and very short-lived.
    *   **`rayon`:** Excellent for CPU-bound parallelism (e.g., processing results, complex calculations on a single node's data).
        *   **Pros:** Very easy to use for data parallelism, low overhead for CPU-bound tasks.
        *   **Cons:** Not designed for I/O-bound concurrency or explicit task cancellation. You can't easily cancel a `rayon` task once it starts.
    *   **Native threads (`std::thread`):**
        *   **Pros:** Full control, no async runtime overhead.
        *   **Cons:** More complex to manage, harder to cancel gracefully, higher memory footprint per thread, less efficient for large numbers of concurrent I/O-bound tasks than an async runtime.
    *   **Recommendation:** For the "7x7 parallel walks" with async cancellation, **`tokio::spawn` is the most appropriate choice.** You'll be doing mmap reads (I/O), and the cancellation mechanism is built-in. You could potentially use `rayon` for *within-node* processing if a node's data requires heavy computation, but `tokio` for the overall graph traversal.

*   **Graph Mutation While Walking: RwLock vs COW vs MVCC?**
    *   **`RwLock` (Read-Write Lock):**
        *   **Pros:** Simple to implement for protecting shared data. Readers can proceed concurrently.
        *   **Cons:** Writers block all readers and other writers. Can become a bottleneck if mutations are frequent or long-running, especially with many concurrent readers (your 49 walks).
        *   **Applicability:** Good for protecting small, frequently read, infrequently written parts of the graph metadata (e.g., global counters, root node pointers). Not suitable for protecting the entire mmap-backed graph during traversal.
    *   **COW (Copy-on-Write):**
        *   **Pros:** Readers always see a consistent snapshot. Writers create a new version of the modified data.
        *   **Cons:** Can be memory-intensive if large parts of the graph are modified. Requires a mechanism to manage multiple versions and garbage collect old ones. Not ideal for in-place mmap modifications.
        *   **Applicability:** More suited for immutable data structures or when updates are infrequent and create entirely new versions.
    *   **MVCC (Multi-Version Concurrency Control):**
        *   **Pros:** Readers never block writers, and writers never block readers. Each transaction sees a consistent snapshot.
        *   **Cons:** Most complex to implement. Requires versioning of data, garbage collection of old versions, and careful transaction management.
        *   **Applicability:** This is the gold standard for highly concurrent databases (like PostgreSQL, RocksDB). If your graph is truly dynamic and needs high read/write concurrency, you'll eventually move towards MVCC principles.
    *   **Specific to mmap-backed HNSW:**
        *   Given the HNSW mutation challenges, you're likely looking at a combination:
            *   **Append-only mmap segments:** New data is written to new segments. Old segments are immutable.
            *   **Lazy deletion:** Nodes are marked as deleted in a separate, small, in-memory or `RwLock`-protected structure.
            *   **Periodic compaction/rebuild:** A background process rebuilds the HNSW index from active segments, removing deleted nodes. This is effectively a form of MVCC at a higher level (index versions).
            *   **Atomic updates for pointers:** If you have a global pointer to the "current" HNSW index, you can use `std::sync::atomic::AtomicPtr` to atomically swap to a new index version after a rebuild.
        *   **Recommendation:** For the graph *data* itself, aim for an append-only model with lazy deletion and background compaction. For *metadata* or small, frequently updated pointers, `RwLock` or atomics are appropriate. Full MVCC for the entire graph is a massive undertaking.

### Q7. BIGGEST ONE: is the 7×7 intuition actually BETTER than what we have (depth-8 single-thread)?
   - Empirical question: when does parallelism beat sequential search in graph traversal?
   - What's the breakeven point (query size, graph size, hardware)?

**Critique:** This is indeed the biggest question, and it's purely empirical. Your intuition is plausible, but not guaranteed.

*   **When does parallelism beat sequential search in graph traversal?**
    1.  **I/O-bound vs. CPU-bound:**
        *   **I/O-bound:** If fetching each atom from mmap (especially if it's not in cache) is the bottleneck, then parallelizing allows multiple I/O requests to be "in flight" concurrently. While one task waits for disk, another can make progress. This is where `tokio` shines.
        *   **CPU-bound:** If processing each atom (e.g., vector similarity calculation, complex predicate evaluation) is the bottleneck, then parallelism helps if you have enough CPU cores to run tasks truly in parallel. However, if the CPU work is very fine-grained, context switching overhead can eat into gains.
    2.  **Graph Structure:**
        *   **High Branching Factor, Shallow Answers:** If the graph has many outgoing edges from each node, and answers are often found relatively close to the start node, then a wide beam search (like 7x7) is likely to find an answer faster than a deep, narrow sequential search. You're "spreading out" to cover more ground quickly.
        *   **Low Branching Factor, Deep Answers:** If the graph is more like a long chain, and answers are very deep, then a deep sequential search might be more efficient. A wide beam might explore many dead ends before finding the deep path.
        *   **Multiple Paths to Answer:** If there are multiple distinct paths to an answer, parallelism increases the chance of finding *one* of them quickly.
    3.  **Heuristic Quality:**
        *   **Good Heuristic:** If your semantic proximity heuristic is very good at guiding the search towards the answer, a narrow,
