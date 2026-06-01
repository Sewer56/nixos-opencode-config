# OpenCode Config

Self-hosted [OpenCode](https://opencode.ai) with a Home Manager module for
repeatable NixOS setup, pinned source, and multi-provider LLM routing.

> [!WARNING]
> This is a personal config — not intended for public use and not portable.
> It contains hardcoded paths, references to local secrets, and opinionated
> defaults. Feel free to study it for ideas though.

I split this off from my main system repo to keep history isolated; I
update it often enough that tracking OS/system changes becomes hard.

## Directory Structure

```
.
├── default.nix            # Home Manager module (packages, symlinks)
├── opencode-source/       # Pinned OpenCode source (submodule)
├── config/                # Active OpenCode configuration (symlinked)
│   ├── opencode.json      # Provider, agent, MCP, and permission config
│   ├── tui.json           # TUI theme and keybind overrides
│   ├── agent/             # Agent prompt definitions
│   ├── command/           # Slash-command definitions
│   ├── doc/               # Supplementary workflow docs
│   ├── rules/             # Coding and documentation rules (cards + groups)
│   └── plugins/           # TypeScript plugins
├── plugins/               # Local plugin source packages
├── scripts/               # Utility scripts
├── tools/                 # Workflow tools (Python and Go)
└── AGENTS.md              # Repo-level agent instructions
```

`config/` is symlinked to `~/.config/opencode` at build time (see
[default.nix](default.nix)).

## Configuration

### Commands

Slash commands live under `config/command/`.

**Plan then implement** — draft a plan, finalize it, then write the code:

1. `/plan/draft` — generate a plan from a prompt
2. `/plan/finalize` — validate and lock the plan
3. `/plan/convert-to-draft` — convert current context into a draft plan
4. `/implement/plan` — write the code from the finalized plan

Or skip planning and implement directly with `/implement/freeform`.

Clean up after an implementation with `/implement/cleanup-diff`.

**Refactor**

- `/refactor/errors` — refactor error handling
- `/refactor/modularize` — split large modules
- `/refactor/parameterize` — extract parameters
- `/refactor/reorder` — reorder declarations
- `/refactor/document` — add doc comments to source code

**Docs**

- `/docs/write` — write end-user documentation
- `/docs/review` — review end-user documentation

**Review & Audit**

- `/audit/public-api` — audit public API surface
- `/audit/public-api-targeted` — audit a specific API subset
- `/review/coderabbit` — run CodeRabbit review

**Git & Issues**

- `/commit/main` — generate a conventional commit
- `/summarize/pr-simple` — simple PR summary
- `/write/issue` — write a GitHub issue

**Other**

- `/branding/draft` — draft project names and brand direction for a given folder path (local)
- `/workflow/optimize` — optimize workflow patterns

### Review Topology

#### Adjudicated review (high-risk)

Correctness, audit, and implementation fidelity domains use per-domain
adjudicators (review processes that merge findings from multiple reviewers).
The caller dispatches the `-cached` adjudicator during normal
iterations and the `-cacheless` adjudicator for the final full-artifact audit.

```
domain-adjudicator-cached (normal iterations)
  ├── domain-a-cached  (GLM-5.1, temp 1.0)
  ├── domain-b-cached  (GLM-5.1, temp 0.7)
  ├── emits merged # REVIEW with Decision + IDs

domain-adjudicator-cacheless (final full-artifact audit)
  ├── domain-a-cacheless  (GLM-5.1, temp 1.0)
  ├── domain-b-cacheless  (GLM-5.1, temp 0.7)
  ├── emits merged # REVIEW with inline findings (no file I/O)
```

Cached: each leg (A and B) writes findings and cache to separate sidecar files
(`.a.` / `.b.`). The adjudicator reads sidecar actions, merges, and writes one
canonical actions file plus one canonical cache. The caller reads the actions
file directly. Cacheless: legs return findings inline; the adjudicator parses
each leg's inline `## Findings` and emits merged findings inline. No file I/O.

Every autonomous workflow runs a final cacheless audit before returning READY
or SUCCESS.

Both legs currently use GLM-5.1 (A at temp 1.0, B at temp 0.7) because
DeepSeek-V4-Pro is unreliable right now. Once it stabilizes, the intended
setup is GLM + DeepSeek — model diversity beats temperature diversity.

#### Single review (low-risk)

Tests, performance, docs, branding, and placement domains use one reviewer
with `-cached` and `-cacheless` variants. No adjudicator.

- **Cached**: writes findings to an actions file, cache to a separate cache
  file. Caller reads the actions file directly.
- **Cacheless**: returns findings inline. No file I/O.

Re-review reads the canonical cache from the prior round, trusts it for
unchanged steps, and only inspects what changed.

#### Cache isolation

Each cached reviewer leg writes to its own sidecar cache. Neither leg sees the
other's output or the canonical cache. Cacheless legs return findings inline —
no sidecar files, no file I/O.

Why isolate? Shared cache (auxiliary review state) lets Leg B see Leg A's
findings before forming its
own judgment; agreeableness bias (LLMs reject invalid findings <25% of the
time) turns B into a rubber stamp. Cacheless extends this further: the
final-check audit leg never sees prior review state, eliminating anchoring
from cached observations.

Relevant:
- SWR-Bench (arxiv 2509.01494) — multi-review aggregation: 15.25% to 21.91% F1
- CodeAgent (EMNLP 2024) — supervisory QA-Checker patterns map well to
  adjudicators
- Beyond Majority Voting (OpenReview) — aggregation should not be simple voting
- HCCA (arxiv 2603.21454) — information restriction is the necessary condition
  for effective multi-LLM verification
- Agreeableness Bias (OpenReview) — TNR below 25% in LLM judges
- Anchors in the Machine (arxiv 2511.05766) — anchoring bias cannot be
  instructed away
- Behavioral Entanglement (arxiv 2604.07650) — shared evaluative context
  reintroduces model coupling

### Rules

Coding and documentation rules live under `config/rules/`. Rules are organized
as **cards** (judgment definitions) grouped into **groups** (the
public import API). Agents and commands import groups, not individual cards.

### Plugins

Three TypeScript plugins extend OpenCode's behavior:

#### Caveman (`caveman.ts`)

Makes `build` and `plan` agent responses terse. Three intensity modes:

- `/caveman` — full mode (drop articles, use fragments)
- `/caveman lite` — professional tight, keep articles
- `/caveman ultra` — ultra-terse, abbreviations, arrows

Deactivate with "stop caveman" or "normal mode". Code blocks and commit
messages are unaffected.

#### Markdown Expand (`md-expand.ts`)

Re-exports the local `opencode-plugin-md-expand` package. Expands
`{file:...}` and `{env:...}` tokens in `.md` agent prompts. Useful for
injecting secrets or project-specific context without hardcoding them in
prompt files. Supported tokens:

- `{file:~/.secrets/key}` — absolute or `~`-relative file content
- `{file:./relative/path}` — relative to project directory
- `{env:VAR_NAME}` — environment variable value

#### RTK — Rust Token Killer (`rtk.ts`)

Delegates shell-command rewriting to the `rtk` binary for token savings.
Intercepts `bash`/`shell` tool calls before execution and rewrites them
through `rtk rewrite`. Currently disabled (`rtk.ts.bak`) due to bugs.

### Permissions

All agents, tools, and MCP servers are disabled by default — everything is
opt-in rather than OpenCode's default opt-out. Set permissions in
`opencode.json` and in agent header files (frontmatter `permission` blocks).

### MCP Servers

Only enabled for specific agents (see `opencode.json` permissions).

| Server | Transport | Purpose |
|---|---|---|
| GitHub | Local (Docker) | GitHub API (token from `~/.secrets/github-token`) |
| Context7 | Local (NPX) | Library documentation lookup |
| DeepWiki | Remote (SSE) | Repository documentation analysis |
| Discord | Local (Docker) | Discord API — **disabled** |

### TUI

`config/tui.json` customizes the terminal UI:

- **Theme**: Catppuccin
- **Scroll speed**: 1
- **Keybind**: `Ctrl+[` opens the command list
- **Plugin**: `oc-tps@latest` (tokens per second)

### Provider

A custom provider `sewer-axonhub` (OpenAI-compatible) routes to various
providers. Configured models include GLM-5-Turbo, GLM-5.1, GLM-5V-Turbo,
MiniMax-M2.7, Step 3.7 Flash, DeepSeek V4 Flash, and DeepSeek V4 Pro. Small
model (used for titles) is Step 3.7 Flash.

### Model tiers

Agent model tiers live in `scripts/model-tiers.json`, beside the wrapper that
runs the Go TUI/CLI. Agent files opt in with frontmatter markers:

```yaml
model: sewer-axonhub/MiniMax-M3 # MED
```

The tool rewrites only the model token and preserves `# LOW`, `# MED`, or
`# HIGH` comments. Unmarked `model:` lines are left untouched.

```bash
scripts/opencode-model-tiers              # TUI
scripts/opencode-model-tiers status
scripts/opencode-model-tiers apply normal --dry-run
scripts/opencode-model-tiers apply normal
scripts/opencode-work-mode --dry-run
scripts/opencode-work-mode
scripts/opencode-model-tiers set normal MED sewer-axonhub/MiniMax-M3
```

The TUI reads choices from `opencode models`, supports typed filtering, previews
file changes, saves `scripts/model-tiers.json`, and can apply the selected profile.
Work mode is guarded to only use `sewer-axonhub-work/*` models.

Extra CLI helpers:

```bash
scripts/opencode-model-tiers models
scripts/opencode-model-tiers models --work
scripts/opencode-model-tiers configure work
```

The Go app can also be run through the root flake:

```bash
nix run .#opencode-model-tiers -- status
nix run .#opencode-model-tiers
nix run .#opencode-work-mode -- --dry-run
```

Root `direnv` support is enabled through `.envrc` / `flake.nix`; run
`direnv allow` after cloning or changing the dev shell.

## Building OpenCode

I use a local self-built copy of OpenCode, pinned as a submodule in
`opencode-source/`.

This is a fork that replaces the default ~8.5k system prompt with a minimal
~2.5k one. The prompt is conditionally assembled based on which tools are
available — cross-tool instructions are omitted when the tool isn't present.
Optional supplemental sections (e.g. git workflow, GitHub CLI) can be enabled
per-agent.

```bash
# Build from source
opencode-build

# Run (defaults to current directory)
opencode

# Run with arguments
opencode /path/to/project
```

## Nix Module

`default.nix` provides two shell wrappers and installs the dependencies
OpenCode needs at runtime:

| Package | Purpose |
|---|---|
| `opencode` | Wrapper — runs the self-built binary with `OPENCODE_ENABLE_EXA=1` set; defaults to `opencode .` |
| `opencode-build` | Builds the forked OpenCode from source (`bun install` + `bun run build --single`) |
| `coderabbit-cli` | CodeRabbit review tool (from `llm-agents` flake input) |
| `nodejs` / `yarn` / `bun` | Runtime for MCP servers and plugin development |
| `docker` | Container runtime for GitHub and Discord MCP servers |
| `typescript` / `go` | Language toolchains for local development |

### Symlinks

- `~/.config/opencode` → this repo's `config/` directory
- `~/opencode` → this repo's root (convenient access)
