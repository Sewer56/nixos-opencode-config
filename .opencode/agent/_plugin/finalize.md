---
mode: primary
description: Converts a confirmed plugin plan into reviewed machine instructions
permission:
  "*": deny
  read:
    "*": allow
    "*.env": deny
    "*.env.*": deny
    "*.env.example": allow
  edit:
    "*": deny
    "*PROMPT-PLUGIN-PLAN.handoff.md": allow
    "*PROMPT-PLUGIN-PLAN.rev.*.md": allow
  todowrite: allow
  external_directory: allow
  glob: allow
  grep: allow
  list: allow
  task:
    "*": deny
    "codebase-explorer": allow
    "mcp-search": allow
    "_plugin/reviewers/errors": allow
    "_plugin/reviewers/reorder": allow
    "_plugin/reviewers/documentation": allow
    "_plugin/reviewers/correctness": allow
---

Convert a confirmed plugin plan into reviewed machine instructions.

# Inputs
- The latest user message may confirm the plan, provide finalize-time notes, or note changes since the draft.
- Required local artifact: `PROMPT-PLUGIN-PLAN.md`

# Artifacts
- `context_path`: `PROMPT-PLUGIN-PLAN.md`
- `handoff_path`: `PROMPT-PLUGIN-PLAN.handoff.md`
- `rev_pattern`: `PROMPT-PLUGIN-PLAN.rev.*.md`

# Process

## 1. Preconditions and source of truth
- Read `context_path` (`PROMPT-PLUGIN-PLAN.md`) as the source of truth, supplemented only by any explicit finalize-time notes from the latest user message.
- Treat the `/plugin/finalize` invocation as the confirmation boundary.

## 2. Deepen discovery only where needed
- Start from the paths and shapes already present in `context_path`.
- Consume `Overall Goal:` lines and `[P#]` labeled steps directly.
- `[P#]` items use free-form explanation + diff block. Extract file paths from diff block headers. Treat as draft-level guidance — ground REV diffs in actual file content.
- Deepen discovery only where the confirmed context leaves frontmatter fields, permission patterns, naming, cross-references, or output formats unresolved.
- Infer which optimization rules apply to each confirmed target from its behavior: review loop, subagent coordination, machine-readable output, or convention/artifact changes.
- Use `@codebase-explorer` for repo discovery first when needed.
- Use `@mcp-search` for external libraries or APIs only when needed.
- Read the files surfaced by discovery that matter to the machine artifact.

## 3. Write the handoff file
- Rewrite `handoff_path` from scratch for this run.
- Preserve the latest consolidated user request verbatim under `## Raw Request`.
- Write `handoff_path` using the `# Templates` section below.

## 4. Write the machine artifact
- Derive discrete `REV-###` items from the confirmed context and handoff.
- Stable numbering: number items sequentially from 001. If a REV is removed during revision, leave the gap — do not renumber other items.
- Write `handoff_path` using the `# Templates` section (handoff now includes Summary, Revision History, and REV Index).
- Write each REV item to its own file matching `rev_pattern` using the `# Templates` section.
- Apply only the relevant optimization rules to each target. Split rule fragments across the affected prompts and reviewers instead of copying the whole contract into every file.
- Embed operational rules directly in generated targets.

## Phase Structure
- Review loop structure: a core phase (correctness) runs first; after it passes, a polish phase (documentation/errors/reorder) runs with independent Delta tracking and iteration limits.

## 5. Run the review loop
Follow the ordered steps below exactly, in order.

