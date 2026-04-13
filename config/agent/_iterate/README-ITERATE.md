# Iterate Optimizations

Reference for optimization patterns used by the `/iterate` workflow.

## Per-Reviewer Cache and Delta

Applies only to targets that themselves run a review loop or coordinate subagents. Each reviewer owns a cache file (`PROMPT-ITERATE.review-<domain>.md`) it reads at start and writes at end. The finalize agent writes a `## Delta` section to handoff listing which REV items changed. Reviewers skip re-evaluating Unchanged items — only Changed, New, and Open items are evaluated per pass.

Cache files:
- `PROMPT-ITERATE.review-correctness.md`
- `PROMPT-ITERATE.review-economy.md`
- `PROMPT-ITERATE.review-style.md`
- `PROMPT-ITERATE.review-performance.md`

## Fixed Output Format

All iterate reviewers return structured output in fenced code blocks with `text` language tag. Output starts with `# REVIEW`, contains `Decision: PASS | ADVISORY | BLOCKING`, and includes `## Findings` and `## Verified` headings.

## No Duplicated Artifact Content

Do not re-state information available in another artifact. Reference by section name or file path instead. Applies pairwise: context↔handoff, context↔machine, handoff↔machine, machine↔targets, targets↔targets.

## File-Based Coordination

When a finalize agent or orchestrator coordinates multiple subagents, use a shared ledger or coordination file for cross-domain arbitration. Domain-internal issue tracking stays in reviewer cache files — the Review Ledger in handoff contains only `### Decisions`.

## Tight Subagent Inputs

Applies to any command or agent that spawns subagents (reviewers, explorers, etc.). The callee's agent file is the contract — the caller trusts it, not re-states it.

Include:
- Artifact paths the callee cannot discover on its own
- Deltas and scoping context
- User-supplied notes or arguments affecting the task

Omit:
- Output format, focus/check lists, role assignment — the callee's agent file already defines these
- Target file paths already enumerated in a shared artifact the callee receives
