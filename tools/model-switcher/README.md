# opencode-model-switcher

TUI for managing model tier assignments across OpenCode agent profiles.

## Commands

```text
opencode-model-switcher [profile]
```

Launch the editor with an optional profile.

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

Each profile maps tier names to `model` and `variant` objects.
See [the shipped presets].

- All profiles must have identical tier keys.
- Variants are `low`, `medium`, `high`, `xhigh`, or `max`.
- `$tierOrder` is optional.
- Missing tiers are discovered from profile keys and agent model tags.

## Agent files

The tool scans `.md` files in `config/agent/` and `.opencode/agent/`:

```yaml
model: provider/model-name # EASY
variant: low
```

And rewrites stuff as needed.

## Adding a new profile

Add a key to `model-switcher.json` with the same tier set as existing profiles.

## Adding a new tier

1. Add tier key to all profiles in `model-switcher.json`.
2. Optionally update `$tierOrder` to control display order.
3. Tag model lines in agent `.md` files with `# NEWTIER`.

[the shipped presets]: ../../config/model-switcher.json
