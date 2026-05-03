/**
 * File Interpolation plugin — expands {file:...} and {env:...} in .md agent prompts.
 *
 * OpenCode already supports {file:~/.secrets/openai-key} in JSON config files,
 * but .md agent/command/mode/skill files receive no interpolation. This plugin
 * fixes that by rewriting the system prompt at LLM-call time via the
 * `experimental.chat.system.transform` hook.
 *
 * Agents and commands load into the system prompt; tokens in those files are
 * expanded before the LLM sees them. Plain-text variable references like
 * `GENERAL_RULES_PATH` are left untouched — only {file:...} and {env:...} match.
 *
 * # Supported tokens
 * - `{file:~/.secrets/key}`  — absolute or ~-relative file content
 * - `{file:./relative/path}` — relative to project directory; falls
 *   back to config directory when the file is not found
 * - `{env:VAR_NAME}`         — environment variable value
 *
 * # Raw inlining
 * Inlined file content is raw text spliced directly into the prompt; the LLM
 * sees the content, not a file path or tool call. Plain-text variable references
 * like `GENERAL_RULES_PATH` are left untouched — only {file:...} and {env:...} match.
 *
 * # Usage
 * In any .md agent file:
 * ```markdown
 * Your API key is {file:~/.secrets/openai-key}
 * Project config: {file:./config/prompt-ctx.txt}
 * Region: {env:AWS_REGION}
 * System rules: {file:./config/rules/general.md}
 * ```
 *
 * # Debug Logging
 * Set `FILE_INTERP_DEBUG=1` to write logs to
 * `config/plugins/.logs/file-interp/debug.log`. No TUI output.
 *
 * # Public API
 * - `FileInterpPlugin` — default export, consumed by OpenCode plugin loader
 */
import type { Plugin } from "@opencode-ai/plugin"
import path from "node:path"
import os from "node:os"
import fsp from "node:fs/promises"
import fs from "node:fs"

// ── Entry point ──────────────────────────────────────────────────────────────

/**
 * OpenCode plugin that expands {file:...} and {env:...} tokens in .md agent
 * system prompts.
 *
 * Captures `directory` from `PluginInput` at init time to resolve relative
 * paths. Rewrites every system prompt entry that contains tokens; no-op
 * when no tokens are present.
 *
 * # Hooks
 * - `experimental.chat.system.transform` — expands tokens in each system
 *   prompt string. No-op when no tokens are present.
 */
export const FileInterpPlugin: Plugin = async (input) => {
  const projectDir = input.directory
  debugLog(`init: projectDir=${projectDir}`)

  return {
    "experimental.chat.system.transform": async (
      _input: unknown,
      output: { system: string[] },
    ) => {
      for (let i = 0; i < output.system.length; i++) {
        const entry = output.system[i]
        const hasTokens = FILE_RE.test(entry) || ENV_RE.test(entry)
        if (!hasTokens) continue

        // reset regex lastIndex after .test()
        FILE_RE.lastIndex = 0
        ENV_RE.lastIndex = 0

        debugLog(`system[${i}]: expanding tokens (${entry.length} chars)`)
        output.system[i] = await expand(entry, projectDir)
      }
    },
  } as unknown as Awaited<ReturnType<Plugin>>
}

// ── Internals ────────────────────────────────────────────────────────────────

/**
 * Set `FILE_INTERP_DEBUG=1` in your shell env to enable debug logging.
 * Logs are written to `config/plugins/.logs/file-interp/debug.log`.
 */
const DEBUG = process.env.FILE_INTERP_DEBUG === "1"

/** Standalone log directory — created lazily on first debug write. */
const LOG_DIR = path.join(
  path.dirname(new URL(import.meta.url).pathname.replace(/^\/([A-Za-z]:)/, "$1")),
  ".logs",
  "file-interp",
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

/** OpenCode config directory — fallback base for relative {file:...} paths. */
const CONFIG_DIR = path.dirname(
  path.dirname(new URL(import.meta.url).pathname.replace(/^\/([A-Za-z]:)/, "$1")),
)

/** Regex matching {file:<path>} tokens. Captures the path portion. */
const FILE_RE = /\{file:([^}]+)\}/g

/** Regex matching {env:<var>} tokens. Captures the var name. */
const ENV_RE = /\{env:([^}]+)\}/g

/**
 * Resolve a raw token path to an absolute filesystem path.
 *
 * - `~/...`  → `$HOME/...`
 * - `./...`  → relative to `baseDir`
 * - `../...` → relative to `baseDir`
 * - other    → used as-is (assumed absolute)
 */
export function resolvePath(raw: string, baseDir: string): string {
  if (raw.startsWith("~/") || raw === "~") {
    return path.join(os.homedir(), raw.slice(1))
  }
  if (raw.startsWith("./") || raw.startsWith("../")) {
    return path.resolve(baseDir, raw)
  }
  return path.isAbsolute(raw) ? raw : path.resolve(baseDir, raw)
}

/**
 * Expand {env:VAR} and {file:path} tokens in `text`.
 *
 * Environment substitution runs first (synchronous), then file substitution
 * (async reads). Missing env vars resolve to empty string. Missing files
 * throw — this mirrors OpenCode's own `substitute()` behaviour in
 * `config/paths.ts`.
 */
export async function expand(text: string, baseDir: string): Promise<string> {
  // {env:VAR} — synchronous, replace in-place
  text = text.replace(ENV_RE, (_, varName: string) => {
    const value = process.env[varName] ?? ""
    debugLog(`env: ${varName} → ${value ? "<set>" : "<unset>"}`)
    return value
  })

  // {file:path} — collect matches, read files, splice in
  const matches = Array.from(text.matchAll(FILE_RE))
  if (!matches.length) return text

  let out = ""
  let cursor = 0

  for (const match of matches) {
    const token = match[0]
    const index = match.index!
    out += text.slice(cursor, index)

    const rawPath = match[1]
    const resolved = resolvePath(rawPath, baseDir)

    let content: string
    try {
      content = (await fsp.readFile(resolved, "utf8")).trim()
      debugLog(`file: ${token} → ${resolved} (${content.length} chars)`)
    } catch (err: unknown) {
      const e = err as NodeJS.ErrnoException
      if (e.code === "ENOENT" && (rawPath.startsWith("./") || rawPath.startsWith("../"))) {
        const configResolved = path.resolve(CONFIG_DIR, rawPath)
        if (configResolved !== resolved) {
          try {
            content = (await fsp.readFile(configResolved, "utf8")).trim()
            debugLog(`file: ${token} → ${configResolved} (${content.length} chars) [config dir fallback]`)
          } catch (err2: unknown) {
            const e2 = err2 as NodeJS.ErrnoException
            if (e2.code === "ENOENT") {
              debugLog(`file: ${token} → ${resolved} DOES NOT EXIST`)
            } else {
              debugLog(`file: ${token} → ${configResolved} READ ERROR: ${e2.message}`)
            }
            content = ""
          }
        } else {
          debugLog(`file: ${token} → ${resolved} DOES NOT EXIST`)
          content = ""
        }
      } else if (e.code === "ENOENT") {
        debugLog(`file: ${token} → ${resolved} DOES NOT EXIST`)
        content = ""
      } else {
        debugLog(`file: ${token} → ${resolved} READ ERROR: ${e.message}`)
        content = ""
      }
    }

    out += content
    cursor = index + token.length
  }

  out += text.slice(cursor)
  return out
}
