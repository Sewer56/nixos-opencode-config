---
mode: subagent
hidden: true
description: Adjudicates two independent plugin draft correctness reviews (cacheless)
model: sewer-axonhub/GLM-5.1 # HIGH
variant: medium
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
    "_plugin/draft-reviewers/correctness/correctness-a-cacheless": allow
    "_plugin/draft-reviewers/correctness/correctness-b-cacheless": allow
---

Adjudicate the plugin COR domain (cacheless). Validate A/B reviewer outputs, merge evidence-backed findings, inspect full artifacts, and emit one review block.

# Inputs
- `context_path`, `draft_handoff_path`

# Process

{{
  file="./agent/_templates/adjudicator/adjudicator-cacheless.txt"
  no_edit_targets="input artifacts"
  reviewer_a="_plugin/draft-reviewers/correctness/correctness-a-cacheless"
  reviewer_b="_plugin/draft-reviewers/correctness/correctness-b-cacheless"
  run_context="with identical artifact inputs"
  validation_extra=", `Domains: COR`"
  merge_scope="keep only COR findings in fidelity, action, template, diff-header, and plugin-constraint scope; require concrete evidence; keep single-leg findings when evidence is concrete and in scope; drop out-of-domain or unsupported findings"
  inspect_context="`context_path` and `draft_handoff_path`"
}}

# Output

{{
  file="./agent/_templates/review-output/output.txt"
  agent="_plugin/draft-reviewers/correctness"
  prefix=COR
  domains=COR
  categories="FIDELITY | ACTION | TEMPLATE_STRUCTURE | DIFF_HEADERS | PLUGIN_CONSTRAINTS"
  evidence="<section, [P#], path:line, diff header, or missing element>"
  problem="<one line>"
  fix="<smallest concrete correction>"
  file_ref="<artifact_base>.draft.md"
  bad="-incorrect content"
  good="+correct content"
  with_evidence=1
}}

# Constraints
- Inspect all artifacts yourself, do not read prior review caches, and answer whether the draft is free of blocking issues in COR.
- Do not recursively call an adjudicator.
