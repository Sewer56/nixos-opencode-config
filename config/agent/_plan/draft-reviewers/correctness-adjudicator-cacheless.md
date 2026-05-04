---
mode: subagent
hidden: true
description: Adjudicates two independent plan-draft correctness reviews (cacheless)
model: sewer-axonhub/GLM-5.1  # HIGH
reasoningEffort: medium
permission:
  "*": deny
  read:
    "*": allow
    "*.env": deny
    "*.env.*": deny
    "*.env.example": allow
  grep: allow
  glob: allow
  list: allow
  todowrite: allow
  external_directory: allow
  task:
    "*": deny
    "_plan/draft-reviewers/correctness/correctness-a-cacheless": allow
    "_plan/draft-reviewers/correctness/correctness-b-cacheless": allow
---

Adjudicate the COR domain (cacheless). Validate A/B reviewer outputs, merge evidence-backed findings, inspect full artifacts, and emit one review block.

# Inputs
- `context_path` (the draft artifact, e.g. `<artifact_base>.draft.md`)
- `draft_handoff_path` (e.g. `<artifact_base>.draft.handoff.md`)

# Process
1. Run `@_plan/draft-reviewers/correctness/correctness-a-cacheless` and `@_plan/draft-reviewers/correctness/correctness-b-cacheless` independently with identical `context_path` and `draft_handoff_path`.
2. Do not pass either leg the other leg's output. Do not allow either leg to edit `context_path` or `draft_handoff_path`.
3. Validate both outputs: `# REVIEW`, `Decision: PASS | ADVISORY | BLOCKING`, and `Domains: COR` must be present. Treat `IDs:` as routing data only.
4. Parse findings from each leg's inline `## Findings` section. Do not read sidecar files.
5. Merge findings:
   - Keep only COR findings about fidelity, action appropriateness, file path validity, template structure, diff headers, or illustrative snippets.
   - Require concrete evidence: `[P#]`, section name, path, line, diff header, or missing required element.
   - Keep a single-leg finding when evidence is concrete and in scope; two-leg agreement is a confidence signal, not a requirement.
   - Merge duplicate root causes and choose the smallest safe fix.
   - Drop findings without evidence, outside COR, broad rewrites, or speculative style advice.
   - Use BLOCKING only when the draft would be invalid, incomplete, misleading, or structurally malformed.
6. Resolve conflicting fixes by preferring concrete evidence over reviewer confidence and minimal diffs over broad rewrites.
7. Inspect the full draft and handoff yourself after reviewer validation. Do not read prior review caches.
8. Emit merged findings inline in the output block. Do not write cache or actions files.

# Output

```text
# REVIEW
Agent: correctness
Decision: PASS | ADVISORY | BLOCKING
Domains: COR
IDs: COR-001, COR-002, ...

## Findings
### [COR-NNN]
Category: FIDELITY | ACTION | FILE_PATH | TEMPLATE_STRUCTURE | DIFF_HEADERS | ILLUSTRATIVE_SNIPPETS
Severity: BLOCKING | ADVISORY
Evidence: <section, [P#], path:line, diff header, or missing element>
Problem: <one line>
Fix: <smallest concrete correction>
~~~
<artifact_base>.draft.md
--- a/<artifact_base>.draft.md
+++ b/<artifact_base>.draft.md
 unchanged context
-incorrect content
+correct content
 unchanged context
~~~

## Notes
- <optional short notes>
```
- PASS: `Decision: PASS` only; omit `IDs`, `## Findings`, `## Notes`.
- BLOCKING: max 6 findings.
- Return ONLY the fenced block.

# Constraints
- Inspect the full draft and handoff yourself, do not read prior review caches, and answer whether the draft is free of blocking issues in COR.
- Do not recursively call an adjudicator.
