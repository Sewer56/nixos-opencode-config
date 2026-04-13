# Iterate Optimizations

Reference for optimization patterns used by the `/iterate` workflow
and other similar workflows.

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

If Delta did not change:
- Reuse prior analysis and cache
- Re-emit valid output

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
