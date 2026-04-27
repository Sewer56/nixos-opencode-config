/**
 * Export for Analysis plugin — exports a full OpenCode session as readable Markdown.
 *
 * Registers an `export_for_analysis` tool via the `tool.register` hook (primary)
 * and `Tool.define` (fallback). The AI discovers and calls this tool naturally
 * when the user asks to export a session — no command proxy needed.
 *
 * # Usage
 * Ask the AI: "export this session for analysis" or "export session <id>"
 *
 * # Debug Logging
 * Set `EXPORT_FOR_ANALYSIS_DEBUG=1` to write logs to
 * `<plugin-dir>/.logs/export-for-analysis/debug.log`.
 *
 * # Future Enhancement
 * When the Dialog API (PR #9910) lands, this plugin can add direct user
 * interaction via `input.dialog.show()` — selecting sessions, choosing output
 * paths — without AI involvement at all.
 *
 * # Public API
 * - `ExportForAnalysisPlugin` — default export, consumed by OpenCode plugin loader
 */
import type { Plugin } from "@opencode-ai/plugin"
import type { AssistantMessage, Message, Part, Session } from "@opencode-ai/sdk"
import fs from "node:fs"
import path from "node:path"

/** Set `EXPORT_FOR_ANALYSIS_DEBUG=1` to enable standalone debug logging. */
const DEBUG = process.env.EXPORT_FOR_ANALYSIS_DEBUG === "1"

/** Standalone log directory — created lazily on first debug write. */
const LOG_DIR = path.join(
  path.dirname(new URL(import.meta.url).pathname.replace(/^\/([A-Za-z]:)/, "$1")),
  ".logs",
  "export-for-analysis",
)

/** Write a debug log line if debugging is enabled. Zero overhead when off. */
function debugLog(...args: unknown[]): void {
  if (!DEBUG) return
  fs.mkdirSync(LOG_DIR, { recursive: true })
  fs.appendFileSync(
    path.join(LOG_DIR, "debug.log"),
    args.map((a) => (typeof a === "string" ? a : JSON.stringify(a))).join(" ") + "\n",
  )
}

/** Maximum string length before truncation in tool-call input display. */
const MAX_STRING_LEN = 500

/** Date prefix for output filenames (ensures chronological sort). */
function datePrefix(): string {
  const d = new Date()
  const pad = (n: number) => String(n).padStart(2, "0")
  return `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())}`
}

/** Sanitize a session title for use as a filename component. */
function sanitizeTitle(title: string): string {
  return (
    title
      .toLowerCase()
      .replace(/[^a-z0-9]+/g, "-")
      .replace(/(^-|-$)/g, "")
      .slice(0, 60) || "untitled"
  )
}

/** Recursively truncate long strings inside a JSON-compatible value. */
function truncateStrings(obj: unknown, maxLen: number): unknown {
  if (typeof obj === "string") return obj.length > maxLen ? "[truncated]" : obj
  if (Array.isArray(obj)) return obj.map((v) => truncateStrings(v, maxLen))
  if (obj && typeof obj === "object") {
    const out: Record<string, unknown> = {}
    for (const [k, v] of Object.entries(obj)) out[k] = truncateStrings(v, maxLen)
    return out
  }
  return obj
}

/** Pretty-print tool-call input JSON with string truncation. */
function formatToolInput(input: Record<string, unknown>): string {
  return JSON.stringify(truncateStrings(input, MAX_STRING_LEN), null, 2)
}

/**
 * Format a single message (info + parts) as Markdown.
 *
 * Handles all part types defined in the SDK:
 * - `reasoning` → blockquote with 💭 Thinking header
 * - `text` → plain paragraphs
 * - `tool` → 🔧 header + pretty-printed input JSON (output excluded)
 * - `agent` → 🤖 Subagent boundary marker
 * - `step-start` / `step-finish` / `snapshot` / `patch` / `file` → skipped
 */
