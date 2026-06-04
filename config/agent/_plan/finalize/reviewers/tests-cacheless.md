---
mode: subagent
hidden: true
description: Reviews step test strategy for finalized plans (cacheless)
model: sewer-axonhub/glm-5.1 # HIGH
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
---

Review step test strategy. Audit pass — reads all artifacts from scratch, does not read prior review caches.

# Inputs

{{ file="./agent/_templates/review-inputs/plan-steps.txt" }}

# Focus

{{ file="./agent/_plan/finalize/reviewers/_templates/tests-shared-focus.txt" }}

## Read strategy
Read `handoff_path`, `plan_path`, all `step_paths` in full.

{{ file="./agent/_templates/review-read-strategy/source-access.txt" grounding_refs="step file diffs and handoff `## Settled Facts`" }}

{{ file="./agent/_templates/review-mission.txt" artifact_type="test artifacts" domain="test" }}

# Process

{{
  file="./agent/_templates/review-process/cacheless.txt"
  read_context="Read `handoff_path` in full. Read `plan_path` in full. Read all `step_paths` in one batch. Only open repo test files for specific verification needs."
}}

# Output

{{
  file="./agent/_templates/review-output/output.txt"
  agent="_plan/finalize/reviewers/tests-cacheless"
  prefix=TST
  categories="COVERAGE | REDUNDANCY | PARAMETERIZATION | PLACEMENT"
  evidence="<step-id, section, path:line, or missing element>"
  problem="<one line>"
  fix="<unified diff targeting step file(s)>"
  file_ref="<path/to/step/file>"
  bad="--proposed test step"
  good="+corrected test step with proper coverage"
  with_lines=1
  with_evidence=1
  step=""
}}
