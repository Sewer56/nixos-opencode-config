# OpenCode Config

Self-hosted [OpenCode](https://opencode.ai) with a Home Manager module for
repeatable NixOS setup, pinned source, and multi-provider LLM routing.

> [!WARNING]
> This is a personal config - not intended for public use and not portable.
> It contains hardcoded paths, references to local secrets, and opinionated
> defaults. Feel free to study it for ideas though.

I split this off from my main system repo to keep history isolated, because I
update this often enough, making it harder to track OS/system changes.

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
│   ├── rules/             # Coding and documentation rules
│   └── plugins/           # TypeScript plugins
```

`config/` is symlinked to `~/.config/opencode` at build time (see
[default.nix](default.nix)).

## Configuration

### Commands

Slash commands live under `config/command/`.

**Plan then implement** - Draft a plan, finalize it, then implement the code:

1. `/plan/draft` - generate a plan from a prompt
2. `/plan/finalize` - validate and lock the plan
3. `/implement/plan` - write the code

Or skip planning and implement directly with `/implement/freeform`.

Split a large plan into sub-prompts with `/plan/split`.

**Iterate & Plugin**

- `/iterate/edit` - directly edit OpenCode agent/command prompts with pattern contract and compact reviewers
- `/plugin/draft` - draft a plugin implementation plan
- `/plugin/finalize` - convert a confirmed plugin plan into a machine plan

> Iteration agents derive a 2–3 word slug from the request context to name
> artifacts (e.g., `PROMPT-ITERATE-EDIT-reviewer-merge`).

**Orchestrate multi-step work** - Build a prompt pack, then run it:

1. `/orchestrator/prompt-pack` - generate orchestrator files from task descriptions
2. `/orchestrator/run` - execute the built prompt pack

**Refactor**

- `/refactor/errors` - refactor error handling
- `/refactor/modularize` - split large modules
- `/refactor/parameterize` - extract parameters
- `/refactor/reorder` - reorder declarations
- `/refactor/document` - add doc comments to source code

**Docs**

- `/docs/write` - write end-user documentation
- `/docs/review` - review end-user documentation

**Review & Audit**

- `/audit/public-api` - audit public API surface
- `/review/coderabbit` - run CodeRabbit review

**Git**

- `/commit` - generate a conventional commit
- `/summarize/pr-simple` - simple PR summary
- `/ticket/draft` - draft a company-facing issue ticket
- `/branding/draft` - draft project names and brand direction
- `/write/issue` - write a GitHub issue

### Review Topology

#### Adjudicated review (high-risk)

Correctness, audit, and implementation fidelity domains use per-domain
adjudicators. The caller dispatches the `-cached` adjudicator during normal
iterations and the `-cacheless` adjudicator for the final full-artifact audit.

```
domain-adjudicator-cached (normal iterations)
  ├── domain-a-cached  (GLM-5.1, temp 1.0)
  └── domain-b-cached  (GLM-5.1, temp 0.7)
  └── emits merged # REVIEW with Decision + IDs

domain-adjudicator-cacheless (final full-artifact audit)
  ├── domain-a-cacheless  (GLM-5.1, temp 1.0)
  └── domain-b-cacheless  (GLM-5.1, temp 0.7)
  └── emits merged # REVIEW with inline findings (no file I/O)
```

Cached: each leg writes findings and cache to separate sidecar files
(`.a.` / `.b.`). The adjudicator reads sidecar actions, merges, and writes one
canonical actions file plus one canonical cache. The caller reads the actions
file directly. Cacheless: legs return findings inline; the adjudicator parses
each leg's inline `## Findings` and emits merged findings inline. No file I/O.

Every autonomous workflow runs a final cacheless audit before returning READY
or SUCCESS.

Both legs currently use GLM-5.1 (A at temp 1.0, B at temp 0.7) because
DeepSeek-V4-Pro is unreliable right now. Once it stabilizes, the intended
setup is GLM + DeepSeek - model diversity beats temperature diversity.

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
other's output or the canonical cache. Cacheless legs return findings inline -
no sidecar files, no file I/O.

Why isolate? Shared cache lets Leg B see Leg A's findings before forming its
own judgment; agreeableness bias (LLMs reject invalid findings <25% of the
time) turns B into a rubber stamp. Cacheless extends this further: the
final-check audit leg never sees prior review state, eliminating anchoring
from cached observations.

Relevant:
- SWR-Bench (arxiv 2509.01494) - multi-review aggregation: 15.25% to 21.91% F1
- CodeAgent (EMNLP 2024) - supervisory QA-Checker patterns map well to adjudicators
- Beyond Majority Voting (OpenReview) - aggregation should not be simple voting
- HCCA (arxiv 2603.21454) - information restriction is the necessary condition for effective multi-LLM verification
- Agreeableness Bias (OpenReview) - TNR below 25% in LLM judges
- Anchors in the Machine (arxiv 2511.05766) - anchoring bias cannot be instructed away
- Behavioral Entanglement (arxiv 2604.07650) - shared evaluative context reintroduces model coupling

### Rules

Coding and documentation rules live under `config/rules/`. They enforce
minimal diffs, descriptive naming, proper documentation, test
parameterization, performance guidelines, and error handling patterns.

### Plugins

Three TypeScript plugins extend OpenCode's behavior:

#### Caveman (`caveman.ts`)

Makes `build` and `plan` agent responses terse. Activate with `/caveman`,
deactivate with "stop caveman". Code blocks and commit messages are
unaffected.

#### RTK - Rust Token Killer (`rtk.ts`)

Delegates shell-command rewriting to the `rtk` binary for token savings.
Intercepts `bash`/`shell` tool calls before execution and rewrites them
through `rtk rewrite`. Requires `rtk >= 0.23.0` in `$PATH`.

#### File Interpolation (`file-interp.ts`)

Expands `{file:...}` and `{env:...}` tokens in `.md` agent prompts. Useful
for injecting secrets or project-specific context without hardcoding them in
prompt files. Supported tokens:

- `{file:~/.secrets/key}` - absolute or `~`-relative file content
- `{file:./relative/path}` - relative to project directory
- `{env:VAR_NAME}` - environment variable value

### Permissions

All agents, tools, and MCP servers are disabled by default - everything is
opt-in rather than OpenCode's default opt-out. Permissions are set in
`opencode.json` and in agent header files (frontmatter `permission` blocks).

### MCP Servers

Only enabled for specific agents (see `opencode.json` permissions).

| Server | Type | Purpose |
|---|---|---|
| GitHub | Docker | GitHub API (token from `~/.secrets/github-token`) |
| Context7 | NPX (Node Package Execute) | Library documentation lookup |
| DeepWiki | Remote SSE (Server-Sent Events) | Repository documentation analysis |
| Discord | Docker | Discord API (token from `~/.secrets/discord-token`) |

### TUI

`config/tui.json` customizes the terminal UI:

- **Theme**: Catppuccin
- **Scroll speed**: 1
- **Keybind**: `Ctrl+[` opens the command list
- **Plugin**: `oc-tps@latest` (TPS - tokens per second)

## Building OpenCode

I use a local self-built copy of OpenCode, pinned as a submodule in
`opencode-source/`.

This is a fork that replaces the default ~8.5k system prompt with a minimal
~2.5k one. The prompt is conditionally assembled based on which tools are
available - cross-tool instructions are omitted when the tool isn't present.
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
| `opencode` | Wrapper script - runs the self-built binary with `OPENCODE_ENABLE_EXA=1` set; defaults to `opencode .` |
| `opencode-build` | Builds the forked OpenCode from source |
| `coderabbit-cli` | CodeRabbit review tool (from `llm-agents` flake input) |
| `nodejs` / `yarn` / `bun` | Runtime for MCP servers and plugin development |
| `docker` | Container runtime for GitHub and Discord MCP servers |
| `typescript` / `go` | Language toolchains for local development |

### Symlinks

- `~/.config/opencode` → this repo's `config/` directory
- `~/opencode` → this repo's root (convenient access)