function formatMessage(msg: Message, parts: Part[], indent: string): string {
  const lines: string[] = []

  if (msg.role === "assistant") {
    const amsg = msg as AssistantMessage
    lines.push(`${indent}## Assistant (${amsg.modelID} / ${amsg.providerID})`)
  } else {
    lines.push(`${indent}## User`)
  }
  lines.push("")

  for (const part of parts) {
    switch (part.type) {
      case "reasoning": {
        lines.push(`${indent}> 💭 Thinking`)
        for (const line of part.text.split("\n")) {
          lines.push(`${indent}> ${line}`)
        }
        lines.push("")
        break
      }

      case "text":
        lines.push(part.text)
        lines.push("")
        break

      case "tool": {
        const state = part.state
        lines.push(`${indent}🔧 **${part.tool}** (${state.status})`)
        const input =
          state.status === "running"
            ? (state as { input?: unknown }).input
            : state.status === "completed" || state.status === "error"
              ? (state as { input: Record<string, unknown> }).input
              : undefined
        if (input && typeof input === "object" && input !== null) {
          lines.push(`${indent}\`\`\`json`)
          for (const line of formatToolInput(input as Record<string, unknown>).split("\n")) {
            lines.push(`${indent}${line}`)
          }
          lines.push(`${indent}\`\`\``)
        }
        lines.push("")
        break
      }

      case "agent":
        lines.push(`${indent}🤖 Subagent: **${part.name}**`)
        lines.push("")
        break

      case "step-start":
      case "step-finish":
      case "snapshot":
      case "patch":
      case "file":
        break
    }
  }

  return lines.join("\n")
}

/**
 * Recursively export a session and its children as Markdown.
 *
 * Fetches messages, formats each part, then recurses into child sessions.
 * Indentation increases for each nesting level.
 */
async function exportSessionTree(
  client: {
    session: {
      get(opts: { path: { id: string } }): Promise<{ data?: Session; error?: unknown }>;
      messages(opts: {
        path: { id: string };
      }): Promise<{ data?: Array<{ info: Message; parts: Part[] }>; error?: unknown }>;
      children(opts: { path: { id: string } }): Promise<{ data?: Session[]; error?: unknown }>;
    };
    tui: {
      showToast(opts: {
        body: {
          title?: string;
          message: string;
          variant: "info" | "success" | "warning" | "error";
        };
      }): Promise<unknown>;
    };
  },
  sessionId: string,
  indent: string,
): Promise<string> {
  debugLog(`exportSessionTree: id=${sessionId}`)

  const sessionRes = await client.session.get({ path: { id: sessionId } })
  if (sessionRes.error) {
    debugLog(`session.get error:`, sessionRes.error)
    return `${indent}⚠️ Failed to fetch session ${sessionId}\n`
  }
  if (!sessionRes.data) {
    debugLog(`session.get returned no data for`, sessionId)
    return `${indent}⚠️ Session data missing for ${sessionId}\n`
  }
  const session = sessionRes.data

  const messagesRes = await client.session.messages({ path: { id: sessionId } })
  if (messagesRes.error) {
    debugLog(`session.messages error:`, messagesRes.error)
    return `${indent}⚠️ Failed to fetch messages for session ${sessionId}\n`
  }
  if (!messagesRes.data) {
    debugLog(`session.messages returned no data for`, sessionId)
    return `${indent}⚠️ Messages data missing for session ${sessionId}\n`
  }
  const messages = messagesRes.data

  const lines: string[] = []
  lines.push(`${indent}# ${session.title || "Untitled Session"}`)
  lines.push("")
  lines.push(`${indent}Session ID: \`${sessionId}\``)
  lines.push("")

  for (const { info, parts } of messages) {
    lines.push(formatMessage(info, parts, indent))
  }

  const childrenRes = await client.session.children({ path: { id: sessionId } })
  if (!childrenRes.error && childrenRes.data && childrenRes.data.length > 0) {
    lines.push(`${indent}---`)
    lines.push("")
    lines.push(`${indent}## Subagent Sessions`)
    lines.push("")
    for (const child of childrenRes.data) {
      const childContent = await exportSessionTree(client, child.id, indent + "  ")
      lines.push(childContent)
    }
  } else if (childrenRes.error) {
    debugLog("session.children error:", childrenRes.error)
  }

  return lines.join("\n")
}

/**
 * Execute the full export: resolve session, traverse tree, write file, show toast.
 *
 * If no sessionID is provided, defaults to the most recently updated session.
 * Returns the output file path as the tool result string.
 *
 * @throws {Error} When no sessions exist and no sessionID is provided
 * @throws {Error} When the specified session is not found
 * @throws {Error} When I/O operations (mkdir, writeFile) fail
 */
