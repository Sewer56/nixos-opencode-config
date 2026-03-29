---
description: "Generate a human review pack, then prompt packs for orchestrated execution"
agent: orchestrator/builder
---

## User Input

```text
$ARGUMENTS
```

You **MUST** consider the user input before proceeding (if not empty).

# Prompt Pack Generator

Generate a human review pack first, then prompt files for orchestrated execution after approval. Prompts define requirements and deliverables; implementation planning happens during orchestration.

think hard

## Core Principles
- Human-first review: let the user review a plain-language draft before deep discovery or machine-only artifacts are generated.
- Deliverable-first: each prompt must produce working code. No placeholder-only prompts.
- Isolation-safe: each prompt must run in isolation; include all required context and file paths.
- No speculative types/errors: define types/errors only when used in this prompt; later prompts may extend.
- Tests required: every prompt uses `basic` tests.
- Apply the canonical modularization rules in this file when shaping prompts.
- Consider shared rules:
  - `/home/sewer/nixos/users/sewer/home-manager/programs/opencode/config/rules/GENERAL-RULES.md`
  - `/home/sewer/nixos/users/sewer/home-manager/programs/opencode/config/rules/DOCUMENTATION-RULES.md`
  - `/home/sewer/nixos/users/sewer/home-manager/programs/opencode/config/rules/PERFORMANCE-RULES.md`
  - `/home/sewer/nixos/users/sewer/home-manager/programs/opencode/config/rules/TEST-PARAMETERIZATION-RULES.md`
  - `/home/sewer/nixos/users/sewer/home-manager/programs/opencode/config/rules/CODE-PLACEMENT-RULES.md`
- Prompts define outcomes and constraints, not the final implementation plan.
- `# Implementation Hints` and `# Module Layout` are guidance. A simpler valid approach may be used if it still satisfies requirements, clarifications, and settled facts without sacrificing performance.

## Simple Flow
1. Draft `PROMPT-REVIEW.md`.
2. Let the human review it, answer questions, and suggest edits.
3. On `go`, generate the machine prompt pack.
4. Run requirements preflight and write `PROMPT-ORCHESTRATOR.md`.

## Workflow

### Phase 1: Parse Request
- Extract core objective and components from user input only (ignore generator/system prompt)
- Draft prompt list (title + one-line objective)
- Order by dependencies
- Apply task sizing guidance (default to small, single-objective prompts)
- Convert broad goals into vertical slices that yield working code
- Produce a requirement-to-prompt ownership draft in memory only:
  - Each `IN` requirement must have exactly one primary owner prompt
  - Secondary prompts may reference a requirement but must not own it
- Keep this pass simple and human-reviewable; do not write machine-only artifacts yet

### Phase 2: Light Discovery
- Do a minimal repo scan only to avoid obviously bad prompt splits or misleading prompt titles
- Read only the most relevant entry points and patterns; do not do full prompt research yet
- Resolve only ambiguities that would make the review draft misleading
- Do not create findings files, prompt files, or the requirements inventory yet

### Phase 3: Create Human Review Pack
- Create `PROMPT-REVIEW.md` in the current working directory
- Write a plain-language draft for the human to review before any machine-heavy artifacts are generated
- Keep it free of requirement IDs, trace matrices, acceptance criteria jargon, revision tables, and other machine-only sections
- Include:
  - overall goal
  - proposed prompt list
  - for each prompt: what this does, why it is separate, draft plan, likely areas touched, needs first, open questions, done when, what we'll check

### Phase 4: Review Questions and Confirmation
- While waiting for confirmation, ask up to 10 questions total if answers would improve prompt quality
- Ask all questions in one batch using the `question` tool
- Prefer simple wording and concise options; include a recommended option when there is one
- Update `PROMPT-REVIEW.md` based on user feedback and answers
- Let the user edit, reorder, split, or merge prompts
- Continue to Phase 5 only when the user says `go`
- After `go`, do not ask more review questions unless full discovery uncovers a new hard blocker

When presenting the draft, use:
```
Review draft written to `PROMPT-REVIEW.md`.

Proposed prompts:
1. Prompt 01 - {title}: {objective}
2. Prompt 02 - {title}: {objective}
...

Questions: <count or none>

Say "go" to continue, or suggest changes.
```

### Phase 5: Full Discovery and Generate Machine Prompt Pack
- Review every item, source, and reference from the user input; do not skip
- Reuse existing research artifacts verbatim (schemas/types, tables, precedence rules, constants, signatures); do not invent
- If details are missing, say so succinctly; do not fabricate
- Do not mention the input file name/path in prompts or findings; use generic phrasing like "from input"
- Use subagents as needed:
  - `@mcp-search` for external library/API specifics
  - `@codebase-explorer` for codebase search and pattern discovery
