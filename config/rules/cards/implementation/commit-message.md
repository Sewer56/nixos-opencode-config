### Commit message
Subject: `<Prefix>: concise outcome`, one line.
Use one of these prefixes:
- `Added:` new features
- `Changed:` changes to existing functionality
- `Deprecated:` soon-to-be removed features
- `Removed:` removed features
- `Fixed:` bug fixes
- `Security:` vulnerability fixes
- `Perf:` performance work

Body: short one-line bullets, information-dense.
Add concrete numbers when known.
No multi-line bullets, no file inventory, no implementation transcript.

A key-detail paragraph may follow the bullets only for important context.
Important context is caveats, compatibility, or migration notes.
Otherwise omit the paragraph.

Use subject only for small changes.
One logical change per commit.

Draft, review, and refine each commit message through `rust-llm-tidy`.
The tool only processes `.rs` and `.md` files.
`--no-config` keeps the pass deterministic in any target repo.

1. Draft the message per the commit style.
2. Write the full message to a temp file ending in `.md`, e.g. under `/tmp/`.
3. Review non-mutating with `rust-llm-tidy --no-config --dry-run --json <file>`.
4. Refine: apply every finding plus the conciseness style, then save and rerun.
5. Repeat until the JSON output is `[]`.
6. Create a new commit by default with `git commit -F <file>`.
7. Amend only on explicit user request via `git commit --amend -F <file>`, after confirming inspected `HEAD` is the intended target.
8. Delete the temp file after the commit.

If the binary is unavailable, commit without the tidy pass.
Report the skipped pass in the output; this is non-blocking.
