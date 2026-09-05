### Fidelity and acceptance
Trace every user requirement and constraint to an acceptance obligation, decision, invariant, or explicit non-goal across the whole bundle.
Acceptance requires observable behavior, stable contracts, or executable checks, not file lists or internal steps.
Map every obligation to cohort evidence without duplicate matrices.
Cohorts must advance the goal without contradicting approved decisions or exclusions.
Investigation-only requests plan discovery, not implementation.
Require acyclic dependencies and valid intermediate repository states.
Verify paths/symbols or mark plausible new targets under existing modules.
Bounded placement discovery is allowed only for implementation details, not unresolved design.

### Risk and readiness
Route `CORRECTNESS` and `QUALITY` always, with quality before every implementation commit.
Route `PERFORMANCE` always; only docs-only cohorts may record `NO` with a reason.
Route `TESTS` for changed observable behavior; justify inapplicable tests.
Route `SECURITY` for concrete trust, auth, secrets, IPC, untrusted input, filesystem/shell/SQL, serialization, crypto, permissions, or dependency risks.
Reflect material security, migration, compatibility, documentation, and workload-scale risks in acceptance or verification.
For changed/replaced/removed behavior that comments or docs might reference, unresolved public-API compatibility obligations are blocking questions.
Do not invent an old-behavior comment, external API contract, or answer to establish readiness.
Reject speculative patch mechanics that falsely constrain a valid implementation or become stale.
