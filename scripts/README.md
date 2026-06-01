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

`model-tiers.json` intentionally has no comments because it is strict JSON.
Schema by convention:

```json
{
  "normal": { "LOW": "provider/model", "MED": "provider/model", "HIGH": "provider/model" },
  "work": { "LOW": "sewer-axonhub-work/model", "MED": "sewer-axonhub-work/model", "HIGH": "sewer-axonhub-work/model" }
}
```

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
