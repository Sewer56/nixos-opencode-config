/**
 * @module v2 - OpenCode V2 entry point for the caveman plugin.
 *
 * V2 decodes the default export as `{id, setup}`. `setup` registers a
 * `context` session hook that pushes INSTRUCTION for allowlisted agents.
 */
import { ALLOWED_AGENTS, INSTRUCTION } from "./instruction"

type SessionDomain = {
  hook?: (kind: string, fn: (event: Record<string, unknown>) => void) => Promise<unknown>
}

export default {
  id: "caveman",

  async setup(ctx: { session?: SessionDomain }) {
    await ctx.session?.hook?.("context", (event) => {
      const agent = event.agent
      if (typeof agent !== "string" || !ALLOWED_AGENTS.has(agent)) return
      const system = event.system as Array<{ type: string; text: string }> | undefined
      if (!Array.isArray(system)) return
      // OpenCode may load this plugin twice (config array plus directory
      // discovery); push only when the block is not already present.
      const present = system.some((part) => part.text?.includes("CAVEMAN MODE ACTIVE"))
      if (!present) system.push({ type: "text", text: INSTRUCTION })
    })
  },
}
