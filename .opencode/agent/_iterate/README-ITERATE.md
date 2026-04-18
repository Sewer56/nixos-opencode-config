# Iterate Optimizations

Reference for optimization patterns used by the `/iterate` workflow
and other similar workflows.

- [Split REV Files](#split-rev-files)
  — each revision item in its own file; handoff absorbs machine.md
- [Section Ordering Convention](#section-ordering-convention)
  — Inputs → Process → Supplemental ordering for produced files
- [Cache and Delta](#cache-and-delta)
  - [Mechanism](#mechanism)
    — each reviewer owns a cache file; finalize writes the change list
  - [Process Step Order](#process-step-order)
    — reviewer step sequence: load → delta → select → inspect → cache → emit
  - [Items to Re-Evaluate](#items-to-re-evaluate)
    — reviewers open only changed, new, unresolved-finding, or
      decision-referenced items
  - [Malformed-Output Retries](#malformed-output-retries)
    — fix format only; do not re-read files or redo analysis
- [Fixed Output Format](#fixed-output-format)
  — all reviewers return structured `# REVIEW` blocks in `text` fences
- [No Duplicated Artifact Content](#no-duplicated-artifact-content)
  — reference by section name or path, never re-state
- [File-Based Coordination](#file-based-coordination)
  — one shared file for reviewer disagreements, not scattered state
- [Tight Subagent Inputs](#tight-subagent-inputs)
  — pass only what the called agent cannot derive from its own file
- [Self-Iteration](#self-iteration)
  — path-based detection of wording-only vs rule-change self-iteration
 - [Line-location Convention](#line-location-convention)
  — `Lines: ~<start>-<end>` locates changes; context is authoritative
- [Human-Friendly [P#] Items](#human-friendly-p-items)
  — draft-stage items use explanation + diff with paths in diff headers
- [Focus-as-Scope](#focus-as-scope)
  — Focus is the reviewer scope boundary; meta enforces no overlap

## Split REV Files

Applies to all finalize pipelines: `_iterate`, `_plugin`, `_plan`, and
`_orchestrator`.

### No Separate machine.md

The `machine.md` artifact is eliminated. Handoff absorbs its content:
Summary, Revision History, and REV/Step Index. All actionable diff/step
content lives in individual files matching `rev_pattern` or `step_pattern`.

Consumers read: handoff (single coordination document) + per-item files matching the pattern.

### File Layout

- `PROMPT-ITERATE.handoff.md` — coordination document with Summary,
  Revision History, REV Index, Delta, and Review Ledger
- `PROMPT-ITERATE.rev.001.md` — first revision item
- `PROMPT-ITERATE.rev.002.md` — second revision item
- (gaps are valid; deleted items leave holes in numbering)

For `_plugin`: same pattern with `PROMPT-PLUGIN-PLAN.rev.*.md`.

For `_plan`: implementation and test steps split into
`PROMPT-PLAN.step.I1.md`, `PROMPT-PLAN.step.T1.md`, etc.
Requirements, mapping, trace matrix, and external symbols stay in
the handoff.

For `_orchestrator`: the planner splits steps into
`PROMPT-NN-*-PLAN.step.I1.md`, `PROMPT-NN-*-PLAN.step.T1.md`, etc.

### Stable Numbering

Items are numbered sequentially. If an item is removed during finalize's
internal revision loop, the gap is left in place — no renumbering.
Gaps (e.g., 001, 003, 005) are valid.

### Consumption Pattern

Reviewers and implementers read handoff first for context and the
index, then read all needed individual files in parallel (issue all
read calls in one batch). This avoids loading all revision content
into context when only a subset changed, and minimizes round-trips.

## Section Ordering Convention

All command and agent files produced by `/iterate` follow
Inputs → Process → Supplemental section ordering.

**Inputs zone** — what the agent/command receives:
`# Inputs`, `# Artifacts`, `# Derived Paths`, `# Prerequisites`,
`## User Input`

**Process zone** — ordered steps and execution contracts:
`# Process`, `# Workflow`, `## Workflow`

**Supplemental zone** — everything else:
`# Output`, `# Constraints`, `# Rules`, `# Focus`, `# Capabilities`,
`# Safety`, `# Templates`, `# Examples`, `# Guidelines`, `# Defaults`,
`# Blocking Criteria`, `# Issue Categories`

Within Supplemental, prefer: Output → Constraints → Rules →
Templates/Examples. This sub-ordering is advisory — minor variations
acceptable.

Exemptions:
- Pure-proxy commands (frontmatter + `$ARGUMENTS` only)
- Simple capability agents (role + Focus/Capabilities/Safety only)
- `_iterate` reviewer files (Focus defines the review process and sits
  before Process by design)

Section heading style: `# Inputs` for agents, `## User Input` for
commands. Keep existing heading levels; only reorder sections.

Keep the ordered step list within Process contiguous. Move supporting
reference material — inputs, focus notes, templates, and examples —
below Process into Supplemental when the file shape allows it. Keep the
output step last so the required review block or finalize status block
remains the final answer.

## Cache and Delta

Applies only to targets that run a review loop or coordinate
subagents.

### Mechanism

Each reviewer owns a cache file
(`PROMPT-ITERATE.review-<domain>.md`). It reads the cache at start
and updates only changed entries at end. Full write only when the
cache is missing or malformed.

The finalize agent rewrites `## Delta` before the first review pass,
then recomputes it after every material revision.

Each Delta entry records:
- `Status`: Unchanged | Changed | New
- `Touched`: path to the file that changed
- `Why`: brief reason for the change

Artifact markers for `Source Context` and `Review Ledger` let
reviewers skip rereading unchanged artifacts.

Reviewers skip re-evaluating Unchanged items. They only check:
- Changed items
- New items
- Decision-referenced items
- Items with unresolved findings from cache

Cache files:
- `PROMPT-ITERATE.review-correctness.md`
- `PROMPT-ITERATE.review-wording.md`
- `PROMPT-ITERATE.review-style.md`
- `PROMPT-ITERATE.review-performance.md`
- `PROMPT-ITERATE.review-dedup.md`
- `PROMPT-ITERATE.review-diff.md`
- `PROMPT-ITERATE.review-meta.md`

### Process Step Order

For reviewers, the Process-zone step order:
1. Load cache
2. Read Delta and Decisions
3. Reopen only Changed, New, items with unresolved findings, or
   decision-referenced REV items
4. Read the REV Index from handoff, then read selected REV files matching `rev_pattern`
5. Update cache — only changed entries
6. Emit the required final output block

For finalize, keep the review-loop steps together in `# Process` and
place prompt examples in the reference sections below the ordered steps.

### Items to Re-Evaluate

Reviewers start from cache plus Delta. They carry forward cached
`PASS` items with no open findings when their Delta state remains
`Unchanged`.

Read the REV Index from handoff first. Then read selected REV files matching `rev_pattern`. Open target files only for:
- Changed items
- New items
- Items with unresolved findings from cache
- Decision-referenced REV items

### Malformed-Output Retries

When a reviewer returns badly formatted output, fix the format only —
do not re-read files or redo the analysis.

If Delta and Decisions did not change:
- Reuse prior analysis and cache
- Re-emit valid output from the existing review state
- Keep the retry format-only

Re-read artifacts only when the retry includes new Delta or Decision
entries.

## Fixed Output Format

All reviewers return structured output in fenced code blocks with
`text` language tag.

Output must contain:
- Starts with `# REVIEW`
- `Decision: PASS | ADVISORY | BLOCKING`
- `## Findings` heading
- `## Verified` heading

## No Duplicated Artifact Content

Do not re-state information available in another artifact.

Reference by section name or file path instead. Applies pairwise:
- context ↔ handoff
- context ↔ machine
- handoff ↔ machine
- machine ↔ targets
- targets ↔ targets

## File-Based Coordination

When multiple reviewers disagree, write decisions to one shared file
instead of scattering them across reviewer outputs.

Each reviewer tracks its own issues in its cache file. The handoff
file stores only `### Decisions`.

## Tight Subagent Inputs

Applies to any command or agent that spawns subagents (reviewers,
explorers, etc.).

The called agent's file is the contract — trust it, don't repeat it.

Include:
- Artifact paths the called agent cannot find on its own
- Delta and Decision excerpts plus scope
- User-supplied notes or arguments affecting the task

Omit:
- Output format — the called agent's file already defines this
- Focus/check lists — the called agent's file already defines these
- Role assignment — the called agent's file already defines this
- Target file paths already listed in a shared artifact the called
  agent receives
- Blanket read orders — the called agent uses Delta and cache state
  to choose what to open

## Self-Iteration

When `/iterate` targets `_iterate` agents, reviewers, or iterate
commands, the draft agent detects self-iteration from target paths and
classifies intent as `wording-only` or `rule-change`. Detection is
path-based — no new flags or commands. Non-self iterations are
unaffected.

- **wording-only**: text clarifications with no effect on what rules
  get enforced. Standard finalize and review flow.
- **rule-change**: modifications to rules that control future
  `/iterate` output. Requires at least one REV updating what rules
  get enforced; the meta reviewer blocks if missing.

### wording-only example

Request: "Clarify the description of Process step 3 in draft.md"

Generated `## Self-Iteration`: `Intent: wording-only`,
`Target-Scope: .opencode/agent/_iterate/draft.md`

### rule-change example

Request: "Add a new optimization rule to draft.md that reviewers
must enforce"

Generated `## Self-Iteration`: `Intent: rule-change`,
`Target-Scope: .opencode/agent/_iterate/draft.md,
.opencode/agent/_iterate/reviewers/correctness.md`

The machine artifact must include a REV updating the reviewer focus
list to enforce the new rule.

## Line-location Convention

All finalize agents and reviewers use `Lines: ~<start>-<end> | None`
as the sole line-location indicator in REV and step files
(`~` ≈ ±10 lines). Hunks include 2+ context lines before and
after each change; context is the authoritative locator.
Reviewers validate content, not counts — block only when context
lines are missing or do not match the target file.

## Human-Friendly [P#] Items

Draft-stage `[P#]` items in `PROMPT-ITERATE.md`,
`PROMPT-PLUGIN-PLAN.md`, and `PROMPT-PLAN.md` use free-form
explanation + diff block with paths in diff headers.

File paths appear in the diff block header (`--- a/<path>`).
REFINE/UPDATE actions include the diff block. CREATE/ADD/INSERT
actions use explanation only (or a code snippet for `_plan`).

Format rules (2+ context lines per hunk) follow
the Line-location Convention above.

## Focus-as-Scope

Each reviewer's `# Focus` defines what it checks — anything not
listed is out of scope. The meta reviewer blocks when a Focus item
is broad enough to overlap another reviewer's domain, prompting
the author to narrow or split it.
