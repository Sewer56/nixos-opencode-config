# Template Library

Canonical catalog of reusable fragments, `.txt` includes, and inline schema shapes used across `config/agent/`, `config/command/`, and `config/rules/`.

## How to Use

1. Include a fragment with `{{ file="./path/to/fragment.txt" key="value" }}`. The consumer defines runtime arguments; the fragment defines shape.
2. Fragments are not standalone prompts. Always pair with caller context (domain, input paths, validation rules) that is NOT in the fragment.
3. Fragments are grouped by **purpose**, not by file tree. Each group lists where the fragments live and which agents/commands consume them.
4. Keep fragments generic. Domain specifics (language, rule, artifact) are caller arguments.
5. Do not paste full fragment text into target prompts. Include with `{{ file=... }}`.

## Relationship to Rules

Rules in `config/rules/` also use `{{ file="..." }}` inclusion and share the same compositional pattern:
- `rules/groups/*/*.md` assemble related cards or targets into rule groups
- `rules/cards/*/*.md` define individual rule cards
- Both include each other via `{{ file="..." }}`
- Both are included from agents via `{{ file="./rules/..." }}`

Treat `config/rules/` templates as rule-fragment equivalents to prompt templates.

---

## 1. Review Lifecycle Fragments (`agent/_templates/`)

Shared across finalize, finalize-fast, implement, plan, docs, refactor, audit, audit-fast reviewers.

### 1.1 Review Mission
- **File:** `agent/_templates/review-mission.txt`
- **Shape:** One-line “Mission: Determine whether the `<artifact_type>` is free of blocking `<domain>` issues.”
- **Used by:** Every reviewer that owns a distinct domain.
- **Args:** `artifact_type`, `domain`

### 1.2 Review Process Scripts
- **File:** `agent/_templates/review-process/cached.txt`
- **Shape:** 6-step script: extract paths → read handoff + delta → select in-scope → inspect → write cache + actions → emit output.
- **Used by:** All cached reviewers (correctness, tests, performance, audit, docs, placement, wording, consistency, errors).
- **Conditional branches:** `has_cache_derivation`, `has_actions_path`, `reads_change_plan`, `has_inline_delta`, `reads_review_ledger`, `render_expanded`, `preserve_byte_exact`, `show_cache_update_detail`

- **File:** `agent/_templates/review-process/cacheless.txt`
- **Shape:** 3-step script: read inputs → inspect in-scope content → emit findings inline.
- **Used by:** All cacheless reviewers (initial pass, collector, final audit).
- **Conditional branches:** `read_context`, `reads_change_plan`, `single_file_pass`, `reads_review_ledger`, `reads_decisions`, `run_functional_validation`

### 1.3 Review Inputs Contracts
- **File:** `agent/_templates/review-inputs/plan-steps.txt`
- **Shape:** Lists standard input paths: `handoff_path`, `plan_path`, `step_paths`.
- **Used by:** finalize/performance/tests/audit/placement reviewers.

- **File:** `agent/_templates/review-inputs/refactor.txt`
- **Shape:** `handoff_path` for refactor workflows.
- **Used by:** refactor reviewers.

### 1.4 Read Strategy
- **File:** `agent/_templates/review-read-strategy/source-access.txt`
- **Shape:** Declares trust surface (“Do not attempt to read source file paths”).
- **Args:** `grounding_refs`
- **Used by:** finalize reviewers where source is not in the worktree.

### 1.5 Finding Format
- **File:** `agent/_templates/review-finding.txt`
- **Shape:** Single finding block: `[PREFIX-NNN]`, severity, optional file/lines/evidence/step, Problem, Fix, inline diff.
- **Args:** `prefix`, `categories`, `evidence`, `problem`, `fix`, `file_ref`, `bad`, `good`, `with_file`, `with_lines`, `with_evidence`, `with_detail`, `detail`, `with_step_file`, `step`
- **Used by:** output templates, footer template, action files, cache tables.

### 1.6 Cache Table
- **File:** `agent/_templates/review-cache-table.txt`
- **Shape:** `# Cache: domain` + Verified Observations + optional Audit/Findings Ledger.
- **Args:** `domain`, `has_actions_path`, `ref_type`, `prefix`, `with_audit_ledger`, `with_implement_cols`
- **Used by:** cached footer template, all cached reviewers.

