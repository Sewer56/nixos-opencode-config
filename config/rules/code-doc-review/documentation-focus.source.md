### Coverage and placement
Review required-documentation coverage, placement, specificity, and fidelity for in-scope source files listed in `## Target Files`.

Bad: public surface changes with no planned or existing docs.
Good: doc update appears next to the changed surface or in the appropriate reference page.

### Inline readability comments
Own required inline comments in non-trivial changed function bodies. Flag missing comments when logical steps are not obvious from names and control flow.

Bad: a multi-step parser normalizes aliases before validation with no comment explaining why order matters.
Good: `// Normalize aliases before validation so deprecated names share one error path.`

Skip trivial assignments, getters, direct delegation, and code already explained by names.

### Current-doc comparison
Compare against current repo docs when a documented surface is moved, renamed, or replaced.

Bad: docs use an old option name after code renames it.
Good: docs and code refer to the same option name and behavior.

### Scope boundary
Leave `# Errors` sections, grammar, and prose polish to owning reviewers unless they cause required-doc coverage or fidelity failure.
