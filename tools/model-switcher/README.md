# opencode-model-switcher

TUI for managing model tier assignments across OpenCode agent profiles.

## Commands

```
opencode-model-switcher [profile]
```

Launch interactive editor. Optionally pre-select a profile name.

**Keys**

| Key               | Action                                     |
| ----------------- | ------------------------------------------ |
| `←`/`→`           | Switch profile                             |
| `↑`/`↓`           | Navigate tiers                             |
| `Enter` / `Space` | Open model picker (fuzzy search)           |
| `v`               | Open variant picker                        |
| `s`               | Save config to disk                        |
| `a`               | Apply current profile to agent `.md` files |
| `q` / `Esc`       | Quit                                       |

In picker: type to filter, `Enter` select, `Esc` cancel.

## Config file

Stored at `~/.config/opencode/model-switcher.json` (or `$XDG_CONFIG_HOME/opencode/model-switcher.json`).

```jsonc
{
  "$tierOrder": {"0": "EASY", "1": "MEDIUM", "2": "HARD"},
  "normal": {
    "EASY": {"model": "provider/cheap-model", "variant": "low"},
    "MEDIUM": {"model": "provider/default-model", "variant": "medium"},
    "HARD": {"model": "provider/expensive-model", "variant": "high"}
  },
  "work": {
    "EASY": {"model": "sewer-axonhub-work/cheap", "variant": "low"},
    "MEDIUM": {"model": "sewer-axonhub-work/default", "variant": "medium"},
    "HARD": {"model": "sewer-axonhub-work/expensive", "variant": "high"}
  }
}
```

- All profiles must have identical tier keys.
- `work` profile requires `sewer-axonhub-work/` provider prefix.
- Variants are `low`, `medium`, `high`, `xhigh`, or `max`.
- `$tierOrder` is optional. Missing tiers are discovered from profile keys and tagged model lines in agent files.

## Agent files

Tool scans `config/agent/` and `.opencode/agent/` for `.md` files containing tagged model lines:

```
model: provider/model-name # EASY
variant: low
```

**`a` (apply)** rewrites each tagged model and its following variant line to match the current profile.

## Adding a new profile

Edit `model-switcher.json` - add a new key with the same tier set as existing profiles.

## Adding a new tier

1. Add tier key to all profiles in `model-switcher.json`.
2. Optionally update `$tierOrder` to control display order.
3. Tag model lines in agent `.md` files with `# NEWTIER`.
