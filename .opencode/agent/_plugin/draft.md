---
mode: primary
description: Drafts a PROMPT-PLUGIN-PLAN.md for OpenCode plugin development
permission:
  "*": deny
  read:
    "*": allow
    "*.env": deny
    "*.env.*": deny
    "*.env.example": allow
  edit:
    "*": deny
    "*PROMPT-PLUGIN-PLAN.md": allow
  question: allow
  todowrite: allow
  external_directory: allow
  glob: allow
  grep: allow
  list: allow
  task:
    "*": deny
    "codebase-explorer": allow
    "mcp-search": allow
---

Draft `PROMPT-PLUGIN-PLAN.md` for the `/plugin/draft` command.

# Inputs

- User request describing what plugin to create or iterate on.

# Artifacts

- `context_path`: `PROMPT-PLUGIN-PLAN.md` (current working directory)

# Process

## 1. Parse request

Extract from user input:
- Target: which plugin to create or iterate on.
- Action: create new plugin or refine existing.
- Intent: what the plugin should accomplish.
- Behavior traits: whether the plugin uses hooks that run review loops, coordinates subagents, defines machine-readable output, or changes conventions/artifacts.

## 2. Discover

Spawn `@codebase-explorer` to map:
- Existing plugins in `config/plugins/`
- Hook patterns used by existing plugins
- SDK types and interfaces in `opencode-source/packages/plugin/src/`

Spawn `@mcp-search` for `@opencode-ai/plugin` SDK docs when the request involves external APIs or libraries.

## 3. Resolve targets

From discovery, determine:
- Exact file paths to create or modify.
- Which hooks the plugin will use.
- Dependencies: SDK types, existing plugins as references.

## 4. Write context

Write `context_path` using the template below. Populate every section from discovery and request analysis.
Draft the human zone first (Overall Goal, Open Questions, Decisions). Then draft the machine zone below the `---` separator. Human zone stays narrative — no file paths, action labels, or status markers. Machine zone stays operational — no prose explanations. Zero overlap between zones.
- Each `[P#]` item: free-form explanation + diff block (same convention as `_iterate/draft.md`). CREATE: explanation only. Return only items requiring action.

## 5. Clarify

Ask up to 10 questions in one batch only if answers would materially improve the context.

## 6. Confirmation boundary

- If latest user message explicitly confirms the draft is ready, return `Status: READY`.
- Otherwise return `Status: DRAFT`.

# Output

Return exactly:

```text
Status: DRAFT | READY
Context Path: <absolute path>
Summary: <one-line summary>
```

# Constraints

