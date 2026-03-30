---
description: "Build orchestrator prompt files from task descriptions"
agent: orchestrator/builder
---

## User Input

```text
$ARGUMENTS
```

Use the input as either:
- a directory containing markdown task files
- one or more markdown task files

If empty, use the current working directory.

## Hard Rules
- Start from the task description files. Do not invent tasks.
- Do deep discovery before writing prompts.
- Preserve task intent. Do not silently merge, split, drop, or reorder tasks in ways that change that intent.
- If discovery finds a real blocker or a required reshape, stop and explain it instead of rewriting the task set.
- Write machine-ready prompts that define outcomes, constraints, and evidence. Do not write implementation plans.
- Every prompt must be standalone, include the context and file paths a fresh runner needs, and produce real code.
- Review written prompt-pack files, not an in-memory draft.
- `RULES_DIR`: `/home/sewer/nixos/users/sewer/home-manager/programs/opencode/config/rules`
- Apply the shaping rules in this file and these shared rules relative to `RULES_DIR`:
  - `general.md`
  - `documentation.md`
  - `performance.md`
  - `testing.md`
  - `test-parameterization.md`
  - `code-placement.md`
- The shared rule files are the source of truth for reusable code-writing guidance.

## Workflow

### Phase 1: Load task inputs
- Resolve the input path or use the current working directory.
- Accept one or more markdown task files.
- If the input is a directory, discover task files inside it.
- When present, prefer `PROMPT-SPLIT.md` as the overview and `PROMPT-DRAFT-*.md` as per-task inputs.
- If specific files are provided, use those files directly.
- Require at least one actionable task file.
- Use the inputs to determine task order, dependencies, scope notes, clarifications, and the source document.
- If files disagree, keep the most concrete task description and carry the resolved interpretation into `# Clarifications`.
- Determine the source document path:
  - If any input file contains a valid `Source Document:` path, use it.
  - Otherwise, use the most relevant input file.

### Phase 2: Do discovery
- Read every described task, cited source, and referenced artifact before writing prompts.
- Confirm the work is needed:
  - update or sync: compare current and requested state; skip if identical
  - add or create: confirm it does not already exist
  - fix: confirm the bug is real
  - migration: compare current and target state; skip if already compliant
- Reuse existing artifacts verbatim when they exist: schemas, types, tables, precedence rules, constants, signatures.
- Do not mention internal command names in generated prompts.
- Use subagents when needed:
  - `@codebase-explorer` for repo search and pattern discovery
  - `@mcp-search` for external libraries and APIs
- Default to parallel subagent calls unless one depends on another.
- Gather enough context that a runner with no prior memory can execute the prompt.
- Capture the minimum required files in `# Required Reads`, each with a short relevance note.
- Write prompt-scoped findings files as `PROMPT-FINDING-<prompt-stem>-NN.md` and list them in `# Findings`.
- Keep findings prompt-scoped. Duplicate findings across prompts if that keeps prompts standalone.
- Put extra research in findings files, not extra sections in prompt files.
- Capture repo conventions in findings when they matter: test commands, lint/build expectations, and CI versions.

### Phase 3: Write draft `PROMPT-PRD-REQUIREMENTS.md`
- Create it in the current working directory.
- Extract every discrete requirement from the task files and the source document only.
- Use stable IDs: `REQ-001`, `REQ-002`, ...
- Tag each requirement as `IN`, `OUT`, or `POST_INIT`.
- Include the source section, a short acceptance note, and `Owner Prompt` for every requirement.
- Set `Owner Prompt` to a prompt file or `None`.
- Do not drop requirements; mark `OUT` or `POST_INIT` explicitly.

### Phase 4: Write draft `PROMPT-NN-{title}.md`
- Create one prompt per described task.
- Keep task titles unless discovery requires a clearer name.
- Keep task boundaries when they are viable.
- If a task must be split because of a real blocker, stop and ask the user first.
- Order prompts by dependency.
- Every prompt must include concrete deliverables and at least one code artifact.
- Carry relevant decisions, dependencies, and open questions into `# Clarifications`.
- Every `# Requirements` item must include a requirement ID.
- Add `# Settled Facts` with `FACT-###` source pointers.
- Add `# Verification Scope` with in-scope checks and known unrelated failures.

### Phase 5: Write draft `PROMPT-ORCHESTRATOR.md`
- Create it in the current working directory.
- Include:
  - the overall objective
  - the prompt list with dependencies
  - `PRD Path` relative to the working directory
  - `Requirements Inventory: PROMPT-PRD-REQUIREMENTS.md`
  - `## Requirement Ownership` entries in this format:
    - `REQ-### - Owner: PROMPT-NN-... - Secondary: ... | None`
