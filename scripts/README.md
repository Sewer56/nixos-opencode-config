# Scripts

## `opencode-model-tiers`

Main Go TUI/CLI wrapper for model tier assignment.

```bash
scripts/opencode-model-tiers
scripts/opencode-model-tiers status
scripts/opencode-model-tiers apply normal --dry-run
scripts/opencode-model-tiers apply normal
scripts/opencode-model-tiers work --dry-run
scripts/opencode-model-tiers work
scripts/opencode-model-tiers set normal MED sewer-axonhub/MiniMax-M3
scripts/opencode-model-tiers models
scripts/opencode-model-tiers models --work
scripts/opencode-model-tiers configure work
```

Tier data lives beside the wrapper in `scripts/model-tiers.json`.

The tool reads a `$tierOrder` key to discover the canonical tier list:

```json
{
  "$tierOrder": {"0": "LOW", "1": "MED", "2": "HIGH"},
  "normal": { "LOW": "...", "MED": "...", "HIGH": "..." },
  "work":   { "LOW": "...", "MED": "...", "HIGH": "..." }
}
```

Keys in `$tierOrder` are numeric strings (`"0"`, `"1"`, …) that define the
display and iteration order. Every profile must contain each key from the
order.

If `$tierOrder` is absent, the tool collects every tier key across all
profiles and sorts them alphabetically — useful for ad-hoc or evolving configs.

To add a new tier variant (e.g. `HIGH-FAST`):

1. Add the tier key to `$tierOrder`, e.g. `"3": "HIGH-FAST"`.
2. Add the tier value to each profile: `"HIGH-FAST": "provider/model"`.
3. Agent files can now use `# HIGH-FAST` markers.

No recompilation needed — the tool adapts at next startup.

## `opencode-work-mode`

Shortcut for work mode:

```bash
scripts/opencode-work-mode --dry-run
scripts/opencode-work-mode
```

## `render-file.sh`

Render one md-expand prompt file.

```bash
scripts/render-file.sh config/agent/mcp-search.md
```

## `validate-file-interp.sh`

Validate md-expand references.

```bash
scripts/validate-file-interp.sh
scripts/validate-file-interp.sh config/agent
```