### 1.7 Output Blocks
- **File:** `agent/_templates/review-output/output.txt`
- **Shape:** Full `# REVIEW` block with `Cache/Actions/Agent/Decision/Domains/IDs`, `## Findings` (1–2 finding blocks), `## Verified`, `## Notes`.
- **Args:** all finding args + `mode`, `agent`, `domains`, `with_domains`, `with_verified`, `verified_ref`, `return_rule_extra`
- **Used by:** cacheless reviewers and some cached reviewers that emit inline findings.

- **File:** `agent/_templates/review-output/pointer.txt`
- **Shape:** Compact pointer `# REVIEW` block with `Cache/Actions/Agent/Decision/Domains/IDs`. No `## Findings` inline.
- **Args:** `with_cache_path`, `with_actions_path`, `agent`, `domains`, `with_domains`, `prefix`, `return_rule_extra`
- **Used by:** cached adjudicators and re-reviewers.

- **File:** `agent/_templates/review-output/compact-output.txt`
- **Shape:** Minimal `# REVIEW` with `## Findings` as a bullet list (one-line per finding) + `## Verified` + `## Notes`.
- **Args:** `agent`, `prefix`, `finding_detail`, `verified_ref`
- **Used by:** rereview or lightweight review variants.

### 1.8 Review Footers
- **File:** `agent/_templates/review-footer/cached.txt`
- **Shape:** Optionally includes cache-table template + actions file format + pointer output block with return instructions.
- **Args:** all cache-table args + all pointer args + `skip_cache_format`, `has_actions_path`, `output_extra`, `agent`, `domains`, `prefix`
- **Used by:** every cached reviewer as the bottom of its prompt.

- **File:** `agent/_templates/review-footer/cacheless.txt`
- **Shape:** Includes `review-output/output.txt` with return instructions.
- **Args:** all output.txt args
- **Used by:** every cacheless reviewer as the bottom of its prompt.

---

## 2. Adjudication Stack (`agent/_templates/adjudicator/`)

Shared across OPT-016 adjudicated reviews for correctness and audit.

### 2.1 Adjudicator Script — Cached
- **File:** `agent/_templates/adjudicator/adjudicator-cached.txt`
- **Shape:** 18-step script: resolve paths → derive sidecars → run A/B legs → validate → merge → write parent cache + actions → emit pointer block.
- **Args:** `reviewer_a`, `reviewer_b`, `has_cache_derivation`, `cache_derivation`, `run_context`, `validation_extra`, `merge_scope`

### 2.2 Adjudicator Script — Cacheless
- **File:** `agent/_templates/adjudicator/adjudicator-cacheless.txt`
- **Shape:** 9-step script: run A/B → parse inline findings → merge → emit inline `# REVIEW` block.
- **Args:** `reviewer_a`, `reviewer_b`, `run_context`, `validation_extra`, `merge_scope`, `inspect_context`

### 2.3 Cache Contract Header
- **File:** `agent/_templates/adjudicator/cache-contract.txt`
- **Shape:** One-line contract reminder to preserve pointer/actions/cache contract for the domain.
- **Args:** `domain`

---

## 3. Explorer Fragments (`agent/_templates/explorer/`)

Used by codebase-explorer, draft explorer, and plan agents.

### 3.1 Explorer Output
- **File:** `agent/_templates/explorer/output.txt`
- **Shape:** `# Explorer Manifest` with Files Touched table, Key Symbols, Test Files, Observations, optional Open Questions.
- **Args:** `has_action`, `has_plan_ref`, `has_open_questions`, `row_example`

### 3.2 Explorer Constraints
- **File:** `agent/_templates/explorer/constraints.txt`
- **Shape:** One-liner “Read each file once. Output ≤80 lines.”
- **Args:** `constraint_extra`, `density_rule`

---

## 4. Domain-Specific Reviewer Templates

Nested under `{workflow}/reviewers/_templates/`.

### 4.1 Correctness
- **Location:** `agent/_plan/draft/reviewers/correctness/_templates/`
- **Files:** `body.txt`, `cache-format.txt`
- **Shape:** `body.txt` is a parameterized review body for correctness A/B legs. `cache-format.txt` defines the COR cache schema.

### 4.2 Performance
- **Location:** `agent/_plan/finalize/reviewers/_templates/performance-shared-focus.txt`
- **Shape:** Shared focus instructions across finalized performance reviewers.

