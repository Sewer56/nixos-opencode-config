### Requirement fidelity
Every user requirement and constraint maps to an `[AC-#]`, `[P#]`, `[D#]`, explicit invariant, or explicit out-of-scope decision.

### Observable acceptance
Acceptance criteria describe externally observable behavior, stable contracts, or executable checks. They do not merely restate files to edit or internal implementation steps.

### Action appropriateness
`[P#]` outcomes must advance the stated goal without contradicting decisions, invariants, or non-goals. Investigation-only requests plan discovery rather than silently becoming implementation work.

### Dependency integrity
Dependencies are acyclic. Contract/data/schema producers precede consumers unless tightly coupled work is deliberately kept in one cohort to preserve a valid repository state.

### Target plausibility
Named paths and symbols must be verified or explicitly described as plausible new targets under an existing module. Bounded discovery areas are allowed when exact placement is an implementation detail rather than a design decision.

### Risk and verification coverage
Changed behavior needs tests or an explicit reason tests are not applicable. Performance review is always routed; docs-only items record the skip reason. Material security, migration, compatibility, and user-documentation risks and concrete workload-scale performance risks must be routed and reflected in acceptance or verification.

### Appropriate detail
Plans contain approved behavior, contracts, dependencies, and evidence—not speculative patch mechanics. Exact code is advisory at best and blocking when it falsely constrains a valid implementation or quickly becomes stale.
