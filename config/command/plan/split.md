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

## Flow
1. Understand the request, constraints, and deliverables.
2. Inspect the repo for the files, boundaries, and patterns that matter.
3. Create the ordered task list.
4. Ask up to 10 questions in one batch only if the answers would materially improve the split.
5. Write `PROMPT-SPLIT.md` and `PROMPT-DRAFT-*.md` in the current working directory.
6. Remove stale `PROMPT-DRAFT-*.md` files in the same directory if they are no longer part of the current split.
7. Tell the user to review the drafts. They can be used directly or passed to downstream tooling such as `orchestrator/prompt-pack`.

## Guidelines
- Keep the split general-purpose.
- Split by deliverable or dependency, not by tiny implementation steps.
- If the request is already small, still produce exactly one draft.
- Use clear, stable titles.
- Put shared context in `PROMPT-SPLIT.md`; keep each draft task-specific.
- In `## Draft Plan`, include a small code snippet for a known interface, such as a new function signature, only when it clarifies the task.
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
- Do not add requirement inventories, acceptance matrices, or other machine-only scaffolding.