- Every `IN` requirement must have exactly one owner.
- In `PROMPT-ORCHESTRATOR.md`, keep ownership mapping only in `## Requirement Ownership`.

### Phase 6: Review draft prompt pack
- Spawn `@orchestrator/prompt-pack-reviewer` with:
  - `requirements_path`: absolute path to `PROMPT-PRD-REQUIREMENTS.md`
  - `orchestrator_path`: absolute path to `PROMPT-ORCHESTRATOR.md`
  - `source_paths`: absolute paths to the original task files, the source document, and any `PROMPT-SPLIT.md` or `PROMPT-DRAFT-*.md` inputs that shaped the pack
  - `original_context`: the original user request text, or a short summary when the raw text is not available
- If the reviewer returns `BLOCKING`, revise the draft prompt pack and review again.
- Proceed only when the reviewer is `PASS` or `ADVISORY`.
- Max 5 review iterations.

### Phase 7: Run requirements preflight
- Run `@orchestrator/runner/requirements/requirements-preflight` with:
  - `requirements_path`: absolute path to `PROMPT-PRD-REQUIREMENTS.md`
  - `prompts_dir`: absolute path to the current working directory
  - `orchestrator_path`: absolute path to `PROMPT-ORCHESTRATOR.md`
  - `prd_path`: absolute path to the source document
- If preflight returns `FAIL` or `PARTIAL`, revise the draft prompt pack, then rerun Phase 6 before running preflight again.
- Proceed only on `PASS`.

### Phase 8: Hand off
```
Prompt pack ready.

Run `@ orchestrator` with `PROMPT-ORCHESTRATOR.md` to start execution.
For a single prompt, use `@ orchestrator/runner`.
```

## Prompt File Format: `PROMPT-NN-{title}.md`

````markdown
# Mission
[1-2 sentence goal]

# Objective
[What must be achieved]

# Context
[Background, decisions, and file paths needed for isolated execution]

# Required Reads
- path/to/file: [why this file matters]

# Requirements
- REQ-###: [specific, measurable requirement]

# Deliverables
- [concrete code artifacts]

# Constraints
- [technical constraints and things to avoid]
- No placeholder types or errors; define new ones only when used here; later prompts may extend.

# Success Criteria
- [observable outcomes]

# Scope
- IN: [what's in scope]
- OUT: [what's out of scope]

# Dependencies
None | depends on PROMPT-NN-...

# Clarifications
Q: <question>
A: <answer>

# Findings
- PROMPT-FINDING-<prompt-stem>-01.md: <one-line relevance>

# Settled Facts
- FACT-001: <validated fact used by this prompt> (Source: PROMPT-FINDING-... or path:line)

# Implementation Hints
- [existing patterns, APIs, or files to reuse]
- [guidance for planner/coder]

# Verification Scope
- In scope checks: <format/lint/build/tests relevant to this prompt>
- Out of scope known failures: <pre-existing unrelated failures if any, else None>
- Priority files: <files reviewers should prioritize>
````

## Orchestrator Index: `PROMPT-ORCHESTRATOR.md`

```markdown
# Orchestrator Index

Overall Objective: <short line>
PRD Path: <relative path to source document>
Requirements Inventory: PROMPT-PRD-REQUIREMENTS.md

## Prompts
- PROMPT-01-{title}.md - Objective: <short> - Dependencies: None
- PROMPT-02-{title}.md - Objective: <short> - Dependencies: PROMPT-01

## Requirement Ownership
- REQ-001 - Owner: PROMPT-01-{title}.md - Secondary: None
- REQ-002 - Owner: PROMPT-02-{title}.md - Secondary: PROMPT-03-{title}.md
```

## Requirements Inventory: `PROMPT-PRD-REQUIREMENTS.md`

```markdown
# PRD Requirements Inventory

Source PRD: <relative path to source document>

## REQ-001 [IN] <requirement>
- Source: <section>
- Acceptance: <evidence or outcome>
- Owner Prompt: PROMPT-01-<title>.md

## REQ-002 [POST_INIT] <requirement>
- Source: <section>
- Acceptance: <evidence or outcome>
- Owner Prompt: None
```

## Shaping Rules
- Respect the task boundaries from the input when they are viable.
- Keep extraction, migration, and verification together when they serve one objective.
- If a task spans subsystems or integrations, split only when a real blocker forces it, and ask the user first.
- If the request itself is a monolith-to-modular or modular-to-monolith migration, make that its own prompt before other work.

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