- Default to parallel subagent calls; only serialize when a dependency requires it
- Treat findings as suggestions, not specs; use judgment when populating `# Implementation Hints`
- Prefer reusing existing types and patterns; only introduce new ones when required by the current prompt
- Gather enough context so a runner with no prior memory can execute the prompt
- Identify the minimal required files to read and capture them in `# Required Reads` with brief relevance notes
- Capture a short "repo conventions snapshot" in findings when relevant (test commands, CI action versions, lint/build expectations) to reduce avoidable downstream review churn
- Log findings per prompt in `PROMPT-FINDING-<prompt-stem>-NN.md` (comprehensive, prompt-relevant) and add a one-line entry in the prompt's `# Findings`
- Include other research discoveries the same way; keep findings prompt-scoped (duplication across prompts is OK)
- Put all supplemental artifacts in findings; do not add extra sections to prompt files
- Create `PROMPT-PRD-REQUIREMENTS.md` in the current working directory:
  - Extract every discrete requirement from the user input/PRD only (exclude generator/system/prompt-format requirements)
  - Use stable IDs: `REQ-001`, `REQ-002`, ... (zero-padded, sequential)
  - Tag each with scope: `IN`, `OUT`, or `POST_INIT`
  - Record source section from the user input/PRD (e.g., Key Goals, Features > Remapping & Bindings)
  - Write a short acceptance note per requirement (what evidence would satisfy it)
  - Do not drop requirements; mark out-of-scope or post-init explicitly
  - Prefer per-requirement headings to reduce tokens in inventory files
- Create in current working directory:
  - `PROMPT-NN-{title}.md` - one per task (standalone, self-contained)
  - Ensure each prompt includes concrete deliverables
  - Carry approved review decisions into `# Clarifications`
  - Every prompt `# Requirements` entry must include a requirement ID (e.g., `REQ-012: ...`)
  - Add `# Settled Facts` with validated facts and source pointers (`FACT-###`)
  - Add `# Verification Scope` to define in-scope checks and known unrelated pre-existing failures
  - Always include `# Module Layout` with language and a structure diff when layout changes
  - If unchanged, set `Structure: unchanged` and omit the diff block and `Why` line

### Phase 6: Validate Requirements Coverage (Subagent)
- Spawn `@orchestrator/requirements-preflight` with:
  - `requirements_path` (absolute path to `PROMPT-PRD-REQUIREMENTS.md`)
  - `prompts_dir` (absolute path to the current working directory)
  - `prd_path` (absolute path to the PRD input)
- If status is FAIL or PARTIAL: revise the prompt pack and re-run this phase
- If PASS: proceed

### Phase 7: Generate Orchestrator Index
Create `PROMPT-ORCHESTRATOR.md` in current working directory with:
- Overall objective
- Prompt list with dependencies and tests
- `PRD Path` and `Requirements Inventory` paths (relative)
- Add a `## Requirement Ownership` section:
  - `REQ-### — Owner: PROMPT-NN-... — Secondary: ... | None`
  - Every `IN` requirement must have exactly one owner
  - This section is the source of truth for requirement-to-prompt mapping

### Phase 8: Hand Off to User
```
Ready for orchestration with `@ orchestrator` (scheduler). For a single prompt, use `@ orchestrator/runner`.
```

## Review Pack Format: `PROMPT-REVIEW.md`

```markdown
# Prompt Review Pack

Overall Goal: <short line>

## Proposed Prompts

### Prompt 01: <title>
- What this does: <plain-language outcome>
- Why this is separate: <why this is its own step>
- Draft plan:
  1. <step>
  2. <step>
  3. <step>
- Likely areas touched:
  - path/to/file-or-dir
- Needs first: None | Prompt 0N
- Open questions:
  - <question or None>
- Done when:
  - <human-readable outcome>
- What we'll check:
  - basic

## Review Notes
- Status: waiting for feedback | approved
- Decisions:
  - <feedback, answer, or none>
```

## Prompt File Format: `PROMPT-NN-{title}.md`

````markdown
# Mission
[1-2 sentence goal for this task]

# Objective
[What must be achieved]

# Context
[Relevant background; include file paths and decisions for isolated execution]

# Required Reads
- path/to/file: [Why this file is relevant]

# Requirements
- REQ-###: [Specific, measurable requirement]
- REQ-###: [Expected behaviors and outcomes]

# Deliverables
- [Concrete code artifacts from this prompt]

# Constraints
- [Technical constraints]
- [What to avoid]
- No placeholder types/errors; define new ones only when used here; later prompts may extend
- `# Implementation Hints` and `# Module Layout` are guidance, not fixed implementation steps; a simpler valid implementation is allowed if requirements, clarifications, and settled facts still hold without sacrificing performance

# Success Criteria
- [How to know the objective is met]

