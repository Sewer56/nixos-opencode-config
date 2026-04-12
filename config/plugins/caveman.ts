/**
 * Caveman plugin — ultra-compressed communication for OpenCode.
 *
 * Injects brevity instructions into the system prompt on every LLM turn
 * so the model responds with minimal tokens while keeping full technical
 * accuracy. Mode is tracked purely in-memory.
 *
 * # Usage
 * - `/caveman`        → activate full mode (default)
 * - `/caveman lite`   → professional tight, keep articles
 * - `/caveman ultra`  → ultra-terse, abbreviations, arrows
 * - "stop caveman" / "normal mode" → deactivate
 *
 * # Public API
 * - `CavemanPlugin` — default export, consumed by OpenCode plugin loader
 */
import type { Plugin } from "@opencode-ai/plugin"
import type { Event } from "@opencode-ai/sdk"

/** Agents that receive caveman instructions. Others are unaffected. */
const ALLOWED_AGENTS = new Set(["build", "plan"])

/** Max entries in the session→agent map before oldest insertion is evicted. */
const SESSION_MAP_MAX = 256

/** Supported brevity intensity levels. */
type CavemanMode = "lite" | "full" | "ultra"

/** Shared rules appended to every mode instruction: persistence, auto-clarity, boundaries. */
const SHARED = `CAVEMAN MODE ACTIVE. ACTIVE EVERY RESPONSE. No revert after many turns. No filler drift. Still active if unsure. Off only: "stop caveman" / "normal mode".

Auto-Clarity: drop caveman for security warnings, irreversible action confirmations, multi-step sequences where fragment order risks misread, user asks to clarify. Resume after.
Boundaries: code/commits/PRs written normal.`

/**
 * Per-mode instruction strings pushed into the system prompt.
 * Only the active mode's string is injected; the rest stay in memory.
 */
const MODE_INSTRUCTIONS: Record<CavemanMode, string> = {
  lite: `Respond terse. No filler (just/really/basically/actually/simply), no hedging, no pleasantries (sure/certainly/of course/happy to). Keep articles + full sentences. Professional but tight. Short synonyms (big not extensive, fix not "implement a solution for"). Technical terms exact. Code blocks unchanged. Errors quoted exact.

${SHARED}`,

  full: `Respond terse like smart caveman. All technical substance stay. Only fluff die.
Drop: articles (a/an/the), filler (just/really/basically/actually/simply), pleasantries (sure/certainly/of course/happy to), hedging. Fragments OK. Short synonyms (big not extensive, fix not "implement a solution for"). Technical terms exact. Code blocks unchanged. Errors quoted exact.
Pattern: [thing] [action] [reason]. [next step].
Not: "Sure! I'd be happy to help you with that. The issue you're experiencing is likely caused by..."
Yes: "Bug in auth middleware. Token expiry check use < not <=. Fix:"

${SHARED}`,

  ultra: `Respond ultra-terse. Abbreviate: DB/auth/config/req/res/fn/impl. Strip conjunctions. Arrows for causality (X → Y). Drop all articles, filler, pleasantries. Fragments only. Short synonyms mandatory. Technical terms exact. Code blocks unchanged. Errors quoted exact.
Pattern: [abbr] [action] → [reason]
Not: "The database connection is failing because the authentication middleware..."
Yes: "DB conn fail → auth middleware token expiry. Fix: change < to <="

${SHARED}`,
}

/**
 * OpenCode plugin that auto-activates caveman brevity mode for specific agents.
 *
 * Injects the active mode's instruction string into the LLM system prompt
 * via the `experimental.chat.system.transform` hook and tracks mode changes
 * through user messages via the `event` hook. Only injects for agents in
 * ALLOWED_AGENTS; other agents are unaffected.
 *
 * # Hooks
 * - `chat.message` — records the session→agent mapping when a user message
 *   arrives, before the system prompt is built. This ensures the agent is
 *   known by the time `system.transform` runs, preserving prompt caching.
 * - `experimental.chat.system.transform` — pushes the active mode's instruction
 *   string into `output.system` on every LLM call; no-op when inactive or
 *   the calling agent is not in the allowlist.
 * - `event` — listens for `message.updated` events from the user to detect
 *   `/caveman [level]` commands or natural-language deactivation phrases,
 *   then updates the in-memory mode.
 */
export const CavemanPlugin: Plugin = async () => {
  let mode: CavemanMode | null = "full"
  const sessionAgent = new Map<string, string>()

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
      }
    },

    "experimental.chat.system.transform": async (
      _input: unknown,
      output: { system: string[] },
    ) => {
      if (!mode) return

      const input = _input as { sessionID?: string }
      const agent = input.sessionID ? sessionAgent.get(input.sessionID) : undefined
      if (!agent || !ALLOWED_AGENTS.has(agent)) return

      output.system.push(MODE_INSTRUCTIONS[mode])
    },

    event: async ({ event }: { event: Event }) => {
      if (event.type !== "message.updated") return

      const info = (event.properties as Record<string, unknown>).info as Record<string, unknown> | undefined
      if (!info || info.role !== "user") return

      const text = extractUserText({ properties: { info } })
      if (!text) return

      const detected = detectMode(text)
      if (detected === "off") {
        mode = null
      } else if (detected) {
        mode = detected
      }
    },
  } as unknown as Awaited<ReturnType<Plugin>>
}

/**
 * Extract plain text from a user message event payload.
 *
 * Tries `info.parts` (structured parts array) first, then falls back to
 * `info.content` (string or content-part array) for older event shapes.
 *
 * # Returns
 * - Concatenated text from all text-type parts, or empty string.
 */
function extractUserText(event: { properties: { info: Record<string, unknown> } }): string {
  const info = event.properties.info
  if (info.role !== "user") return ""

  const parts = info.parts as Array<{ type: string; text?: string }> | undefined
  if (parts) {
    return parts
      .filter((p) => p.type === "text" && typeof p.text === "string")
      .map((p) => p.text as string)
      .join(" ")
  }

  const content = info.content
  if (typeof content === "string") return content
  if (Array.isArray(content)) {
    return content
      .filter((p: Record<string, unknown>) => p.type === "text" && typeof p.text === "string")
      .map((p: Record<string, unknown>) => (p as { text: string }).text)
      .join(" ")
  }

  return ""
}

/**
 * Detect a mode-switch or deactivation command from user text.
 *
 * # Returns
 * - A `CavemanMode` when a `/caveman [level]` command is matched.
 * - `"off"` when a deactivation phrase is matched.
 * - `null` when no command is detected.
 */
function detectMode(text: string): CavemanMode | "off" | null {
  const trimmed = text.trim()
  if (/^\/caveman\s+lite\b/i.test(trimmed)) return "lite"
  if (/^\/caveman\s+ultra\b/i.test(trimmed)) return "ultra"
  if (/^\/caveman$/i.test(trimmed)) return "full"
  if (/(?:^|\s)(stop caveman|normal mode|caveman off|disable caveman)(?:\s|$)/i.test(text)) return "off"
  return null
}
