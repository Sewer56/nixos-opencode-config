### Requirement fidelity
Each user requirement, constraint, and question from the original request must be addressed by at least one `[P#]` item.
Bad: user asks for migration docs; no `[P#]` touches docs.
Good: `[P#]` explicitly owns migration docs or records why out of scope.

### Action appropriateness
`[P#]` item actions must match the stated goal and not contradict user intent.
Bad: user requests investigation-only plan; `[P#]` directs implementation.
Good: `[P#]` performs discovery or asks an open question.

### File path validity
Paths in `**Files:**` lines and diff headers must exist or be plausible new targets within repo structure.
Bad: `src/app/file.ts` in a repo with no `src/app` tree and no create rationale.
Good: existing path or plausible new file path under the matching module.

### Illustrative snippets
Code snippets in draft items are illustrative, not binding implementation instructions.
Severity: ADVISORY unless exact speculative code blocks reviewer understanding.
Bad: full exact function body for speculative implementation.
Good: short shape or signature plus intent.
