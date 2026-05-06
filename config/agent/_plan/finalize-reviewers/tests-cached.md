Review step test strategy. Initial review only — re-review handled by dedicated agent.

# Inputs

{{ file="./agent/_templates/review-inputs/plan-steps.txt" }}
- `cache_path`

# Focus

{{ file="./agent/_plan/finalize-reviewers/_templates/tests-shared-focus.txt" }}

## Read strategy
Read `handoff_path`, `plan_path`, all `step_paths`. Full audit. Write cache.

{{ file="./agent/_templates/review-read-strategy/source-access.txt" grounding_refs="step file diffs and handoff `## Settled Facts`" }}

{{ file="./agent/_templates/review-mission.txt" artifact_type="test artifacts" domain="test" }}

# Process

{{
  file="./agent/_templates/review-process/cached.txt"
  delta_source=handoff_path
  reads_review_ledger=1
  show_cache_format=1
  cache_format="# Review Cache: tests\n\n## Verified Observations\n- <step-id>: <what was verified, with grounding snapshot — one line each>\n\n## Findings\n\n### [TST-NNN]\nStatus: OPEN | RESOLVED\nCategory: COVERAGE | REDUNDANCY | PARAMETERIZATION | PLACEMENT\nSeverity: BLOCKING | ADVISORY\nProblem: <one line>\nFix: <unified diff targeting step file(s)>\n~~~diff\n<path/to/step/file>\n--- a/<path/to/step/file>\n+++ b/<path/to/step/file>\n  unchanged context\n--+proposed test step\n++corrected test step with proper coverage\n  unchanged context\n~~~\nResolution: <only for RESOLVED>"
  pointer_emit=1
}}

{{
  file="./agent/_templates/review-footer/cached.txt"
  agent="_plan/finalize-reviewers/tests-cached"
  domain=tests
  ref_type=step-id
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
  output_extra="- Your final output message MUST be EXACTLY the fenced block above. No other text — no analysis, no summary.\n- PASS block: `Decision: PASS` only. No IDs line.\n- Findings are written to cache only. The orchestrator reads `cache_path` for complete findings.\n- BLOCKING: max 6 findings. Cache findings in `cache_path`.\n- Verified observations MUST include grounding snapshots.\n- Write findings directly to cache. Do not re-narrate each step in reasoning."
}}
