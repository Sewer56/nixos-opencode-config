### Draft template structure
Draft artifacts must contain `# Title`, `## Overall Goal`, `## Open Questions`, `## Decisions`, `---`, `[P#]` items, and `**Files:**` lines.
Bad: `[P#]` items appear before Decisions or omit `**Files:**`.
Good: required sections present with `None` when empty.

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
