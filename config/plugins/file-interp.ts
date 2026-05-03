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
 * - `expand`, `resolvePath` — exported for tests and benchmarks
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
  if (DEBUG) debugLog(`init: projectDir=${projectDir}`)

  return {
    "experimental.chat.system.transform": async (
      _input: unknown,
      output: { system: string[] },
    ) => {
      for (let i = 0; i < output.system.length; i++) {
        const entry = output.system[i]
        if (!hasExpandableToken(entry)) continue

        if (DEBUG) debugLog(`system[${i}]: expanding tokens (${entry.length} chars)`)
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

/** Standalone log file path. */
const LOG_FILE = path.join(LOG_DIR, "debug.log")

/** Avoid repeated mkdir work when debug logging is enabled. */
let logDirReady = false

/** Write a debug log line if debugging is enabled. Zero overhead when off. */
function debugLog(...args: unknown[]): void {
  if (!DEBUG) return
  if (!logDirReady) {
    fs.mkdirSync(LOG_DIR, { recursive: true })
    logDirReady = true
  }
  fs.appendFileSync(
    LOG_FILE,
    args.map((a) => (typeof a === "string" ? a : JSON.stringify(a))).join(" ") + "\n",
  )
}

/** OpenCode config directory — fallback base for relative {file:...} paths. */
const CONFIG_DIR = path.dirname(
  path.dirname(new URL(import.meta.url).pathname.replace(/^\/([A-Za-z]:)/, "$1")),
)

/** `$HOME`, captured once instead of asking `os` per token. */
const HOME_DIR = os.homedir()

/** Token prefixes. Keep exact: no plain `$VAR`, `%VAR%`, or bare names. */
const TOKEN_START = "{"
const FILE_PREFIX = "{file:"
const ENV_PREFIX = "{env:"
const TOKEN_END = "}"

/** Fast transform gate. Exact expansion still requires a closing `}` later. */
function hasExpandableToken(text: string): boolean {
  let start = text.indexOf(TOKEN_START)
  while (start !== -1) {
    const next = text.charCodeAt(start + 1)
    if (next === 102 && text.startsWith(FILE_PREFIX, start)) return true // f
    if (next === 101 && text.startsWith(ENV_PREFIX, start)) return true // e
    start = text.indexOf(TOKEN_START, start + 1)
  }
  return false
}

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
    return path.join(HOME_DIR, raw.slice(1))
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
 * (async reads). Missing env vars and missing/unreadable files resolve to
 * empty string.
 */
export async function expand(text: string, baseDir: string): Promise<string> {
  if (text.indexOf(TOKEN_START) === -1) return text

  const hasEnv = text.includes(ENV_PREFIX)
  let hasFile = text.includes(FILE_PREFIX)
  if (!hasEnv && !hasFile) return text

  // Preserve existing semantics: env substitution runs before file substitution,
  // so env values may intentionally inject {file:...} tokens.
  if (hasEnv) {
    text = expandEnvTokens(text)
  }

  if (!hasFile) hasFile = text.includes(FILE_PREFIX)
  if (!hasFile) return text
  return expandFileTokens(text, baseDir)
}

/** Expand {env:VAR} tokens with manual scanning to avoid regex allocation/state. */
function expandEnvTokens(text: string): string {
  let out = ""
  let cursor = 0
  let searchFrom = 0
  let changed = false

  while (true) {
    const start = text.indexOf(ENV_PREFIX, searchFrom)
    if (start === -1) break

    const valueStart = start + ENV_PREFIX.length
    const end = text.indexOf(TOKEN_END, valueStart)
    if (end === -1) break

    // Preserve /\{env:([^}]+)\}/ semantics: empty `{env:}` is not a token.
    if (end === valueStart) {
      searchFrom = valueStart
      continue
    }

    const varName = text.slice(valueStart, end)
    const value = process.env[varName] ?? ""
    if (DEBUG) debugLog(`env: ${varName} → ${value ? "<set>" : "<unset>"}`)

    out += text.slice(cursor, start) + value
    cursor = end + 1
    searchFrom = cursor
    changed = true
  }

  return changed ? out + text.slice(cursor) : text
}

/** Expand {file:path} tokens. Reads start during scan and resolve in parallel. */
async function expandFileTokens(text: string, baseDir: string): Promise<string> {
  const parts: string[] = []
  const reads: Promise<string>[] = []
  const rawPaths: string[] = []
  let readCache: Map<string, Promise<string>> | undefined

  let cursor = 0
  let searchFrom = 0

  while (true) {
    const start = text.indexOf(FILE_PREFIX, searchFrom)
    if (start === -1) break

    const valueStart = start + FILE_PREFIX.length
    const end = text.indexOf(TOKEN_END, valueStart)
    if (end === -1) break

    // Preserve /\{file:([^}]+)\}/ semantics: empty `{file:}` is not a token.
    if (end === valueStart) {
      searchFrom = valueStart
      continue
    }

    const rawPath = text.slice(valueStart, end)
    const token = DEBUG ? text.slice(start, end + 1) : ""
    let read: Promise<string> | undefined

    if (readCache) {
      read = readCache.get(rawPath)
    } else if (reads.length > 0) {
      readCache = new Map<string, Promise<string>>()
      for (let i = 0; i < reads.length; i++) {
        readCache.set(rawPaths[i], reads[i])
      }
      read = readCache.get(rawPath)
    }

    if (!read) {
      read = readTokenFile(rawPath, baseDir, token)
      readCache?.set(rawPath, read)
    }

    parts.push(text.slice(cursor, start))
    rawPaths.push(rawPath)
    reads.push(read)
    cursor = end + 1
    searchFrom = cursor
  }

  if (!reads.length) return text

  const tail = text.slice(cursor)
  if (reads.length === 1) {
    return parts[0] + (await reads[0]) + tail
  }

  const contents = await Promise.all(reads)
  let out = ""
  for (let i = 0; i < contents.length; i++) {
    out += parts[i] + contents[i]
  }
  return out + tail
}

/** Read one {file:...} token, with config-dir fallback for relative paths. */
async function readTokenFile(rawPath: string, baseDir: string, token: string): Promise<string> {
  const resolved = resolvePath(rawPath, baseDir)

  try {
    const content = (await fsp.readFile(resolved, "utf8")).trim()
    if (DEBUG) debugLog(`file: ${token} → ${resolved} (${content.length} chars)`)
    return content
  } catch (err: unknown) {
    const e = err as NodeJS.ErrnoException
    const canFallback = e.code === "ENOENT" && (rawPath.startsWith("./") || rawPath.startsWith("../"))
    if (!canFallback) {
      if (DEBUG) {
        if (e.code === "ENOENT") debugLog(`file: ${token} → ${resolved} DOES NOT EXIST`)
        else debugLog(`file: ${token} → ${resolved} READ ERROR: ${e.message}`)
      }
      return ""
    }

    const configResolved = path.resolve(CONFIG_DIR, rawPath)
    if (configResolved === resolved) {
      if (DEBUG) debugLog(`file: ${token} → ${resolved} DOES NOT EXIST`)
      return ""
    }

    try {
      const content = (await fsp.readFile(configResolved, "utf8")).trim()
      if (DEBUG) debugLog(`file: ${token} → ${configResolved} (${content.length} chars) [config dir fallback]`)
      return content
    } catch (err2: unknown) {
      const e2 = err2 as NodeJS.ErrnoException
      if (DEBUG) {
        if (e2.code === "ENOENT") debugLog(`file: ${token} → ${resolved} DOES NOT EXIST`)
        else debugLog(`file: ${token} → ${configResolved} READ ERROR: ${e2.message}`)
      }
      return ""
    }
  }
}
