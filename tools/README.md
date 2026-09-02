# Tools

Rust workspace for local OpenCode utilities.

## Binaries

- `opencode-model-switcher` — TUI for `# EASY` / `# MEDIUM` / `# HARD` model tiers and variants. Config: `config/model-switcher.json`.
- `opencode-sessions` — browse/export OpenCode SQLite sessions.
- `chunk-files-by-tokens` — split files/directories into token-sized chunks.
- `token-count-after-expand` — render md-expand prompt files and estimate tokens.
- `opencode-yolo-mode` — flip external_directory `'*'` ask↔allow across agent frontmatter and `config/opencode.json`; guards after `'*'` (secrets deny/ask) keep winning.
- `rust-llm-tidy` — reorder/lint Rust source; built from the submodule input.

## Shell helpers

- `tools/render-file.sh <path>` — render one md-expand prompt file.
- `tools/validate-file-interp.sh [paths...]` — validate md-expand references.

## Examples

```bash
cargo run -p opencode-model-switcher              # Launch TUI (default profile)
cargo run -p opencode-model-switcher -- normal    # Launch TUI with "normal" profile
cargo run -p chunk-files-by-tokens -- -s 32000 config/agent
cargo run -p token-count-after-expand -- config/agent/mcp-search.md
cargo run -p opencode-yolo-mode -- status   # report mode; also: on, off
nix run .#rust-llm-tidy -- reorder --dry-run src/main.rs
```