### 4.3 Tests
- **Location:** `agent/_plan/finalize/reviewers/_templates/tests-shared-focus.txt`, `tests-cache-format.txt`
- **Shape:** `shared-focus` includes test strategy rules; `cache-format` defines test-specific cache columns.

### 4.4 Audit
- **Location:** `agent/_plan/finalize/reviewers/audit/_templates/body.txt`
- **Shape:** Parameterized audit review body for audit-a and audit-b legs.

### 4.5 Docs Reviewers
- **Location:** `agent/_docs/reviewers/_templates/`
- **Files:** `wording-header.txt`, `consistency-header.txt`, `shared-output.txt`
- **Shape:** Header fragments for wording/consistency reviewers; shared-output defines their output format.

### 4.6 Refactor Errors
- **Location:** `agent/_refactor/_templates/`
- **Files:** `errors-reviewer-header.txt`, `lang-rust-errors.txt`, `lang-typescript-errors.txt`
- **Shape:** Header for error-review legs; language-specific error taxonomies.

### 4.7 CodeDoc and EUDoc Reviewers
- **Location:** `agent/_plan/finalize/codedoc-reviewers/_templates/`, `eudoc-reviewers/_templates/`
- **Files:** `docs-readability-header.txt`, `docs-readability-output.txt`, `errors-header.txt`, `errors-output.txt`, `correctness-header.txt`
- **Shape:** Output schemas and headers for finalize-phase documentation review.

---

## 5. Language and Audit Templates

Under `agent/_audit/_templates/` and `agent/_refactor/_templates/lang-*`.

### 5.1 Public API Audit
- **Location:** `agent/_audit/_templates/`
- **Files:** `analysis-report.txt`, `lang-go.txt`, `lang-java.txt`, `lang-kotlin.txt`, `lang-python.txt`, `lang-rust.txt`, `lang-typescript.txt`
- **Shape:** `analysis-report.txt` defines the report schema for PROMPT-API-AUDIT.md. Language files define language-specific visibility and usage patterns.

### 5.2 Refactor Error Languages
- **Location:** `agent/_refactor/_templates/`
- **Files:** `lang-rust-errors.txt`, `lang-typescript-errors.txt`
- **Shape:** Taxonomy of error categories per language with severity guidance.

---

## 6. Inline Markdown Templates

Some agents define domain-specific schemas inline in their `.md` prompt files.

### 6.1 Code Generate Step and Handoff Templates
- **Location:** `agent/_plan/finalize/code-generate.md`
- **Shapes:**
  - Implementation Step (`I#`): path, action, anchor, lines, diff block, code shape, dependencies, evidence
  - Test Step (`T#`): similar shape with `Purpose`, `Covers: REQ-###`, `Parameterization`
  - Handoff: plan handoff with sections for mapping, trace matrix, review ledger placeholders

### 6.2 Command Templates
- **Location:** `command/` (e.g., `command/write/issue.md`)
- **Shape:** Built-in issue templates (bug, feature) and repo template detection logic.
- **Dominant pattern:** Template selection, not template ingestion.

---

## 7. Rule Fragments (`config/rules/`)

Rules use the same `{{ file="..." }}` inclusion system as agents.

- `rules/groups/*/*.md` — Assembled rule groups that include cards or other groups
- `rules/cards/*/*.md` — Single-card rules included by groups
- Agents include rule groups via `{{ file="./rules/groups/<category>/<target>.md" }}`

Key rule families consumed by templates above:
- `rules/groups/correctness/self-plan-draft.md`, `target-step-audit.md`
- `rules/groups/quality/*.md` (general, placement, minimum-visibility)
- `rules/groups/tests/target-test-strategy.md`, `target-test-parameterization.md`
- `rules/groups/docs/target-code-docs.md`, `target-error-docs.md`, `target-eudoc-correctness.md`, `self-draft-docs.md`
- `rules/groups/style/self-wording.md`, `self-readability.md`, `target-engagement.md`, `set-eudoc-polish.md`, `target-wording.md`, `target-readability.md`
- `rules/groups/performance/target-performance.md`
- `rules/groups/audit/search-public-api-analysis.md`

---

## Index by Type

