# V1 removal plan (cutover checklist)

Run these when V2 becomes the daily driver. Do not execute while V1 is
still in use. First repeat the V2 plugin and permission smoke tests.
Check items off as done.

## config/opencode.json

- [ ] Drop local plugins from the `plugin` array (V2 auto-discovers
      `config/plugins/` directories; the array only exists for V1) — this
      also removes the double-registration under V2.
- [ ] Remove top-level `subagent_depth` (keep `experimental.subagent_depth`).
- [ ] Remove `compaction.prune` (no V2 equivalent; tune `compaction.keep.tokens`
      instead, default 15000 vs V1 `reserved` 12000).
- [ ] Remove `permission.doom_loop` (not enforced in V2; accepted loss).
- [ ] Remove `experimental.disable_paste_summary` (omitted by V2).
- [ ] Rename `autoupdate` to `update: "disable"`.
- [ ] Convert `permission` map to native `permissions` ordered array
      (bash→shell, task→subagent, write/patch→edit; MCP actions
      `<server>_<tool>`; keep `github_*`/`context7_*`/`deepwiki_*` denies).
- [ ] Rename `plugin` to `plugins`; tuples already unused.
- [ ] Decide `provider.*.models.*.reasoning` (V2 omits the flag; verify
      reasoning-effort variants still activate, else use V2-native model
      settings).
- [ ] Optional: convert providers to native (`npm` → `package:
      "aisdk:@ai-sdk/openai-compatible"`, variants object → array).
- [ ] Keep `"opencode-axonhub-tracing"` only after its own repo ships a
      V2 port (`session.hook("model.request")`).

## Plugins (drop the V1 halves)

- [ ] caveman, path-guard, md-expand, models-discovery: remove `server()`
      from the default export and the `@opencode-ai/plugin` type imports.
- [ ] md-expand: drop the v1 hook implementation and V1-only tests; decide
      npm publish handling for `main: ./src/index.ts` (build step or
      exports map for the published package).
- [ ] Remove `dist` build expectations; plugin deps hook keeps bun-installing.

## Repo and Nix

- [ ] Remove `opencode-source/` submodule, the `production` fork, and
      `/migrate` + `migrate-planner` agents (V1-line rebase workflow).
- [ ] Remove `opencodeScript`/`opencode-build` from flake.nix; make the V2
      binary the default `opencode` command (keep an `opencode1` alias if
      needed during transition).
- [ ] Revisit tools: `opencode-sessions` (V1 sqlite; V2 has a new DB),
      `opencode-yolo-mode` (V1 permission shapes), `opencode-model-switcher`
      (V1 model keys) — port or retire.
- [ ] Validator: switch grammar checks from V1 permission map to the V2
      ordered array once config is converted; update
      `CONFIG_PATH_FORMS` logic if the ~/opencode symlink stays.
- [ ] Remove V1-only gitignore/runtime leftovers: `config/tui.json`,
      `config/model-switcher.json` (keep `cli.json` tracked;
      `service.json` is ignored).
- [ ] Update README/AGENTS to drop the coexistence notes.

## Upstream follow-ups (not blockers)

- [ ] File: config/plugin-list edits disarm all plugins until the daemon
      restarts (`kill` the `opencode2 serve --service` process after any
      opencode.json change; sessions in between run unguarded).
- [ ] File: V2 config watcher fails inotify on the symlinked
      `~/.config/opencode` ("Not a directory") — hot reload of the global
      config may be impaired.
- [ ] File: V2 docs do not document the `server.ts` directory-entrypoint
      convention that 2.0.16 actually requires for local plugins.
