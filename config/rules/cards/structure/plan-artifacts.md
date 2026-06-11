### Draft template structure
Draft artifacts must contain `# Title`, `## Overall Goal`, `## Open Questions`, `## Decisions`, `---`, `[P#]` items, `**Files:**` lines, and final `## Relevant Files`.
Use `None` for empty required sections.

### Relevant files table
`## Relevant Files` must use columns `Path | Type | Plan Refs | Why`. Paths must exist or be plausible new targets unless the row is `None | none | None | no relevant files`.
Include one row per source, test, documentation, config, or neighboring file an implementer may need.

### Diff headers
Every diff block header must reference a valid file path.
Block placeholder headers such as `--- a/file` or `+++ b/file`.