# Scope
- IN: [what's in scope]
- OUT: [what's out of scope]

# Tests
basic

# Dependencies
None | depends on PROMPT-NN-...

# Clarifications
Q: <question>
A: <answer>

# Findings
- PROMPT-FINDING-<prompt-stem>-01.md: <one-line relevance>

# Settled Facts
- FACT-001: <validated fact used by this prompt> (Source: PROMPT-FINDING-... or path:line)

# Module Layout
- Language: <e.g., rust|typescript|python|csharp>
- Structure: unchanged | changed

If `changed`, show only the relevant structure delta:

```diff
src/config/
- types.rs
+ config_binding.rs
+ models/
+   binding_profile.rs
+   device_mapping.rs
```

- Why: <one-line rationale>

If `unchanged`, stop after `- Structure: unchanged`.

# Implementation Hints
- [Patterns, library usage, existing code to reuse]
- [Actionable guidance for planner/coder]

# Verification Scope
- In scope checks: <format/lint/build/tests relevant to this prompt>
- Out of scope known failures: <pre-existing unrelated failures if any, else None>
- Priority files: <files reviewers should prioritize>
````

## Orchestrator Index: `PROMPT-ORCHESTRATOR.md`

```markdown
# Orchestrator Index

Overall Objective: <short line>
PRD Path: <relative path to PRD input>
Requirements Inventory: PROMPT-PRD-REQUIREMENTS.md

## Prompts
- PROMPT-01-{title}.md — Objective: <short> — Dependencies: None
- PROMPT-02-{title}.md — Objective: <short> — Dependencies: PROMPT-01

## Requirement Ownership
- REQ-001 — Owner: PROMPT-01-{title}.md — Secondary: None
- REQ-002 — Owner: PROMPT-02-{title}.md — Secondary: PROMPT-03-{title}.md
```

## Requirements Inventory: `PROMPT-PRD-REQUIREMENTS.md`

```markdown
# PRD Requirements Inventory

Source PRD: PROMPT-PRD.md

## REQ-001 [IN] <requirement>
- Source: <section>
- Acceptance: <evidence or outcome>
- Owner Prompt: PROMPT-01-<title>.md

## REQ-002 [POST_INIT] <requirement>
- Source: <section>
- Acceptance: <evidence or outcome>
- Owner Prompt: None
```

## Investigation Rules
Before creating any prompt:
- **Update/sync tasks**: fetch and compare; skip if identical
- **Add/create tasks**: confirm it doesn't already exist
- **Fix tasks**: confirm the bug is real
- **Migration tasks**: compare current vs target; skip if compliant

## Canonical Modularization Rules
- Split catch-all files into focused modules/files with single responsibilities.
- Keep top-level orchestration logic in the parent module/file entrypoint.
- Place primarily data-holder models (with only trivial logic) in dedicated model files/folders by default.
- Keep enums/newtypes colocated with a parent type when they are only used by that parent.
- Keep non-public helper types local; do not widen visibility solely to move code.
- Keep conversion impls/functions (`From`/`TryFrom`/mappers) with related type definitions; avoid global `conversions` buckets.
- Co-locate tests with the module they validate; avoid central `tests.rs` files for unrelated modules.
- Prefer parameterised tests for repeated input/output cases rather than many
  near-identical tests (e.g. use `rstest` for Rust), with descriptive case
  names and labelled parameters/comments.
- Keep `models/mod.rs` for module wiring/re-exports; avoid accumulating concrete model definitions there.
- Apply these rules to new code and directly touched code.
- Do not require refactoring pre-existing monolithic code unless the user asks.
- Do not convert modular code into monolithic files unless the user asks.
- If a monolith-to-modular (or modular-to-monolith) migration is requested, plan it as a dedicated objective/prompt before any other requested changes.

## Constraints
- Be thorough; validate work is needed before creating prompts
- Do not omit requirements; mark as OUT or POST_INIT when no work is needed
- Order prompts by dependency
- Each prompt must be standalone and self-contained
- Every prompt must have code as a deliverable (no research-only prompts)
- Every `IN` requirement must have exactly one owner prompt in the index
- Do not duplicate requirement mappings in multiple index sections; keep mapping in `## Requirement Ownership`
- Do not place unresolved blockers into generated prompts; resolve or surface them during the review phase
- Apply the canonical modularization rules in this file
- Always include `# Module Layout`; use `Structure: unchanged` when there are no structural changes

## Task Sizing Guidance
- Default to the smallest useful unit; one primary objective per prompt
- Keep extraction/migration/verification in one prompt when they serve a single objective
- If a task spans subsystems or integrations, split into prompts ordered by dependency
- Split prompts only when objectives are independently shippable/testable
- Avoid cross-cutting refactors unless explicitly required by the user
- Prefer focused change sets, but allow touching multiple files when needed for coherent module boundaries
- Use descriptive, domain-first names for modules/files/types/functions
- If work combines unrelated objectives, split
- When unsure, err on more prompts with smaller scopes
- Aim for <=500 lines per prompt; split if likely to exceed

## Findings File Format: `PROMPT-FINDING-<prompt-stem>-NN.md`

```markdown
# Prompt Finding

Query: <what was searched or inspected>

## Summary
- <concise facts>

## Details
- <key API signatures, constraints, patterns>
- <verbatim artifacts from sources (schemas/types, tables, precedence rules, constants) as needed for planning>

## Relevant Paths
- path/to/file

## Links
- https://example.com/docs
```
