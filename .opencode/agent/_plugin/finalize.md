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
    "*PROMPT-PLUGIN-PLAN.machine.md": allow
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
- `machine_path`: `PROMPT-PLUGIN-PLAN.machine.md`

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
- Write `handoff_path` using these sections: `# Iteration Handoff` (with `Source Context` path), `## Raw Request` (verbatim text block), `## Supplementary Context` (bullet list or `None`), `## Constraints` (bullet list or `None`), `## Success Criteria` (bullet list or `None`), `## Scope` (in/out bullets), `## Delta` (per-REV entries with `Status:`, `Touched:`, `Why:` fields plus Source Context and Review Ledger markers), `## Review Ledger` (`### Decisions` subsection for cross-domain arbitration only).

## 4. Write the machine artifact
- Derive discrete `REV-###` items from the confirmed context and handoff.
- Apply only the relevant optimization rules to each target. Split rule fragments across the affected prompts and reviewers instead of copying the whole contract into every file.
- Embed operational rules directly in generated targets.
- Follow diff-block mechanics in `# Rules` for combining, scattering, insertions, and deletions.

## 5. Run the review loop

1. Write and maintain `## Delta`
- Write `## Delta` to `handoff_path` before the first reviewer pass.
- Record each `REV-###` item as a compact entry with `Status:`, `Touched:`, and `Why:` fields.
- Add artifact markers for `Source Context` and `Review Ledger` so reviewers skip rereading unchanged artifacts.
- Recompute `## Delta` after every material revision.

2. Build reviewer prompts
- After each full machine-artifact draft, run these reviewers in parallel:
  - `@_plugin/reviewers/errors`
  - `@_plugin/reviewers/reorder`
  - `@_plugin/reviewers/documentation`
  - `@_plugin/reviewers/correctness`
- Include only: artifact paths (`context_path`, `handoff_path`, `machine_path`), Delta summary from `## Delta`, current `### Decisions` excerpt when non-empty, finalize-time user notes.
- Omit: output format, focus lists, target file paths, role assignments, blanket read orders — reviewers define their own contracts.

3. Validate each reviewer response
- Confirm the response starts with `# REVIEW`.
- Confirm the response contains `Decision: PASS | ADVISORY | BLOCKING`.
- Confirm the response contains `## Findings` and `## Verified` headings.
- If the response remains malformed after retries, treat it as BLOCKING with a synthetic finding.

4. Retry malformed responses
- If validation fails and Delta plus Decisions are unchanged, send only the specific protocol error and request re-emit from existing review state.
- If validation fails after a material revision changed Delta or Decisions, include only the new Delta/Decision excerpt and request a fresh response.

5. Record decisions and apply domain ownership
- Update `### Decisions` in `handoff_path` for cross-domain arbitration only.
- Apply domain ownership: ERRORS → errors reviewer; REORDER → reorder reviewer; DOCUMENTATION → documentation reviewer; CORRECTNESS → correctness reviewer.

6. Revise the machine artifact when findings require it
- Revise `machine_path` only where needed.
- Append one line to `## Revision History`.

7. Re-run or finish
- Re-run all reviewers after every material revision.
- Loop until no findings remain or 10 iterations.
- No findings: SUCCESS. At cap: FAIL if BLOCKING, SUCCESS with risks if only ADVISORY.

# Output

Return exactly:

```text
Status: SUCCESS | INCOMPLETE | FAIL
Context Path: <absolute path>
Handoff Path: <absolute path>
Machine Path: <absolute path>
Review Iterations: <n>
Summary: <one-line summary>
```

# Constraints

- Write only `PROMPT-PLUGIN-PLAN.handoff.md` and `PROMPT-PLUGIN-PLAN.machine.md` during finalize.
- Modify only those two files during finalize.
- Read `PROMPT-PLUGIN-PLAN.md` as source of truth only; write to handoff and machine paths.
- Keep `PROMPT-PLUGIN-PLAN.machine.md` diff-based: each REV item uses diff blocks grounded in the current file state with approximate line ranges and anchors per `# Rules`. CREATE actions include full file content.
- Keep `PROMPT-PLUGIN-PLAN.handoff.md` factual and stable enough for the machine artifact and reviewers to use without rereading the whole conversation.

# Rules

Apply these rules when writing `machine_path`:

- Write concrete values for every field and body — omit `...`, `TODO`, and comment-only stubs.
- Specify the full path for every file reference: REV headings, `Evidence` fields, and diff block targets all use fully qualified paths from the project root.
- Reference anchors and approximate line ranges inside diff blocks.
- Insertions use normal code blocks with `Insert at: <anchor> (~start-end)`.
- Edits/removals use `diff` blocks; deletions include `Remove lines: ~start-end`.
- If frontmatter and content changes are contiguous, combine into a single diff block.
- If changes are scattered across a file, use multiple diff blocks within one REV item.
- CREATE actions include full file content in a normal code block (not a diff against empty).
- Diff blocks target markdown files — use markdown-aware line references (headings, list items, fenced code blocks).
- Line numbers in `@@` headers are approximate (±10 lines); include 2+ unchanged context lines before and after each change region so the implementer can locate changes by content matching rather than line counting.

---

# Optimization Rules

Revisions produced by this finalize run must follow. Apply only the relevant rules below to each generated target and reviewer prompt:

- **Reviewer cache + Delta**: targets that run review loops or coordinate subagents include per-reviewer cache files and a Delta section in handoff so reviewers skip unchanged items on re-runs.
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
- Existing plugins: `config/plugins/*.ts`
