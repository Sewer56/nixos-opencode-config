# path-guard

Keeps OpenCode 2 writes from escaping through symlinks. Normal writes are
untouched.

The plugin blocks write-class tool calls (`write`, `edit`, `apply_patch`,
`multiedit`) whose target resolves to a dangling link, or to a path outside
the project the config does not allow.

OpenCode 2.0.16 checks permission patterns against the path as the model
spelled it, so a symlink inside the project can point somewhere the rules
deny. This plugin resolves the real target first, then decides.

## How it decides

1. Resolve the target to its physical path. Relative paths resolve against
   the session's project directory, not the server process directory.
2. A missing final file is fine (writing a new file). A symlink that leads
   nowhere is not: it fails closed.
3. Target inside the project: allowed. OpenCode's own rules already cover it.
4. Target outside the project: must match an `allow` glob from
   `permission.external_directory` in the shared `opencode.json`.
   `deny` always wins; `ask` and unmatched paths fail closed because the
   plugin cannot show a prompt.

## Examples

Setup used below: a git repo in `/tmp/demo`, and this rule in
`opencode.json`:

```json
"external_directory": {
  "*": "ask",
  "/tmp/**": "allow",
  "/home/sewer/projects/nixos-secrets/**": "deny"
}
```

**Dangling link: refused.** The link points at a file that does not exist.

```bash
cd /tmp/demo
ln -s /tmp/demo/missing.txt link.txt
# model: "write x to link.txt"
# error: path-guard: refusing /tmp/demo/link.txt: dangling symlink at /tmp/demo/link.txt
```

No file is created. Writing a brand-new normal file (`write ok.txt`) still
works; only the broken link is refused.

**Link into a denied directory: refused.** The link exists and works, but
its real target is denied.

```bash
ln -s /home/sewer/projects/nixos-secrets/key.txt secret-link.txt
# model: "write x to secret-link.txt"
# error: path-guard: refusing secret-link.txt: canonical target
#        /home/sewer/projects/nixos-secrets/key.txt matches deny rule
#        /home/sewer/projects/nixos-secrets/**
```

The model may pass an absolute or relative path; both resolve the same way.

**Link into an allowed directory: allowed.**

```bash
mkdir -p /tmp/scratch && printf old > /tmp/scratch/out.txt
ln -s /tmp/scratch/out.txt out-link.txt
# model: "write ok to out-link.txt"  -> /tmp/scratch/out.txt now contains ok
```

The real target `/tmp/scratch/out.txt` matches `/tmp/**`, so the write runs.

## Notes

- Under OpenCode V1 this plugin loads as a no-op; the patched V1 binary
  already applies these rules in its permission engine.
- Set `PATH_GUARD_DEBUG=1` to log each decision to the server log.
- After editing `opencode.json`, restart the `opencode2` daemon
  (`pkill -f 'serve --service'`); plugin changes need a daemon restart to
  rearm hooks.
