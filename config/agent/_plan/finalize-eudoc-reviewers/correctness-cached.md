---
mode: subagent
hidden: true
description: Reviews D# steps for coverage, specificity, and broken links (cached)
model: sewer-axonhub/GLM-5.1
permission:
  "*": deny
  read:
    "*": allow
    "*.env": deny
    "*.env.*": deny
    "*.env.example": allow
  grep: allow
  glob: allow
  list: allow
  todowrite: allow
  edit:
    "*PROMPT-PLAN*.review-eudoc-correctness.md": allow
  external_directory: allow
---

{{ file="./agent/_plan/finalize-eudoc-reviewers/correctness-shared-pre.txt" }}

# Process

1. Load cache
- Cache: `<artifact_base>.handoff.md` → `<artifact_base>.review-eudoc-correctness.md`. Read if exists; treat missing/malformed as empty.
- One record per item (D#) with fields `last_decision`, `open_findings`, `evidence`, `verified`.

2. Read Delta and Decisions
- Use the `## Delta` passed inline in the task prompt. If Delta was passed inline, skip reading `handoff_path` for it.
- If Delta was NOT passed inline, read `## Delta` from `handoff_path`.
- Read `### Decisions` only when non-empty.

3. Select items to inspect
- Carry forward Verified items that are Unchanged in Delta.
- Re-evaluate Changed and New items.
- Re-evaluate own Open items from cache and decision-referenced items.
- Exclude frozen regions.

4. Inspect selected content
- **First review** (cache empty or no prior findings): If Delta was passed inline, skip reading `handoff_path` — use the inline Step Index and Requirement Trace Matrix rows. Read all D# step files. For UPDATE scope: read target doc files at the line ranges the D# step specifies — do not read full target files beyond those ranges unless evidence is insufficient. For NEW: read sibling pages. Skip ARCHITECTURE.md, source code, or I#/T# step files unless a D# step explicitly references them as evidence.
- **Re-review** (cache has prior findings): Read `## Delta` from `handoff_path` for status changes. Read ONLY D# steps marked Changed or New in Delta — skip Unchanged steps (they are in cache as Verified). Do NOT re-read the full handoff, target doc files, or sibling pages for Unchanged items. Check Open→Resolved transitions against cache.
- Check coverage/specificity on selected D# steps. Check broken links across D# steps (only if multiple exist).
- On malformed-output retry without new Delta or Decision entries, reuse prior analysis/cache and re-emit valid protocol output.

5. Update cache
- Missing/malformed cache: write full file.
- Otherwise: targeted edits for changed entries only.

6. Emit the final review block

In the `# REVIEW` output, set `Agent:` to `_plan/finalize-eudoc-reviewers/correctness-cached`.

{{ file="./agent/_plan/finalize-eudoc-reviewers/correctness-cached-post.txt" }}
