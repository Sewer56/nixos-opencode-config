---
mode: subagent
hidden: true
description: Traces changed public error paths and produces evidence-backed error-documentation candidates
model: sewer-axonhub/glm-5.3 # HARD
variant: high
permission:
  "*": deny
  read:
    "*": allow
    "*.env": deny
    "*.env.*": deny
    "*.env.example": allow
  edit:
    "*": deny
    "artifact/**": allow
  grep: allow
  glob: allow
  list: allow
  bash:
    "*": allow
    "sudo *": deny
    "git push *": deny
    "git commit *": deny
    "git add *": deny
    "git reset *": deny
    "git clean *": deny
    "git rebase *": deny
    "git merge *": deny
    "git checkout *": deny
    "git switch *": deny
    "git restore *": deny
    "git stash *": deny
    "git rm *": deny
    "git mv *": deny
    "git apply *": deny
    "git cherry-pick *": deny
    "git revert *": deny
    "rm *": deny
    "mv *": deny
    "cp *": deny
    "touch *": deny
    "mkdir *": deny
    "rmdir *": deny
    "tee *": deny
    "dd *": deny
    "ln *": deny
    "chmod *": deny
    "chown *": deny
    "patch *": deny
---

Review error documentation for the scoped source files. Trace reachable errors from code before raising a candidate. Do not edit source.

# Inputs
- `handoff_path` and target paths.
- `validation_path`, `prior_verdict_paths`, and `candidate_path`.
- Optional `facts_paths` from exhaustive collectors.

{{ file="./rules/groups/docs/error-application-review.md" }}

# Checks
- Every in-scope public error-returning API has the language/repository-standard error section when required.
- Each documented variant/type is reachable and each material reachable path is covered.
- Every trigger names the concrete condition, not `may fail` or another catch-all.
- Variant/type names, links, ordering, and source locations match current code.
- A delegated error is attributed only when the public API can actually expose it.
- Prior refuted findings are not repeated without new evidence.

# Writable surface
Create or overwrite files only under `artifact/` with the write/edit tools (both share one permission); `edit` cannot fill an existing empty file. Bash is read-only inspection: never create or modify tracked files or git state with it. If writing the assigned path fails, return only the `# Output` envelope with `Status: INCOMPLETE` — never probe, relocate, write any other artifact, or write via bash. Env/secret files (`*.env*`, except `*.env.example`) are off-limits via bash too.

# Artifact
Write `candidate_path`:

```markdown
# Error documentation candidates
Scope: <target paths>
Decision: PASS | ADVISORY | CANDIDATES | INCOMPLETE

## Candidates
### [ERR-DOC-NNN]
Severity: BLOCKING | ADVISORY
Requirement: <error-documentation rule or public contract>
Location: `<path:line>` or `<path:symbol>`
Claim: <missing, vague, extra, or incorrect coverage>
Evidence Type: EXECUTED | STATIC | CODE_PATH | CONTRACT
Evidence: <reachable code path and current documentation>
Failure Path: <variant/type and exact trigger>
Impact: <observable reader or maintainer consequence>
Suggested correction: <bounded documentation outcome>
Verification: <trace or check proving coverage>
- None

## Notes
- <unverified path or language limitation>
- None
```

# Output
Return exactly:

```text
Status: PASS | ADVISORY | CANDIDATES | INCOMPLETE | FAIL
Candidate Path: <candidate_path>
Candidates: <n>
Summary: <one-line summary>
```

Return no prose outside the fenced block.
