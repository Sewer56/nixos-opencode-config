# Iterate Optimizations

Reference for optimization patterns used by the `/iterate` workflow
and other similar workflows.

## Ordered-Step Placement

Put ordered steps near the top of `/iterate/finalize` and iterate reviewer
prompts.
Keep the ordered step list contiguous.
Move supporting reference material — inputs, focus notes, templates, and
examples — below the ordered steps when the file shape allows it.

For reviewers, use this order:
1. Load cache
2. Read Delta and Decisions
3. Reopen only Changed, New, cached-open, or decision-referenced REV items
4. Inspect only the selected `machine_path` sections and target files
5. Write cache
6. Emit the required final output block

Keep the output step last so the required review block or finalize status block
remains the final answer.
For finalize, keep the review-loop steps together in `# Process` and place prompt
examples in the reference sections below the ordered steps.

## Per-Reviewer Cache and Delta

Applies only to targets that run a review loop or coordinate subagents.

Each reviewer owns a cache file (`PROMPT-ITERATE.review-<domain>.md`). It reads
the cache at start and writes at end.

The finalize agent rewrites `## Delta` before the first review pass, then
recomputes it after every material revision.

Each Delta entry records:
- `Status`: Unchanged | Changed | New
- `Touched`: path to the file that changed
- `Why`: brief reason for the change

Artifact markers for `Source Context` and `Review Ledger` let reviewers skip
rereading unchanged artifacts.

Reviewers skip re-evaluating Unchanged items. They only check:
- Changed items
- New items
- Decision-referenced items
- Cached-open items

Cache files:
- `PROMPT-ITERATE.review-correctness.md`
- `PROMPT-ITERATE.review-economy.md`
- `PROMPT-ITERATE.review-style.md`
- `PROMPT-ITERATE.review-performance.md`

## Read-on-Demand Reviewers

Reviewers start from cache plus Delta. They carry forward cached `PASS` items
with no open findings when their Delta state remains `Unchanged`.

Read `machine_path` sections first. Then open target files only for:
- Changed items
- New items
- Cached-open items
- Decision-referenced REV items

## Format-Only Retries

Malformed-output retries are protocol fixes, not rediscovery.

If Delta and Decisions did not change:
- Reuse prior analysis and cache
- Re-emit valid output from the existing review state
- Keep the retry format-only

Re-read artifacts only when the retry includes new Delta or Decision entries.

## Fixed Output Format

All iterate reviewers return structured output in fenced code blocks with `text`
language tag.

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

When a finalize agent or orchestrator coordinates multiple subagents, use a
shared ledger or coordination file for cross-domain arbitration.

Domain-internal issue tracking stays in reviewer cache files. The Review Ledger
in handoff contains only `### Decisions`.

## Tight Subagent Inputs

Applies to any command or agent that spawns subagents (reviewers, explorers,
etc.).

The callee's agent file is the contract. The caller trusts it, not re-states it.

Include:
- Artifact paths the callee cannot discover on its own
- Delta and Decision excerpts plus scoping context
- User-supplied notes or arguments affecting the task

Omit:
- Output format — the callee's agent file already defines this
- Focus/check lists — the callee's agent file already defines these
- Role assignment — the callee's agent file already defines this
- Target file paths already enumerated in a shared artifact the callee receives
- Blanket read orders — the callee uses Delta and cache state to choose what to
  open

## Self-Iteration

When `/iterate` targets `_iterate` agents, reviewers, or iterate commands, the draft agent detects self-iteration from target paths and classifies intent as `wording-only` or `rule-change`. Detection is path-based — no new flags or commands. Non-self iterations are unaffected.

- **wording-only**: text clarifications with no enforcement-logic impact. Standard finalize and review flow.
- **rule-change**: modifications to instructions governing future `/iterate` output. Requires at least one REV updating enforcement logic; the correctness reviewer blocks if missing.

### wording-only example

Request: "Clarify the description of Process step 3 in draft.md"

Generated `## Self-Iteration`: `Intent: wording-only`, `Target-Scope: .opencode/agent/_iterate/draft.md`

### rule-change example

Request: "Add a new optimization rule to draft.md that reviewers must enforce"

Generated `## Self-Iteration`: `Intent: rule-change`, `Target-Scope: .opencode/agent/_iterate/draft.md, .opencode/agent/_iterate/reviewers/correctness.md`

The machine artifact must include a REV updating the reviewer focus list to enforce the new rule.
