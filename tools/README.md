# Tools

Rust workspace for local OpenCode utilities.

## Binaries

- `opencode-model-switcher` — TUI for `# LOW` / `# MED` / `# HIGH` model tiers. Config: `config/model-switcher.json`.
- `opencode-sessions` — browse/export OpenCode SQLite sessions.
- `chunk-files-by-tokens` — split files/directories into token-sized chunks.
- `token-count-after-expand` — render md-expand prompt files and estimate tokens.
- `iterate-static-check` — deterministic static checks for iterate/edit artifacts.

## Shell helpers

- `tools/render-file.sh <path>` — render one md-expand prompt file.
- `tools/validate-file-interp.sh [paths...]` — validate md-expand references.

## Examples

```bash
cargo run -p opencode-model-switcher              # Launch TUI (default profile)
cargo run -p opencode-model-switcher -- normal    # Launch TUI with "normal" profile
cargo run -p chunk-files-by-tokens -- -s 32000 config/agent
cargo run -p token-count-after-expand -- config/agent/mcp-search.md
cargo run -p iterate-static-check -- PROMPT-EXAMPLE
```
