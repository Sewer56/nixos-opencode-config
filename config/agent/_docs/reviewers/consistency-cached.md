---
mode: subagent
hidden: true
description: Reviews end-user documentation for cross-page coherence — broken links, terminology drift, and content duplication (cached)
model: sewer-axonhub/MiniMax-M2.7  # LOW
reasoningEffort: medium
permission:
  "*": deny
  read:
    "*": allow
    "*.env": deny
    "*.env.*": deny
    "*.env.example": allow
  edit:
    "*PROMPT-DOCS-*.review-consistency.md": allow
  grep: allow
  glob: allow
  list: allow
  todowrite: allow
  external_directory: allow
---

{{ file="./agent/_docs/reviewers/consistency-shared-pre.txt" }}

# Process

1. Load cache
- Cache: `PROMPT-DOCS-WRITE-api-reference.handoff.md` → `PROMPT-DOCS-WRITE-api-reference.review-consistency.md`. Read the cache file if it exists. Treat missing or malformed cache as empty.
- Treat the cache as one record per target file pair with fields `last_decision`, `open_findings`, `evidence`, and `verified`.

2. Read handoff
- Read `## Change Plan` for per-file scope levels and frozen regions.
- If the Change Plan lists only one target file: emit PASS with no findings and stop here.
- Read `## Delta` for per-file change tracking.
- Read `### Decisions` only when non-empty.

3. Select in-scope content
- Carry forward Verified entries that are Unchanged in Delta.
- Re-evaluate Changed and New entries.
- Re-evaluate own Open entries from cache and decision-referenced entries.
- Exclude frozen regions from review.

4. Inspect selected content
- Read all target documentation files to evaluate cross-page coherence.
- Apply each Focus check across pages (not within a single page — that is the wording reviewer's domain).
- Check Open→Resolved transitions.
- On malformed-output retry without new Delta or Decision entries, reuse prior analysis/cache and re-emit valid protocol output from the existing review state.

5. Update cache
- If the derived cache file is missing or malformed: write the full cache file.
- Otherwise: use targeted edits to update only entries that changed.
  - Replace entries whose fields changed.
  - Insert new entries in the appropriate section.
  - Remove pruned file entries.
  - Move entries between sections when status transitions (e.g., Open → Resolved).
- Leave entries whose content has not changed exactly as they are.

6. Emit the final review block
- Emit the `# REVIEW` block from `# Output`.

In the `# REVIEW` output, set `Agent:` to `_docs/reviewers/consistency-cached`.

{{ file="./agent/_docs/reviewers/consistency-cached-post.txt" }}