async function executeExport(
  client: Parameters<typeof exportSessionTree>[0] & {
    session: Parameters<typeof exportSessionTree>[0]["session"] & {
      list(): Promise<{ data?: Session[]; error?: unknown }>;
    };
  },
  cwd: string,
  args: { sessionID?: string; outputPath?: string },
): Promise<string> {
  let sessionId = args.sessionID
  if (!sessionId) {
    const listRes = await client.session.list()
    if (listRes.error || !listRes.data || listRes.data.length === 0) {
      throw new Error("No sessions found and no sessionID provided")
    }
    const sorted = [...listRes.data].sort((a, b) => b.time.updated - a.time.updated)
    sessionId = sorted[0].id
    debugLog(`defaulted to most recent session: ${sessionId}`)
  }

  const sessionRes = await client.session.get({ path: { id: sessionId } })
  if (sessionRes.error) throw new Error(`Session not found: ${sessionId}`)
  if (!sessionRes.data) throw new Error(`Session data missing for ${sessionId}`)
  const session = sessionRes.data

  const outputPath =
    args.outputPath ||
    path.join(cwd, "export-for-analysis", `${datePrefix()}-${sanitizeTitle(session.title)}.md`)

  const content = await exportSessionTree(client, sessionId, "")
  fs.mkdirSync(path.dirname(outputPath), { recursive: true })
  fs.writeFileSync(outputPath, content, "utf8")

  debugLog(`wrote ${content.length} chars to ${outputPath}`)

  await client.tui
    .showToast({
      body: {
        title: "Export complete",
        message: `Session exported to ${outputPath}`,
        variant: "success",
      },
    })
    .catch((err) => { debugLog("showToast error:", err) })

  return outputPath
}

/**
 * OpenCode plugin that registers the `export_for_analysis` tool.
 *
 * The tool exports a full session (including reasoning, tool calls, and
 * recursive subagent content) as a readable Markdown file. The AI discovers
 * and calls this tool naturally — no command proxy needed.
 *
 * # Registration paths
 * - Primary: `tool.register` hook → `register()` callback
 * - Fallback: `Tool.define` on PluginInput (called at init time)
 * Both paths attempt registration with the same tool definition object;
 * the runtime determines which succeeds.
 *
 * # Hooks
 * - `tool.register` — registers the export tool with the OpenCode server
 */
export const ExportForAnalysisPlugin: Plugin = async (input) => {
  const { client, Tool, directory } = input

  /** Shared execute handler for both registration paths. */
  const execute = async (args: Record<string, unknown>): Promise<string> => {
    try {
      return await executeExport(
        client as unknown as Parameters<typeof executeExport>[0],
        directory,
        {
          sessionID: args.sessionID as string | undefined,
          outputPath: args.outputPath as string | undefined,
        },
      )
    } catch (err) {
      debugLog("execute error:", err)
      return `Error: ${err instanceof Error ? err.message : String(err)}`
    }
  }

  /** Tool definition object passed to both `register()` and `Tool.define`. */
  const toolDef = {
    id: "export_for_analysis",
    description:
      "Export a full OpenCode session as a readable Markdown file, including thinking text, tool calls (sans responses), and recursive subagent content",
    parameters: {
      type: "object" as const,
      properties: {
        sessionID: {
          type: "string" as const,
          description: "Session ID to export (defaults to the most recently updated session)",
          optional: true as const,
        },
        outputPath: {
          type: "string" as const,
          description:
            "Output file path (defaults to <cwd>/export-for-analysis/<date>-<sanitized-session-title>.md)",
          optional: true as const,
        },
      },
    },
    execute,
  }

  // Fallback: Tool.define at init time (may or may not support execute)
  try {
    Tool.define("export_for_analysis", toolDef)
    debugLog("Tool.define: registration attempted")
  } catch (err) {
    debugLog("Tool.define: failed:", err)
  }

  return {
    "tool.register": async (
      _input: unknown,
      output: { register: (tool: unknown) => void | Promise<void> },
    ) => {
      try {
        output.register(toolDef)
        debugLog("tool.register: register() called")
      } catch (err) {
        debugLog("tool.register: register() failed:", err)
      }
    },
  } as unknown as Awaited<ReturnType<Plugin>>
}
