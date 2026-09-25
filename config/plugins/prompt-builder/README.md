# prompt-builder

Replaces OpenCode 2's default base prompt with sections that describe only
the tools the request can actually call.

OpenCode 2 sends one generic base prompt regardless of which tools an agent
or model exposes.

This plugin reads each request's tool snapshot. It builds `# Environment`
with the working directory and platform, then `# Tool Usage Guidelines`
for the tools present in that request.

Restricted agents never get guidance for tools they cannot call.

Cross-tool rules also follow the tool set. `apply_patch` suppresses the
`edit`/`write` sections, and read-before-edit wording requires both tools.
The subagent section names only the search tools the agent has.

The plugin recognizes names from both generations: `bash`/`shell`,
`task`/`subagent`, and `apply_patch`/`patch`.

## Example

A request advertising `read`, `edit`, `glob` and `shell` turns these system
parts:

```text
[0] You are an AI agent running in OpenCode, ...   (default base prompt)
[1] You are a focused code editor.
```

into:

```text
[0] # Environment
    Working directory: /home/user/project
    Platform: linux
[1] # Tool Usage Guidelines
    ## Common Rules
    ... (prefer glob/grep/read/edit over shell; read before edit)
    ## `Read` Tool
    ...
    ## `Edit` Tool
    ...
[2] You are a focused code editor.
```

The part containing the base-prompt marker is removed (`replaced`). When no
part matches, the sections are prepended and nothing is deleted
(`prepended`), so agents with their own `system` prompt keep it.

## Options

`supplemental` (V2 only): file paths whose content is appended as
`# Supplemental Context` sections, one `##` per file named by basename
without extension.

```json
"plugins": {
  "./plugins/prompt-builder": { "supplemental": ["./docs/style.md"] }
}
```

## Notes

- Under OpenCode V1 this plugin loads as a no-op; the patched V1 binary
  already builds tool-conditional prompts in source.
- Set `PROMPT_BUILDER_DEBUG=1` to log the chosen strategy and section count
  for each request.
