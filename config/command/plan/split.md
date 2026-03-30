---
description: "Split a large task into smaller draft tasks"
agent: build
---

# Split Task

Split a large request into small, reviewable draft tasks grounded in the repo.

## User Input

```text
$ARGUMENTS
```

Use the user input as the source request.

## Workflow

### Phase 1: Understand
- Understand the request, constraints, and deliverables.
- Decide what discovery must confirm before splitting.

### Phase 2: Initial Discovery
- Always run at least one discovery subagent before drafting.
- Use `@codebase-explorer`, `@mcp-search`, or both, based on the request.
- Use discovery findings to identify the files, boundaries, patterns, dependencies, and split signals that matter.

### Phase 3: Write Draft Files
- Remove stale `PROMPT-DRAFT-*.md` files that are no longer part of the current split.
- Write draft `PROMPT-SPLIT.md` and draft `PROMPT-DRAFT-*.md` files in the current working directory so review sees only the current draft set.

### Phase 4: Review and Refine
- Pass the current `PROMPT-SPLIT.md` and all current `PROMPT-DRAFT-*.md` files to an independent reviewer subagent.
- Include relevant discovery findings when they help the review.
- If the reviewer finds meaningful gaps, overlap, weak boundaries, dependency errors, or poor alignment with the user's request, revise the draft files and review again.
- Repeat until the reviewer is satisfied or 3 review iterations have run. If concerns remain, record them in `## Open Questions` or `## Decisions`.

### Phase 5: Clarify Only When Needed
- Ask up to 10 questions in one batch only if the answers would materially improve the split.

### Phase 6: Handoff
- Ensure `PROMPT-SPLIT.md` and `PROMPT-DRAFT-*.md` reflect the latest reviewed draft.
- Tell the user to review the drafts. They can be used directly or passed to downstream tooling such as `orchestrator/prompt-pack`.

## Guidelines
- Keep the split general-purpose.
- Split by deliverable or dependency, not by tiny implementation steps.
- If the request is already small, still produce exactly one draft.
- Use clear, stable titles. Put shared context in `PROMPT-SPLIT.md`; keep each draft task-specific.
- Always do initial discovery with `@codebase-explorer`, `@mcp-search`, or both.
- Use `@codebase-explorer` for repo structure, boundaries, and local patterns.
- Use `@mcp-search` for external libraries, APIs, and docs.
- Always review written draft files, not an in-memory split.
- After drafting, always use an independent reviewer subagent to critique the split.
- Revise only for material findings.
- Review for completeness, overlap, boundaries, dependency ordering, and alignment with the user's request.
- Keep the review loop bounded. Do not exceed 3 review iterations.
- In `## Draft Plan`, include a small code snippet only when it clarifies the task.
- If a snippet needs explanation, add one short note.

## `PROMPT-SPLIT.md`

```markdown
# Task Split

Overall Goal: <short line>
Source Document: <path if one exists, else `user input`>

## Split Signals
- path/to/file-or-dir: <why it affected the split>
- <or `None`>

## Tasks
1. `PROMPT-DRAFT-01-<title>.md` - <short purpose>
2. `PROMPT-DRAFT-02-<title>.md` - <short purpose>

## Open Questions
- <question or `None`>

## Decisions
- <scope choice or `None`>
```

## `PROMPT-DRAFT-NN-{title}.md`

````markdown
# Draft Task

Title: <short task title>
Depends on: None | `PROMPT-DRAFT-0N-<title>.md`

## Objective
<plain-language outcome>

## Relevant Paths
- path/to/file-or-dir: <why it matters>

## Draft Plan
1. <step>
2. <step>

```language
<optional: known function signature, interface shape, schema fragment, or route contract>
```

Note: <one short note if needed>

## Scope Notes
- <task-specific constraint, decision, or open question>
- <or `None`>
````

## Constraints
- Number draft files sequentially.
- Ensure each meaningful area surfaced during inspection or review is owned by at least one draft task, or is called out as intentionally unresolved.
- Avoid overlapping draft ownership unless the dependency is explicit and necessary.
- Do not add requirement inventories, acceptance matrices, or other machine-only scaffolding.
