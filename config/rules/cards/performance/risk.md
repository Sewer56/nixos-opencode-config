### Highest-performance correct implementation
Prefer the highest-performance correct implementation. Then simplify for readability and reviewability, but never trade meaningful performance for brevity or superficial simplicity.

### Allocation-conscious authorship
In changed code, write the allocation- and copy-conscious form by default:
- Prefer borrowing, moving, or reusing an existing value over cloning or copying it.
- Pre-size or reserve capacity when the final size is known or cheaply bounded.
- Prefer single-pass or fused iteration over building intermediate collections.
- Resolve lookups and parses once; do not re-lookup or re-parse the same data.
- Reuse buffers, handles, and connections across loop iterations.
- Avoid per-item I/O, lock acquisition, or serialization inside loops; batch or hoist them.

An avoidable allocation, clone, or copy in changed code is a fix-by-default writer obligation when an equally clear bounded alternative exists. Never trade clarity for speculative micro-gains: no obfuscation for unmeasured wins.

### Bounded and batched work
Avoid unbounded work on growing inputs: add pagination, limits, early exits, batching, streaming, or explicit workload bounds. Avoid nested per-item database, network, filesystem, or expensive computation on list/batch paths. Validate or cap user-controlled workload size before allocating, sorting, logging, serializing, or spawning work proportional to it. Judge from read target code, not plan wording alone.

### Unsafe concurrency
Avoid unbounded fan-out, shared mutable state races, blocking calls in async paths, and missing backpressure.

### Avoid discarded full work
Do not compute or sort results that will be discarded when a bounded/top-N algorithm is available.
Example: select the top slice, then sort only the kept slice.
