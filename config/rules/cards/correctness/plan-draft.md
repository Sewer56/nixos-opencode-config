### Requirement fidelity
Every user requirement, constraint, and question from the original request needs at least one `[P#]` item or an explicit out-of-scope decision.

### Action appropriateness
`[P#]` actions must match the stated goal and not contradict user intent. Investigation-only requests should plan discovery or ask open questions, not implementation.

### File path validity
Paths in `**Files:**` lines and diff headers must exist or be plausible new targets within repo structure.
Block nonexistent paths unless the item gives a plausible creation rationale under the matching module.

### Illustrative snippets
Code snippets in draft items are illustrative, not binding implementation instructions.
Severity: ADVISORY unless exact speculative code blocks reviewer understanding.
Prefer short signatures or shapes over speculative full function bodies.
