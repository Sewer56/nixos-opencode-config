### Highest-performance correct implementation
Prefer the highest-performance correct implementation; simplify for readability, but never trade meaningful performance for brevity or superficial simplicity.

### Allocation-conscious authorship
In changed code, write the allocation- and copy-conscious form by default:
- Borrow, move, or reuse over cloning or copying.
- Pre-size or reserve capacity when the final size is known or cheaply bounded.
- Single-pass or fused iteration over intermediate collections.
- Resolve lookups and parses once; never re-lookup or re-parse.
- Reuse buffers, handles, and connections across loop iterations.
- Batch or hoist per-item I/O, lock acquisition, or serialization out of loops.

An avoidable allocation, clone, or copy in changed code is fix-by-default when an equally clear bounded alternative exists; never obfuscate for unmeasured wins.

### Bounded and batched work
Avoid unbounded work on growing inputs: paginate, limit, batch, stream, or bound explicitly; avoid nested per-item database, network, or filesystem work on list paths.

Cap user-controlled workload size before proportional allocation, sorting, logging, or spawning. Judge from read target code, not plan wording.

### Unsafe concurrency
Avoid unbounded fan-out, shared mutable state races, blocking calls in async paths, and missing backpressure.

### Avoid discarded full work
Do not compute or sort results that will be discarded when a bounded/top-N algorithm is available.
Example: select the top slice, then sort only the kept slice.