| Fragment | Path | Consumers |
|---|---|---|
| review-mission | `agent/_templates/review-mission.txt` | All reviewers |
| review-process-cached | `agent/_templates/review-process/cached.txt` | Cached reviewers |
| review-process-cacheless | `agent/_templates/review-process/cacheless.txt` | Cacheless reviewers |
| review-finding | `agent/_templates/review-finding.txt` | All review outputs |
| review-cache-table | `agent/_templates/review-cache-table.txt` | Cached reviewers |
| review-output-full | `agent/_templates/review-output/output.txt` | Cacheless reviewers |
| review-output-pointer | `agent/_templates/review-output/pointer.txt` | Cached adjudicators |
| review-output-compact | `agent/_templates/review-output/compact-output.txt` | Re-reviewers |
| review-footer-cached | `agent/_templates/review-footer/cached.txt` | Cached reviewers |
| review-footer-cacheless | `agent/_templates/review-footer/cacheless.txt` | Cacheless reviewers |
| review-inputs-plan | `agent/_templates/review-inputs/plan-steps.txt` | Plan reviewers |
| review-inputs-refactor | `agent/_templates/review-inputs/refactor.txt` | Refactor reviewers |
| review-read-strategy | `agent/_templates/review-read-strategy/source-access.txt` | Sourceless reviewers |
| adjudicator-cached | `agent/_templates/adjudicator/adjudicator-cached.txt` | Correctness, audit cached |
| adjudicator-cacheless | `agent/_templates/adjudicator/adjudicator-cacheless.txt` | Correctness, audit cacheless |
| cache-contract | `agent/_templates/adjudicator/cache-contract.txt` | Cached adjudicators |
| explorer-output | `agent/_templates/explorer/output.txt` | Codebase-explorer, draft explorer |
| explorer-constraints | `agent/_templates/explorer/constraints.txt` | All explorers |
| tests-shared-focus | `agent/_plan/finalize/reviewers/_templates/tests-shared-focus.txt` | Tests reviewers |
| tests-cache-format | `agent/_plan/finalize/reviewers/_templates/tests-cache-format.txt` | Tests cached reviewers |
| performance-shared-focus | `agent/_plan/finalize/reviewers/_templates/performance-shared-focus.txt` | Performance reviewers |
| audit-body | `agent/_plan/finalize/reviewers/audit/_templates/body.txt` | Audit A/B legs |
| correctness-body | `agent/_plan/draft/reviewers/correctness/_templates/body.txt` | Correctness A/B legs |
| correctness-cache-format | `agent/_plan/draft/reviewers/correctness/_templates/cache-format.txt` | Correctness cached |
| wording-header | `agent/_docs/reviewers/_templates/wording-header.txt` | Docs wording reviewers |
| consistency-header | `agent/_docs/reviewers/_templates/consistency-header.txt` | Docs consistency reviewers |
| docs-output | `agent/_docs/reviewers/_templates/shared-output.txt` | Docs reviewers |
| errors-reviewer-header | `agent/_refactor/_templates/errors-reviewer-header.txt` | Refactor error reviewers |
| rust-error-taxonomy | `agent/_refactor/_templates/lang-rust-errors.txt` | Rust error reviews |
| typescript-error-taxonomy | `agent/_refactor/_templates/lang-typescript-errors.txt` | TypeScript error reviews |
| audit-analysis-report | `agent/_audit/_templates/analysis-report.txt` | Public API audit |
| audit-lang-go | `agent/_audit/_templates/lang-go.txt` | Go API audit |
| audit-lang-java | `agent/_audit/_templates/lang-java.txt` | Java API audit |
| audit-lang-kotlin | `agent/_audit/_templates/lang-kotlin.txt` | Kotlin API audit |
| audit-lang-python | `agent/_audit/_templates/lang-python.txt` | Python API audit |
| audit-lang-rust | `agent/_audit/_templates/lang-rust.txt` | Rust API audit |
| audit-lang-typescript | `agent/_audit/_templates/lang-typescript.txt` | TypeScript API audit |
| codedoc-readability-header | `agent/_plan/finalize/codedoc-reviewers/_templates/docs-readability-header.txt` | CodeDoc finalize |
| codedoc-readability-output | `agent/_plan/finalize/codedoc-reviewers/_templates/docs-readability-output.txt` | CodeDoc finalize |
| codedoc-errors-header | `agent/_plan/finalize/codedoc-reviewers/_templates/errors-header.txt` | CodeDoc finalize |
| codedoc-errors-output | `agent/_plan/finalize/codedoc-reviewers/_templates/errors-output.txt` | CodeDoc finalize |
| eudoc-correctness-header | `agent/_plan/finalize/eudoc-reviewers/_templates/correctness-header.txt` | EUDoc finalize |