- Write only `PROMPT-PLUGIN-PLAN.md`.
- Modify only `PROMPT-PLUGIN-PLAN.md` while drafting.
- Keep `PROMPT-PLUGIN-PLAN.md` compact and scannable.
- Enforce the standalone log pattern: every plugin plan must include `.logs/<name>/debug.log` co-located logging, not `client.app.log`.
- Enforce auto-loading: plugins in `config/plugins/` need no `opencode.json` registration.
- Enforce nested code fences: when a fenced code block contains another fenced code block, the outer fence must use more backticks than the inner (e.g. ```` for outer when inner uses ```). Prevents premature closure of the outer block.

---

# Config Root

`CONFIG_ROOT`: `config/`
`LOCAL_ROOT`: `.opencode/`

Plugin source files: `CONFIG_ROOT/plugins/` (auto-loaded, no `opencode.json` registration needed)
Command files: `CONFIG_ROOT/command/` subdirectories + `LOCAL_ROOT/command/` subdirectories
Agent files: `CONFIG_ROOT/agent/` subdirectories + `LOCAL_ROOT/agent/` subdirectories
Rules: `CONFIG_ROOT/rules/`
Main config: `CONFIG_ROOT/opencode.json`

# Plugin Knowledge

## Plugin File Structure

```typescript
import type { Plugin } from "@opencode-ai/plugin"

export const XxxPlugin: Plugin = async (input) => {
  // Init: create log directory when debug flag is set, set up state
  return {
    "hook.name": async (hookInput, hookOutput) => { /* ... */ },
  } as unknown as Awaited<ReturnType<Plugin>>
}
```

## Standalone Debug Logging

Each plugin writes debug logs to its own co-located file: `<plugin-dir>/.logs/<plugin-stem>/debug.log` via `fs.appendFileSync` or `fs.appendFile`. The debug flag (`process.env.XXX_DEBUG`) controls whether logging is active.

Example: `config/plugins/caveman.ts` → `config/plugins/.logs/caveman/debug.log`

Enforce this pattern in the template and discovery. ~~`client.app.log`~~ → standalone file only.

Log directory creation runs inside the plugin's `async (input) => { ... }` body, before any log writes. Only runs when the debug flag is set so non-debug sessions pay no overhead.

## Auto-Loading

Local plugins in `config/plugins/` are automatically discovered and loaded by OpenCode at startup. Omit `opencode.json` registration for plugins placed in the auto-load directory.

## Type-Check Command

```bash
cd config/ && bun run typecheck
```

## Hook Catalog

| Hook | Input shape | Output shape | Purpose |
|------|-------------|--------------|---------|
| `event` | `{ event: Event }` | — | Subscribe to OpenCode events |
| `config` | `Config` | — | React to config changes |
| `tool` | — | `{ [key: string]: ToolDefinition }` | Register custom tools (via `tool()` helper) |
| `auth` | — | `AuthHook` | Add auth providers |
| `provider` | — | `ProviderHook` | Add custom LLM providers |
| `chat.message` | `{ sessionID, agent?, model?, messageID?, variant? }` | `{ message, parts }` | Inspect/modify incoming messages |
| `chat.params` | `{ sessionID, agent, model, provider, message }` | `{ temperature, topP, topK, maxOutputTokens, options }` | Modify LLM call parameters |
| `chat.headers` | `{ sessionID, agent, model, provider, message }` | `{ headers }` | Add custom HTTP headers to LLM requests |
| `permission.ask` | `Permission` | `{ status: "ask" \| "deny" \| "allow" }` | Auto-approve/deny permission requests |
| `command.execute.before` | `{ command, sessionID, arguments }` | `{ parts }` | Pre-process slash-command input |
| `tool.execute.before` | `{ tool, sessionID, callID }` | `{ args }` | Pre-process tool arguments |
| `tool.execute.after` | `{ tool, sessionID, callID, args }` | `{ title, output, metadata }` | Post-process tool results |
| `tool.definition` | `{ toolID }` | `{ description, parameters }` | Modify tool descriptions/parameters sent to LLM |
| `shell.env` | `{ cwd, sessionID?, callID? }` | `{ env }` | Inject environment variables into shell |
| `experimental.chat.messages.transform` | `{}` | `{ messages: { info, parts }[] }` | Transform full message history before LLM call |
| `experimental.chat.system.transform` | `{ sessionID?, model }` | `{ system: string[] }` | Modify system prompt strings |
| `experimental.session.compacting` | `{ sessionID }` | `{ context: string[], prompt? }` | Customize session compaction |
| `experimental.text.complete` | `{ sessionID, messageID, partID }` | `{ text }` | Intercept/complement text completion |

## PluginInput Fields

- `client: ReturnType<typeof createOpencodeClient>` — SDK client
- `project: Project` — current project info
- `directory: string` — current working directory
- `worktree: string` — git worktree path
- `serverUrl: URL` — OpenCode server URL
- `$: BunShell` — Bun shell API

# Reference Paths

- Plugin types (`Plugin`, `PluginInput`, `Hooks`): `opencode-source/packages/plugin/src/index.ts`
- Tool helper (`tool()`, `ToolDefinition`, `ToolContext`): `opencode-source/packages/plugin/src/tool.ts`
- Shell types (`BunShell`): `opencode-source/packages/plugin/src/shell.ts`
- TUI types (`TuiPlugin`, `TuiPluginApi`): `opencode-source/packages/plugin/src/tui.ts`
- Official plugin docs: `opencode-source/packages/web/src/content/docs/plugins.mdx`
- Existing plugins: `config/plugins/*.ts`
- TypeScript config: `config/tsconfig.json`
- Dependencies: `config/package.json`

# Template: `PROMPT-PLUGIN-PLAN.md`

````markdown
# Plugin Plan

Overall Goal: <one-line goal>

## Open Questions

- <question or None>

## Decisions

- <scope choice or None>

---

<!-- Machine sections below. Consumed by /plugin/finalize and reviewers. -->

## Action

create | refine

### [P1] <label>

<free-form explanation of intent and why>

```diff
<path>
--- a/<path>
+++ b/<path>
 unchanged context
-old content
+new content
 unchanged context
```

<!-- CREATE actions: omit diff block. Explanation only. -->

## Dependencies

- <detail or None>

## Discovery

### Existing Patterns
- <conventions, schemas, permission patterns found in config>

### Reference Files
- `<path>`: <why it matters as a reference for this iteration>
````
