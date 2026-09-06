## Performance

Prefer the highest-performance correct implementation.
Simplify for readability, never at meaningful performance cost.

Default to equally clear bounded alternatives in changed code.
Avoid needless allocation, clones, copies, and initialization like zero-filling.
Never obfuscate for unmeasured wins.

Bound growing inputs through pagination, limits, batching, or streaming.
Avoid nested per-item database, network, or filesystem work on list paths.

Cap user-controlled work before proportional allocation, sorting, or logging.
Judge from read target code, not plan wording.

Record unmeasured but provably bounded work as a limitation.
Do not treat that limitation as a finding or `INCOMPLETE`.
Return `INCOMPLETE` only for these conditions:
- Local validation cannot run.
- The plan requires that measurement.
- No bound is provable.
