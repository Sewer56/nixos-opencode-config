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

1. Write the full styled message to a temp file ending in `.md`.
2. Run `rust-llm-tidy --no-config --dry-run --json <file>`.
3. Apply every finding and the conciseness style, then save and rerun.
4. Repeat until the JSON output is `[]`.
5. Create a new commit by default with `git commit -F <file>`.
6. Amend only on explicit user request.
   Amend after confirming inspected `HEAD` is the intended target.
   For that amend, use `git commit --amend -F <file>`.
7. Delete the temp file after the commit.

If the binary is unavailable, commit without the tidy pass.
Report the skipped pass in the output; this is non-blocking.
