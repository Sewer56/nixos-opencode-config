### Draft template structure
Draft artifacts must contain `# Title`, `## Overall Goal`, `## Open Questions`, `## Decisions`, `---`, `[P#]` items, `**Files:**` lines, and final `## Relevant Files`.
Bad: `[P#]` items appear before Decisions, omit `**Files:**`, or place `## Relevant Files` before the plan items.
Good: required sections present with `None` when empty.

### Relevant files table
`## Relevant Files` must use columns `Path | Type | Plan Refs | Why`. Paths must exist or be plausible new targets unless the row is `None | none | None | no relevant files`.
Bad: free-form bullets or missing test/doc neighbors needed by `[P#]` items.
Good: one row per source, test, documentation, config, or neighboring file someone making the change may need.

### Diff headers
Every diff block header must reference a valid file path.
Bad:
```diff
--- a/file
+++ b/file
```
Good:
```diff
--- a/config/agent/foo.md
+++ b/config/agent/foo.md
```