### Core review
- Write and maintain `## Delta`: write to `handoff_path` before the first reviewer pass; record each `REV-###` item as a compact entry with `Status:`, `Touched:`, and `Why:` fields; add artifact markers for `Source Context` and `Review Ledger`; recompute after every material revision.
- Build core reviewer prompts: after each full machine-artifact draft, run `@_plugin/reviewers/correctness` in parallel. Treat each reviewer prompt as scoped call data. Include only: artifact paths (`context_path`, `handoff_path`), `rev_pattern` (a glob pattern matching REV target file paths to scope the review), Delta summary, current `### Decisions` excerpt when non-empty, finalize-time user notes. Omit: output format, focus lists, target file paths from REV items, role assignment, blanket read orders — reviewers decide what to open from Delta, cache state, and Decisions.
- Validate each reviewer response: confirm `# REVIEW` header, `Decision: PASS | ADVISORY | BLOCKING`, `## Findings` and `## Verified` headings. If malformed after retries, treat as BLOCKING with a synthetic finding.
- Retry malformed responses: if validation fails and Delta plus Decisions are unchanged, send only the protocol error and request re-emit; if Delta or Decisions changed, include only the new excerpt and request fresh response.
- Record decisions: update `### Decisions` in `handoff_path` for cross-domain arbitration only. Reviewers own issue tracking in their cache files. Core domain ownership: CORRECTNESS → correctness reviewer.
- Revise the machine artifact when findings require it: revise REV files only where needed; append one line to `## Revision History`.
- Re-run core reviewers after every material revision.
- Loop until no findings of any severity remain or 10 iterations.
  No findings: proceed to polish review. At cap: FAIL if BLOCKING, proceed to polish with risks if only ADVISORY.

### Polish review
- Update `## Delta` in `handoff_path`. Mark all core-reviewed items as Unchanged. Set `Why: core phase passed`.
- Build polish reviewer prompts: run `@_plugin/reviewers/documentation`, `@_plugin/reviewers/errors`, and `@_plugin/reviewers/reorder` in parallel. Include the same task-specific data as the core phase.
- Validate each reviewer response (same criteria as core).
- Retry malformed responses (same protocol as core).
- Record decisions: update `### Decisions` in `handoff_path` for cross-domain arbitration. Polish domain ownership: DOCUMENTATION → documentation reviewer; ERRORS → errors reviewer; REORDER → reorder reviewer.
- Apply polish reviewer diffs to REV files. Append one line to `## Revision History`.
- Re-run polish reviewers after every material revision.
- Loop until no findings of any severity remain or 10 iterations.
  No findings: SUCCESS. At cap: FAIL if BLOCKING, proceed to output with risks if only ADVISORY.

# Output

Return exactly:

```text
Status: SUCCESS | INCOMPLETE | FAIL
Context Path: <absolute path>
Handoff Path: <absolute path>
Rev Pattern: <e.g. PROMPT-PLUGIN-PLAN.rev.*.md>
Review Iterations: <n>
Summary: <one-line summary>
```

# Constraints

- Write only `PROMPT-PLUGIN-PLAN.handoff.md` and files matching `PROMPT-PLUGIN-PLAN.rev.*.md` during finalize.
- Modify only those files during finalize.
- Read `PROMPT-PLUGIN-PLAN.md` as source of truth only; write to handoff and machine paths.
- Keep each REV file diff-based with `Lines: ~` locators and context lines per `# Rules`. CREATE actions include full file content.
- Keep `PROMPT-PLUGIN-PLAN.handoff.md` factual and stable enough for the machine artifact and reviewers to use without rereading the whole conversation.

# Rules

Apply these rules when writing REV files:

- Write concrete values for every field and body — omit `...`, `TODO`, and comment-only stubs.
- Specify the full path for every file reference: REV headings, `Evidence` fields, and diff block targets all use fully qualified paths from the project root.
- Reference anchors and approximate location via `Lines: ~<start>-<end> | None` in the REV header.
- Insertions use normal code blocks with `Insert at: <anchor> (~start-end)`.
- Edits/removals use `diff` blocks; deletions include `Remove lines: ~start-end`.
- If frontmatter and content changes are contiguous, combine into a single diff block.
- If changes are scattered across a file, use multiple diff blocks within one REV item.
- CREATE actions include full file content in a normal code block (not a diff against empty).
- Diff blocks target markdown files — use markdown-aware line references (headings, list items, fenced code blocks).
- `Lines: ~` in the REV header indicates approximate location; include 2+ context lines before and after each change.

---

# Optimization Rules

Revisions produced by this finalize run must follow. Apply only the relevant rules below to each generated target and reviewer prompt:

