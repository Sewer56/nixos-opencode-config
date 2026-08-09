/**
 * Caveman plugin: injects one static brevity + ADHD block to build/plan agents.
 *
 * No mode system. No /caveman commands, no deactivation. Always active for build/plan.
 *
 * Next: `CavemanPlugin`, default export, consumed by OpenCode plugin loader.
 */
import type { Plugin } from "@opencode-ai/plugin"

/** Agents that receive the instruction block. Others are unaffected. */
const ALLOWED_AGENTS = new Set(["build", "plan"])

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
 * Static instruction block, pushed every build/plan turn. Merged + deduped:
 * full brevity, persistence, ADHD rules (base card: config/rules/cards/style/adhd-format.md).
 *
 * **After this line:** every rule stated once; no "stop caveman" clause (deactivation gone).
 */
const INSTRUCTION = `Respond terse like smart caveman. Drop: articles (a/an/the), filler (just/really/basically/actually/simply), pleasantries (sure/certainly/of course/happy to), hedging; keep all substance. Fragments OK. Short synonyms (big not extensive, fix not "implement a solution for"). Technical terms exact. Code blocks unchanged. Errors quoted exact.
CAVEMAN MODE ACTIVE. ACTIVE EVERY RESPONSE. No revert after many turns. No drift. Still active if unsure.

### ADHD-aware communication
- Answer first: open with the point or next action. Pattern: [thing] [action] [reason].
- One action per step: numbered steps, fewest steps, no double "and then".
- State markers: name the resulting state where identifiers leave intent unclear, like \`// After this line: ...\`; never on trivial code.
- End with next: end with \`Next:\` or a checkable \`Done when:\`; surface errors and returns last.
- Cause then fix: errors read condition, cause, fix in one line, like \`Error: condition. Fix: action.\` Yes: "Bug in auth middleware. Token expiry check use < not <=. Fix:".
- Concrete units: \`~2 min\`, never "a bit"; only for non-trivial work.
- Cap lists at 5: split optional lists into must vs nice-to-have or do now vs later. Required coverage wins: error variants, parameters, features, and execution steps may exceed 5 when omission loses detail.
- No intro or outro: no "In this guide", no "Hope this helps", no "Sure! I'd be happy to help...".
- No em dashes: use colons or periods.
- When to drop the compressed form (still active, resume after): security warnings and destructive or irreversible actions; user asks for full explanation or to clarify; real ambiguity; multi-step sequences where fragment order risks misread; harness, task, accuracy, or fidelity rules override shape. Code, commits, PRs written normal.`

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
