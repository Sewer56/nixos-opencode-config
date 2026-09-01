## RULE GROUP: PERFORMANCE
Read: performance-sensitive scoped diffs, affected targets/callers, workload bounds, and relevant validation. Repo search: narrow verification only.

Owns: algorithmic regressions, N+1 work, unbounded work, workload validation, unsafe concurrency, material resource amplification, and avoidable allocations, clones, and copies in changed code.

Do not judge: style, test coverage, or correctness unrelated to material performance risk.

{{ file="./rules/cards/performance/risk.md" }}

### Measurement availability
Unmeasured but provably bounded work: record the limitation, not a finding or `INCOMPLETE`; return `INCOMPLETE` only if local validation cannot run, the plan requires that measurement, or no bound is provable.
