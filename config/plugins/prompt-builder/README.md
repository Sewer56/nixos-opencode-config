# prompt-builder

Build OpenCode prompts for the tools available in each request.

The plugin adds the working directory, platform and tool guidance to requests.
An agent with only `read` and `glob` gets guidance for those tools, not for
tools it cannot call.

It also shortens known tool descriptions without changing their input schemas,
permissions or implementations.

## How it works

`server.ts` registers hooks for `context`, `compaction` and `generate` requests.
Each hook reads the request's tool list and builds these sections:

- **Environment:** working directory and platform. Always included.
- **Tool Usage Guidelines:** rules for available tools. Omitted when none apply.
- **Supplemental Context:** optional text from files you configure.

By default, the plugin removes known OpenCode prompt parts: the base prompt,
model information, environment block, date and worktree reminder.

It keeps project instructions attached to the environment block. It also keeps
custom agent prompts and other plugins' sections unless they match a known part.
The new sections go first.

Set `PROMPT_BUILDER_KEEP_CORE=1` before starting OpenCode to keep the extra core
parts. In that mode, the plugin removes only the part containing the default
base-prompt marker. Without a marker, it keeps every part.

Applying the plugin twice to the same request does not duplicate the sections.

Tool aliases select the same guidance: `bash`/`shell`, `task`/`subagent` and
`apply_patch`/`patch`. A patch tool suppresses the `edit` and `write` sections.
Read-before-edit rules appear only when both tools are available.

### Shorter tool descriptions

The plugin shortens known descriptions for `shell`, `read`, `write`, `edit`,
`glob`, `grep` and `question`.

It keeps `subagent` descriptions and their agent lists unchanged.

A rewrite requires the complete known description, not just a matching tool
name. The plugin leaves unfamiliar or extended descriptions alone.

The shell's runtime OS and shell-name sentence may vary; the plugin preserves
it exactly. Repeated calls do not shorten the descriptions again.

The reference text is from OpenCode 2.0.16, commit
`3a103fe0aff726a4edc7492f03f7b88195d9e4c9`, saved in
`test/fixtures/v2.0.16-descriptions.json`.

When OpenCode changes, compare a fresh capture with that fixture. Check any new
behavior before updating the matching text, shorter descriptions and
preservation tests together.

## Supplemental files

Pass file paths in the plugin's `supplemental` option. In
`config/opencode.json`, replace the `"./plugins/prompt-builder"` entry in
the `plugins` array with:

```json
{ "package": "./plugins/prompt-builder", "options": { "supplemental": ["./docs/style.md"] } }
```

Paths are relative to the project directory supplied by OpenCode, or the
process's working directory if none is supplied. Each file becomes a subsection
named after its basename without the extension: `style.md` becomes `## style`.

`{{env:NAME}}` tokens use the server's environment; unset variables become empty
strings, as do `{{arg:key}}` tokens because this plugin supplies no template
arguments.

The plugin does not expand file includes inside supplemental files.
An unreadable file adds an error line to the prompt instead of failing the
request.

## Development and editor setup

You need Node.js 24 or later with npm, and Python 3.10 or later for the report
script and its tests.
Open a terminal in this directory and run:

```sh
npm ci
npm run check
npm test
```

`npm ci` installs the exact development packages in `package-lock.json`.
`npm run check` checks TypeScript without generating JavaScript. `npm test` runs
both the Node and Python tests.

To run one test suite:

```sh
node --test test/*.test.ts
python3 -m unittest discover -s test -p '*_test.py'
```

Open this directory in an editor with TypeScript support; it should pick up
`tsconfig.json` and the installed types automatically.

In VS Code, open a `.ts` file, run **TypeScript: Select TypeScript Version**
from the command palette and choose **Use Workspace Version**. Other editors
can point their TypeScript integration at `node_modules/typescript`.

If the editor cannot find `node:test`, `process` or `node:fs/promises`, run
`npm ci` here and restart its TypeScript service.

`@types/node` supplies editor types only, `node:` imports use Node.js's
built-in APIs, and no `fs` or `path` package is needed.

There is no build step: Node 24 runs this plugin's TypeScript directly, and
OpenCode loads `server.ts`. Type-check anyway; Node strips type annotations
without checking them.

## Capture prompts and tool text

Use `PB_DUMP` to capture prompts and tool text before and after the plugin runs.

These are snapshots at this plugin's hook, not the final network request.
Later plugins may still change the request.

**Captures can contain private project instructions and tool metadata.**

The plugin creates new capture files with owner-only permissions. Existing files
keep their permissions, so use a new private directory for each probe.

1. Create a capture directory in your terminal:

   ```sh
   capture_dir=$(mktemp -d /tmp/prompt-builder.XXXXXX)
   export PB_DUMP="$capture_dir/probe"
   ```

2. Start the OpenCode server from that environment, then make a request.

   The variable must reach the process that loads this plugin, not just a client
   connecting to an already-running server. For a managed service, set it in the
   service environment and restart the service instead.

3. Combine the capture into one Markdown file:

   ```sh
   python3 scripts/dump-prompt.py "$PB_DUMP"
   ```

   By default this reads the `context` capture and creates
   `$PB_DUMP.context.report.md`. Select another hook or report path with:

   ```sh
   python3 scripts/dump-prompt.py "$PB_DUMP" --hook generate --output "$capture_dir/generate.md"
   ```

4. Unset `PB_DUMP` and restart the server without it when you finish.

   For a managed service, also remove the variable from its environment.
   Unsetting it in your terminal does not change an already-running server.

The report script is one Python file with no extra packages. It combines the
five capture files without contacting OpenCode or changing them, creates an
owner-only report, and refuses to overwrite an existing file or symlink.

### Capture files and troubleshooting

For a prefix of `/tmp/probe`, a `context` request writes:

| File                             | Contents                                                    |
| -------------------------------- | ----------------------------------------------------------- |
| `probe.context.raw.txt`          | System prompt before the plugin                             |
| `probe.context.final.txt`        | System prompt after the plugin                              |
| `probe.context.tools.raw.json`   | Original tool descriptions and input schemas                |
| `probe.context.tools.final.json` | Final tool descriptions and input schemas                   |
| `probe.context.tools.txt`        | Final description/schema character counts, not token counts |

Each request overwrites the files for its hook. Use a separate prefix for each
probe, avoid concurrent requests, and wait for the request to finish before
building a report.

A duplicate plugin load skips the capture so it cannot replace the original
snapshot with already-shortened text.

- **No capture files:** check the server's environment, then make a request that
  uses the selected hook.
- **Capture write fails:** the hook fails too. Create the parent directory and
  check its permissions, or restart the server without `PB_DUMP`.
- **Report input is missing or unreadable:** check the prefix and hook.
  The script reads all five UTF-8 files before writing a report.
- **Report already exists:** choose a new `--output` path. If a write failed,
  remove any partial report before retrying.

Set `PROMPT_BUILDER_DEBUG=1` before starting OpenCode to log the strategy,
section count and tool list.
