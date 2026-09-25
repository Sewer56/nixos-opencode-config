/**
 * @module v2 - OpenCode V2 entry point for path-guard.
 *
 * V2 decodes the default export as `{id, setup}`. `setup` wires the guard
 * around `ctx.location` so relative tool paths and the worktree root resolve
 * against the session, not the daemon process.
 */
import path from "node:path"
import { canonicalPath } from "./canonical"
import { makeGuard, type ExecuteBeforeEvent } from "./guard"

type ToolDomain = {
  hook?: (
    name: string,
    handler: (event: ExecuteBeforeEvent) => Promise<void>,
  ) => Promise<unknown>
}
type LocationInfo = {
  directory?: string
  project?: { canonical?: string }
}

// The shared config sits three levels above this file's directory.
const configFile = path.resolve(import.meta.dir, "../../../opencode.json")
const debug = process.env.PATH_GUARD_DEBUG === "1"

export default {
  id: "path-guard",

  /**
   * Intercept write-class tool calls before execution. Relative targets
   * resolve against the instance directory; the worktree root is the
   * canonical project directory.
   */
  async setup(ctx: { tool?: ToolDomain; location?: LocationInfo }) {
    const directory = ctx.location?.directory ?? process.cwd()
    const root =
      ctx.location?.project?.canonical !== undefined
        ? Promise.resolve(ctx.location.project.canonical)
        : canonicalPath(directory).catch(() => path.resolve(directory))
    const guard = makeGuard(configFile, directory, root, debug)

    await ctx.tool?.hook?.("execute.before", async (event) => {
      await guard(event)
    })
  },
}
