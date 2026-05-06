Review step test strategy. Audit pass — reads all artifacts from scratch, does not read prior review caches.

# Inputs

{{ file="./agent/_templates/review-inputs/plan-steps.txt" }}

# Focus

{{ file="./agent/_plan/finalize-reviewers/_templates/tests-shared-focus.txt" }}

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
  file="./agent/_templates/review-output/pointer.txt"
  agent="_plan/finalize-reviewers/tests-cacheless"
  prefix=TST
}}
