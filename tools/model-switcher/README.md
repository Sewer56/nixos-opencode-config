# opencode-model-switcher

TUI for managing model tier assignments across OpenCode agent profiles.

## Commands

```text
opencode-model-switcher [profile]
```

Launch interactive editor. Optionally pre-select a profile name.

**Keys**

| Key               | Action                                     |
| ----------------- | ------------------------------------------ |
| `←`/`→`           | Switch profile                             |
| `↑`/`↓`           | Navigate tiers                             |
| `PgUp`/`PgDn`     | Scroll affected agents                     |
| `Home`/`End`      | First/last affected agent                  |
| `Enter` / `Space` | Open model picker (fuzzy search)           |
| `v`               | Open variant picker                        |
| `s`               | Save config to disk                        |
| `a`               | Apply current profile to agent `.md` files |
| `q` / `Esc`       | Quit                                       |

In picker: type to filter, `Enter` select, `Esc` cancel.

## Config file

Stored at `~/.config/opencode/model-switcher.json`.
With XDG set, use `$XDG_CONFIG_HOME/opencode/model-switcher.json`.

```jsonc
{
  "$tierOrder": {"0": "EASY", "1": "MEDIUM", "2": "HARD", "3": "STYLE-REVIEW", "4": "CORRECTNESS-REVIEW", "5": "CODER", "6": "WRITER"},
  "normal": {
    "EASY": {"model": "provider/cheap-model", "variant": "low"},
    "MEDIUM": {"model": "provider/default-model", "variant": "medium"},
    "HARD": {"model": "provider/expensive-model", "variant": "high"},
    "STYLE-REVIEW": {"model": "provider/expensive-model", "variant": "high"},
    "CORRECTNESS-REVIEW": {"model": "provider/expensive-model", "variant": "high"},
    "CODER": {"model": "provider/default-model", "variant": "medium"},
    "WRITER": {"model": "provider/writing-model", "variant": "low"}
  },
  "work": {
    "EASY": {"model": "sewer-axonhub-work/cheap", "variant": "low"},
    "MEDIUM": {"model": "sewer-axonhub-work/default", "variant": "medium"},
    "HARD": {"model": "sewer-axonhub-work/expensive", "variant": "high"},
    "STYLE-REVIEW": {"model": "sewer-axonhub-work/expensive", "variant": "high"},
    "CORRECTNESS-REVIEW": {"model": "sewer-axonhub-work/expensive", "variant": "high"},
    "CODER": {"model": "sewer-axonhub-work/default", "variant": "medium"},
    "WRITER": {"model": "sewer-axonhub-work/writing", "variant": "medium"}
  }
}
```

- All profiles must have identical tier keys.
- `work` profile requires `sewer-axonhub-work/` provider prefix.
- Variants are `low`, `medium`, `high`, `xhigh`, or `max`.
- `$tierOrder` is optional.
- Missing tiers are discovered from profile keys and agent model tags.

## Agent files

The tool scans `.md` files in `config/agent/` and `.opencode/agent/`:

```yaml
model: provider/model-name # EASY
variant: low
```

**`a` (apply)** rewrites tagged models and following variants for this profile.

Highlight a tier to see its tagged agents, including already-matching files.
Relative paths distinguish agents with the same name in different roots.

Use `PgUp`/`PgDn` or `Home`/`End` to reach agents beyond the visible list.
The list uses leftover space only, so it hides on very short terminals.
Resize to bring it back.

## Adding a new profile

Add a key to `model-switcher.json` with the same tier set as existing profiles.

## Adding a new tier

1. Add tier key to all profiles in `model-switcher.json`.
2. Optionally update `$tierOrder` to control display order.
3. Tag model lines in agent `.md` files with `# NEWTIER`.
