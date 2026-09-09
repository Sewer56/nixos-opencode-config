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
See [the shipped presets](../../config/model-switcher.json).

- All profiles must have identical tier keys.
- `work` profile requires `sewer-axonhub-work/` provider prefix.
- Variants are `low`, `medium`, `high`, `xhigh`, or `max`.
- `$tierOrder` is optional.
- Missing tiers are discovered from profile keys and agent model tags.

### Routing and tiers

Tier order matches the shipped presets.

- PLANNER: Code, draft, draft reviewer and migrate.
- CODER: understood Code assignments and existing implementation roles.
- Draft verifier uses CORRECTNESS-REVIEW.
- The iterate editor inherits its runtime model without an explicit tier.

Code handles obvious fixes directly and prefers cheaper research agents.
It reads essential references and consults supporting citations when needed.

Coder works without delegation or staging; Code owns integration.
Code and Coder never write concurrently and share the same lint standards.

PLANNER uses GLM-5.3 high normally and GPT-6-Astra medium at work.

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