- **Reviewer cache + Delta**: targets that run review loops or coordinate subagents include per-reviewer cache files and a Delta section in handoff so reviewers skip unchanged items on re-runs. Reviewers update only changed cache entries via targeted edits — preserve entries that are Unchanged and Verified unchanged.
- **Fixed output blocks**: use fenced code blocks with `text` language tag for plain structured output. ~~`json`/`yaml` tags for plain structured output~~ → `text` only.
- **No duplicated content**: reference information from other artifacts by section name or file path. ~~Re-quoting content already in another artifact~~ → reference by section name.
- **Shared ledger/file**: use a shared ledger or coordination file for orchestrator state when coordinating subagents. ~~Scattering coordination state across subagent outputs~~ → single shared file.
- **Concise human-facing docs**: include a short documentation update for humans when the iteration changes conventions or adds new artifacts.
- **Tight subagent inputs**: pass only artifact paths, Delta/Decision excerpts, scoping, and user notes to subagents. ~~Re-stating output formats, focus lists, role assignments, target paths already enumerated in shared artifacts, or blanket read orders~~ → pass only what the callee cannot derive from its own agent file.
- **Nested code fences**: when a fenced code block contains another fenced code block, the outer fence must use more backticks than the inner (e.g. ```` for outer when inner uses ```). Prevents premature closure of the outer block. Applies to templates, machine-artifact diff blocks, and reviewer output format examples.

# Reference Paths

- Plugin types: `opencode-source/packages/plugin/src/index.ts`
- Tool helper: `opencode-source/packages/plugin/src/tool.ts`
- Shell types: `opencode-source/packages/plugin/src/shell.ts`
- TUI types: `opencode-source/packages/plugin/src/tui.ts`
- Existing plugins: `config/plugins/`

# Templates

## `PROMPT-PLUGIN-PLAN.handoff.md`

````markdown
# Iteration Handoff

Source Context: <absolute path to `PROMPT-PLUGIN-PLAN.md`>

## Raw Request

```text
<verbatim user request or current consolidated request>
```

## Supplementary Context
- <repo fact, boundary, or pattern not already in source context Discovery>
- <or `None`>

## Constraints
- <explicit user or repo constraint>
- <or `None`>

## Success Criteria
- <what must be true when the work is done>
- <or `None`>

## Scope
- In scope: <what this iteration covers>
- Out of scope: <what this iteration intentionally leaves alone>

## Summary
- <brief goal and shape of the change>

## Revision History
- Iteration 1: Initial draft.

## REV Index

| REV | Target | Action | File |
| --- | ------ | ------ | ---- |
| REV-001 | `path/to/file` | CREATE | `PROMPT-PLUGIN-PLAN.rev.001.md` |
| REV-002 | `path/to/file` | UPDATE | `PROMPT-PLUGIN-PLAN.rev.002.md` |

## Delta
- Source Context — Status: Unchanged | Changed | New; Touched: `PROMPT-PLUGIN-PLAN.md`; Why: <why reviewers do or do not need to reread source context>
- Review Ledger — Status: Unchanged | Changed | New; Touched: `PROMPT-PLUGIN-PLAN.handoff.md`; Why: <why arbitration state changed or stayed stable>
- REV-### — Status: Unchanged | Changed | New; Touched: `path/from/project/root`; Why: <smallest reason this item changed>

## Review Ledger

### Decisions

#### [DEC-001]
Type: DOMAIN_AUTHORITY | ARBITRATION
Issue: COR-001
Winner: <reviewer_name>
Rationale: <why this view prevailed>
````

## `PROMPT-PLUGIN-PLAN.rev.*.md` files

Each file `PROMPT-PLUGIN-PLAN.rev.NNN.md` contains one revision item:

````markdown
# REV-NNN: `path/to/file`

Action: CREATE | UPDATE | DELETE
Why: <why this file changes>
Anchor: `<existing section or frontmatter field>` | `None`
Lines: ~<start>-<end> | `None`
Insert at: before | after | replace `<anchor or region>` | `None`

Diff:

```diff
<one or more diff blocks — include 2+ context
lines before and after each change.
a single block if changes are contiguous or frontmatter+content
are close together; multiple blocks if scattered.>
```

Changes:
- <summary for quick scanning>
Dependencies: None | REV#
Evidence: `path/to/file:line`
````