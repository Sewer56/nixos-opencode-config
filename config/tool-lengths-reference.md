# Tool Description Length Reference

Tool `.txt` files contain the JSON Schema `description` field — **one short sentence only**.
Usage guidance goes in `system-prompt-builder.ts`. Cross-tool rules go in `tool-prompt-facts.ts`.
Domain workflows (git, PRs) go in `supplemental/*.txt`.

## Tool .txt targets

| Tool | Target content | Target chars |
|------|---------------|-------------|
| bash | Run a shell command in a persistent shell session with optional timeout. | ~80 |
| read | Read a file or directory from the local filesystem. | ~55 |
| edit | Replace exact text in a file. Without replaceAll, oldString must match exactly once. | ~80 |
| write | Write a file. Creates parent directories and overwrites existing files. | ~70 |
| glob | Find files by glob pattern. Respects .gitignore and sorts newest first. | ~70 |
| grep | Search file contents with a regex. Returns matching lines with line numbers, sorted newest first. | ~90 |
| task | Delegate work to one of the listed subagents. | ~45 |
| todowrite | Replace the full todo list. | ~25 |
| webfetch | Fetch one URL. HTML is converted to Markdown and JSON is pretty-printed. | ~70 |
| websearch | Web search. Current year: {{year}} — use for recent/current event queries. | ~75 |
| question | Ask user a question. Returns selected labels. Use `multiple: true` for multi-select. Custom answer enabled by default. | ~100 |
| lsp | Language Server Protocol code intelligence. | ~45 |
| apply_patch | Apply a multi-file patch. Envelope: `*** Begin Patch` / `*** End Patch`. Operations: Add File, Delete File, Update File (with context_line, +/- prefix). | ~150 |

## System prompt builder sections (system-prompt-builder.ts)

These contain the **usage guidance** — short bullet lists per tool. Modeled after `llm-coding-tools/src/reloaded-code-core/src/context/tool_prompt/tool_sections.rs`.

### Bash section (4 bullets)

```
- Use it for terminal work (git, package managers, test runners, docker) and shell-native search/filter jobs the specialized tools do not handle well.
- Output combines stdout and stderr. Non-zero exit codes not shown.
- For independent commands, make parallel `bash` calls. For dependent commands, use one call with `&&`.
- Quote paths that contain spaces.
```

### Read section (4-5 bullets, conditional)

```
- Returns `{n}: text`. Lines over 2000 chars are truncated.
- (If glob + bash) Reads files and directories. Use `glob` to find files or `bash` for directory listings.
- (If glob only) Reads files and directories. Use `glob` to find files.
- (If bash only) Reads files and directories. Use `bash` for directory listings.
- (If neither) Reads files and directories.
- Missing files return an error. Binary files cannot be read.
- Read related files in parallel when useful.
```

### Write section (1-2 bullets, conditional)

```
- Existing files are overwritten.
- (If no edit tool) Use this for new files or full rewrites, not small edits.
```

### Edit section (1 bullet)

```
- `old_string` must be non-empty, differ from `new_string`, and appear exactly once (unless `replaceAll`).
```

### Glob section (3 bullets)

```
- Supports *, **, ?, [abc], and {a,b}.
- Returns matching file paths as absolute paths, sorted newest first.
- Results are capped at 100; large result sets are truncated.
```

### Grep section (2-4 bullets, conditional)

```
- `pattern` must not be empty. Search is single-line only; there is no multiline matching.
- Returns matches grouped by file.
- (If bash present) Use this instead of shell `grep`/`rg`.
- (If no glob and no read) Use it for content search, not file-name search or full-file inspection.
```

### Task section (2-3 bullets, conditional)

```
- Use for real delegation or parallel sub-work. Include full context; stateless — don't rely on prior state.
- (If read/glob/grep present) Do not use it when `read`, `glob`, `grep` on one or a few files is enough.
- Results are private to you; summarize for the user.
```

## Common rules (tool-prompt-facts.ts)

Modeled after `llm-coding-tools/src/reloaded-code-core/src/context/tool_prompt/common_rules.rs`.

```
Prefer `glob`, `grep`, `read`, `edit`, and `write` over `bash` for ordinary file work.
Use `glob` for file-name search, `grep` for content search, and `read` for file content.
Prefer `edit` for targeted changes and `write` for new files or full rewrites.
Read before `edit` or overwriting with `write`; for `edit`, copy exact text and omit any `{n}: ` prefixes.
```

## Supplemental context (enabled per-agent)

- `git-workflow.txt`: commit protocol, safety rules (replaces bash.txt git commit section)
- `github-cli.txt`: PR creation with `gh` (replaces bash.txt PR creation section)
