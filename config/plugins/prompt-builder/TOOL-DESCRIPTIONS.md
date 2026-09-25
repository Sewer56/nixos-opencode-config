# Tool descriptions

Shortens known OpenCode V2 descriptions while keeping their useful guidance.

The plugin rewrites `shell`, `read`, `write`, `edit`, `glob`, `grep` and
`question` descriptions in the `context`, `compaction` and `generate` hooks.

Names, input schemas, parameter descriptions, permissions and implementations
stay unchanged.

`subagent` keeps its full description and available-agent list.
The V1 entry point is still a no-op.

The shorter text retains background execution and completion notices, output
truncation, image/PDF reads, directory creation, edit matching rules, literal
search and question options.

The plugin keeps the shell's runtime OS and shell name unchanged.
Worktree/session-move tools and their configuration are not changed.

## When OpenCode changes

Unknown descriptions stay intact until their contracts have been checked.

The checked descriptions come from OpenCode v2.0.16, commit
`3a103fe0aff726a4edc7492f03f7b88195d9e4c9`.

A rewrite requires the complete known description, not just the tool name.
Unknown or extended descriptions
stay intact so that new features and other plugins' instructions survive.

For shell, only the runtime `Commands run on … using ….` sentence may vary.
The plugin keeps it verbatim. Applying the plugin twice does not shorten
the text again.

To update a rewrite, compare a fresh contract dump with the versioned fixture,
check new behavior, and update the text and preservation tests together.

## Capture the advertised contracts

Set `PB_DUMP` in the process that loads the plugin to enable diagnostics.

This is usually the OpenCode server, not just a client connecting to it.
The output's parent directory must exist and be writable.

For example, with `PB_DUMP=/tmp/prompt-check/probe`, a context request writes:

- `probe.context.raw.txt` and `.final.txt`: system parts before and after
  this plugin.
- `probe.context.tools.raw.json` and `.tools.final.json`: full descriptions
  and input schemas.
- `probe.context.tools.txt`: final description/schema character counts,
  not token counts.

These are snapshots at this plugin's hook, not a capture of the final network
request. A later plugin can still change them. They describe the advertised
contract, not execution details that the tool does not document.

Dumps can contain private project instructions and tool metadata.
The plugin creates new files with owner-only permissions; existing files
retain their permissions.

Each request overwrites the same hook's files, so use a separate prefix for each
probe and avoid concurrent requests.

If the builder already handled the event, a duplicate plugin load skips the
dump so it does not overwrite the original snapshot with shortened text.

Unset `PB_DUMP` after collecting a sample. When it is unset, the plugin does
not write diagnostic files.

A write failure fails the hook: check the output directory and permissions,
or unset `PB_DUMP`.
