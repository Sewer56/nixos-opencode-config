# prompt-builder

Replaces OpenCode's default system prompt with mine, tailored to the available
tools. Project instructions and custom agent prompts stay intact.

Unknown OpenCode prompt layout: warn and leave the request unchanged.

## Tweaks

- `PROMPT_BUILDER_KEEP_CORE=1`: keep core parts except the marked base prompt.
- `PROMPT_BUILDER_DEBUG=1`: log hook and tool information.
- `supplemental`: include additional file content in the system prompt. Set it
  in the plugin entry in `config/opencode.json`:

  ```json
  { "package": "./plugins/prompt-builder", "options": { "supplemental": ["./docs/style.md"] } }
  ```

  Paths are relative to the project directory. Unreadable files add an error to
  the prompt.

## Capture a prompt

**Captures may contain private instructions.** From this folder, run:

```sh
./scripts/dump-prompt.py
```

This runs a fresh, private OpenCode server for one real model request (which may
cost money), then saves an ignored `probe.context.report.md` here. It does not
restart or change the shared server.

Move or remove the report before probing again; the script won't overwrite it.

To combine captures you already have, pass their prefix:
`./scripts/dump-prompt.py /path/to/prefix`. Use `--hook` for another hook or
`--output` for a custom report path. The script never overwrites a report.

## Development

Requires Node.js 24+, npm, and Python 3.10+:

```sh
npm ci
npm run check
npm test
```
