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

````
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
````

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

- `/iterate/draft` - draft an iteration context for commands and agents
- `/iterate/finalize` - convert a confirmed iteration context into revision instructions
- `/plugin/draft` - draft a plugin implementation plan
- `/plugin/finalize` - convert a confirmed plugin plan into a machine plan

> Draft and finalize agents derive a 2–3 word slug from the request
> context to name their artifacts (e.g., `PROMPT-PLAN-auth-refactor`).
> Draft and finalize for the same work use the same slug internally.

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
