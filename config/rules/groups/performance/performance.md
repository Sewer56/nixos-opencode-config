## Performance

Prefer the highest-performance correct implementation.
Simplify for readability, never at meaningful performance cost.

### Allocation-conscious authorship
Write allocation- and copy-conscious changed code by default:
- Borrow, move, or reuse instead of cloning/copying.
- Pre-size/reserve capacity when final size is known or cheaply bounded.
- Prefer single-pass/fused iteration over intermediate collections.
- Resolve lookups and parses once.
- Reuse buffers, handles, and connections across iterations.
- Batch or hoist per-item I/O, locks, or serialization out of loops.

Fix avoidable allocations, clones, and copies when an equally clear bounded alternative exists.
Never obfuscate for unmeasured wins.

### Bounded work and concurrency
Bound growing inputs through pagination, limits, batching, or streaming.
Avoid nested per-item database, network, or filesystem work on list paths.

Cap user-controlled workload before proportional allocation, sorting, logging, or spawning.
Judge from read target code, not plan wording.

Avoid unbounded fan-out, shared mutable state races, blocking async calls, and missing backpressure.

Never compute or sort discarded results when a bounded/top-N algorithm exists.
Select the top slice, then sort only that slice.

### Measurement availability
For unmeasured but provably bounded work, record the limitation, not a finding or `INCOMPLETE`.
Return `INCOMPLETE` only if local validation cannot run, the plan requires that measurement, or no bound is provable.
