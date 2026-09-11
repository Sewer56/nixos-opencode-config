# Tools

Rust workspace for local OpenCode utilities.

## Binaries

- `opencode-model-switcher`: edits seven model/variant tiers.
  Config: `config/model-switcher.json`.
  Roles: `STYLE-REVIEW`, `CORRECTNESS-REVIEW`, `CODER`, `WRITER`;
  existing tiers: `EASY`, `MEDIUM`, `HARD`.
- `opencode-sessions`: browse/export OpenCode SQLite sessions.
- `chunk-files-by-tokens`: split files/directories into token-sized chunks.
- `token-count-after-expand`: render prompts and estimate tokens.
- `opencode-yolo-mode`: toggle external_directory `'*'` between ask and allow.
  Applies to agent frontmatter and `config/opencode.json`.
  Later secret guards retain precedence.
- `rust-llm-tidy`: reorder/lint Rust source; built from the submodule input.

## Shell helpers

- `tools/render-file.sh <path>`: render one md-expand prompt file.
- `tools/validate-file-interp.sh [paths...]`: validate md-expand references.

## Examples

```bash
cargo run -p opencode-model-switcher              # Launch TUI (default profile)
cargo run -p opencode-model-switcher -- normal    # Launch TUI with "normal" profile
cargo run -p chunk-files-by-tokens -- -s 32000 config/agent
cargo run -p token-count-after-expand -- config/agent/subagent/web-search.md
cargo run -p opencode-yolo-mode -- status   # report mode; also: on, off
nix run .#rust-llm-tidy -- reorder --dry-run src/main.rs
```
