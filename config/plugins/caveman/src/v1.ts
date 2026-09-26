/**
 * @module v1 - OpenCode V1 entry point for the caveman plugin.
 *
 * V1 calls `server()` and expects the hooks object back. The hook logic
 * tracks which agent each session uses and pushes INSTRUCTION into the
 * system prompt of allowlisted agents.
 */
import type { Plugin } from "@opencode-ai/plugin"

import { ALLOWED_AGENTS, INSTRUCTION } from "./instruction"

/** Max entries in the session->agent map before oldest insertion is evicted. */
const SESSION_MAP_MAX = 256

/**
 * Debug logging. Enable: set `CAVEMAN_DEBUG=1`. Writes via SDK client.app.log to:
 *   ~/.local/share/opencode/log/<YYYY-MM-DDTHHMMSS>.log
 * Does NOT print to TUI.
 */
const DEBUG = process.env.CAVEMAN_DEBUG === "1"

/** @internal Builds a scoped logger that writes to the opencode server log via the SDK client. */
function createLog(client: { app: { log: (opts: unknown) => Promise<unknown> } }) {
  return (...args: unknown[]) => {
    if (!DEBUG) return
    client.app
      .log({
        body: {
          service: "caveman",
          level: "info",
          message: args.map((a) => (typeof a === "string" ? a : JSON.stringify(a))).join(" "),
        },
      })
      .catch(() => {})
  }
}

/**
 * Inject static block to allowlisted agents (build, plan).
 *
 * # Hooks
 * - chat.message: record sessionID->agent; evict oldest past SESSION_MAP_MAX.
 * - experimental.chat.system.transform: push INSTRUCTION when agent allowlisted.
 */
export const CavemanPlugin: Plugin = async (input) => {
  const sessionAgent = new Map<string, string>()
  const log = createLog(input.client as { app: { log: (opts: unknown) => Promise<unknown> } })
  return {
    "chat.message": async (input: Record<string, unknown>) => {
      const sid = input.sessionID as string | undefined
      const agent = input.agent as string | undefined
      if (sid && agent) {
        if (sessionAgent.size >= SESSION_MAP_MAX) {
          const oldest = sessionAgent.keys().next().value
          if (oldest !== undefined) sessionAgent.delete(oldest)
        }
        sessionAgent.set(sid, agent)
        log(`chat.message: session=${sid} agent=${agent} mapSize=${sessionAgent.size}`)
      }
    },

    "experimental.chat.system.transform": async (
      _input: unknown,
      output: { system: string[] },
    ) => {
      const input = _input as { sessionID?: string }
      const agent = input.sessionID ? sessionAgent.get(input.sessionID) : undefined

      if (!agent) {
        log(`system.transform: no agent for session=${input.sessionID} (mapKeys=${sessionAgent.size})`)
        return
      }

      const allowed = ALLOWED_AGENTS.has(agent)
      log(`system.transform: session=${input.sessionID} agent=${agent} allowed=${allowed}`)

      if (!allowed) return

      output.system.push(INSTRUCTION)
    },
  } as unknown as Awaited<ReturnType<Plugin>>
}

export default {
  id: "caveman",
  server: CavemanPlugin,
}
