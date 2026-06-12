# opencode-model-tiers

TUI for managing model tier assignments across OpenCode agent profiles.

## Commands

```
opencode-model-tiers [profile]
```

Launch interactive editor. Optionally pre-select a profile name.

**Keys**

| Key               | Action                                     |
| ----------------- | ------------------------------------------ |
| `←`/`→`           | Switch profile                             |
| `↑`/`↓`           | Navigate tiers                             |
| `Enter` / `Space` | Open model picker (fuzzy search)           |
| `s`               | Save config to disk                        |
| `a`               | Apply current profile to agent `.md` files |
| `q` / `Esc`       | Quit                                       |

In picker: type to filter, `Enter` select, `Esc` cancel.

## Config file

Stored at `~/.config/opencode/model-tiers.json` (or `$XDG_CONFIG_HOME/opencode/model-tiers.json`).

```jsonc
{
  "$tierOrder": {"0": "LOW", "1": "MED", "2": "HIGH"},   // optional; auto-discovered
  "normal": {
    "LOW": "provider/cheap-model",
    "MED": "provider/default-model",
    "HIGH": "provider/expensive-model"
  },
  "work": {
    "LOW": "sewer-axonhub-work/cheap",
    "MED": "sewer-axonhub-work/default",
    "HIGH": "sewer-axonhub-work/expensive"
  }
}
```

- All profiles must have identical tier keys.
- `work` profile requires `sewer-axonhub-work/` provider prefix.
- `$tierOrder` is optional. Missing tiers are discovered from profile keys + `# LOW` / `# MED` / `# HIGH` tags in agent files.

## Agent files

Tool scans `config/agent/` and `.opencode/agent/` for `.md` files containing tagged model lines:

```
model: provider/model-name # LOW
model: other/other-model   # MED keep comment
```

**`a` (apply)** rewrites these lines to match the current profile's tier assignments.

## Adding a new profile

Edit `model-tiers.json` - add a new key with the same tier set as existing profiles.

## Adding a new tier

1. Add tier key to all profiles in `model-tiers.json`.
2. Optionally update `$tierOrder` to control display order.
3. Tag model lines in agent `.md` files with `# NEWTIER`.
