### Coverage and placement
Review required-documentation coverage, placement, specificity, and fidelity for each I#/T# step. Public surface changes need planned docs in the relevant diff or snippet.

Bad: public surface changes with only a generic `update docs` note.
Good: the step diff adds or updates docs next to the changed surface or in the appropriate reference page.

### Inline readability comments
Own required inline comments inside non-trivial planned code-body diffs. Flag missing comments when logical steps are not obvious from names and control flow.

Bad: a parser diff adds normalization before validation with no comment explaining that boundary.
Good: the diff adds `// Normalize aliases before validation so deprecated names share one error path.` before the non-obvious block.

Skip trivial assignments, getters, direct delegation, and code already explained by names.

### Current-doc comparison
Compare against current repo docs when a documented surface is moved, renamed, or replaced.

Bad: planned docs use an old option name after code renames it.
Good: docs and code refer to the same option name and behavior.

### Scope boundary
Leave `# Errors` completeness to the errors reviewer. Leave implementation correctness to implementation reviewers. Do not review D# steps or end-user docs.
