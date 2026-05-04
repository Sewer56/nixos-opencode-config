/**
 * File Interpolation plugin — recursively expands {file:...} and {env:...} in .md agent prompts.
 *
 * OpenCode already supports {file:~/.secrets/openai-key} in JSON config files,
 * but .md agent/command/mode/skill files receive no interpolation. This plugin
 * fixes that by rewriting the system prompt at LLM-call time via the
 * `experimental.chat.system.transform` hook.
 *
 * Imported file content is recursively expanded up to `MAX_DEPTH` levels.
 * Cycles resolve to empty string. Relative paths resolve against the
 * original `baseDir`, not the file they appear in.
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
 * Inlined content is recursively expanded, then spliced into the prompt.
 * At `MAX_DEPTH`, `{file:...}` tokens are left literal; `{env:...}` still expands.
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
 * - `MAX_DEPTH` — maximum recursion depth, exported for tests
 */
import type { Plugin } from "@opencode-ai/plugin"
import path from "node:path"
import os from "node:os"
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

/** Maximum recursion depth for nested {file:...} expansion. Exported for tests. */
export const MAX_DEPTH = 10

/** Shared context for a single expand() call tree — carries cycle guard and read cache. */
interface ExpandContext {
  /** Resolved absolute paths of ancestor files in the current recursion chain. */
  visited: Set<string>
  depth: number
  /** Raw I/O cache keyed by resolved absolute path (content before recursive expansion). */
  readCache: Map<string, Promise<string>>
}

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
 * (async reads). File content is recursively expanded if it contains further
 * {file:...} or {env:...} tokens, up to MAX_DEPTH levels deep. At depth ≥
 * MAX_DEPTH, {file:...} tokens are left as literal text; {env:...} tokens
 * are still expanded. Cycles are detected via ancestor-path tracking — a
 * token referencing a file in its own ancestor chain resolves to empty string.
 *
 * An optional `ExpandContext` carries recursion state (cycle guard, depth
 * counter, raw I/O cache). External callers should omit it — a fresh context
 * is created internally.
 */
export async function expand(text: string, baseDir: string, ctx?: ExpandContext): Promise<string> {
  if (text.indexOf(TOKEN_START) === -1) return text

  if (!ctx) ctx = { visited: new Set(), depth: 0, readCache: new Map() }

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
  // Depth gate: at MAX_DEPTH, leave {file:...} tokens as literal text.
  if (ctx.depth >= MAX_DEPTH) return text
  return expandFileTokens(text, baseDir, ctx)
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
async function expandFileTokens(text: string, baseDir: string, ctx: ExpandContext): Promise<string> {
  const parts: string[] = []
  const reads: Promise<string>[] = []

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
    const resolved = resolvePath(rawPath, baseDir)

    // Cycle detection: skip files already in this token's ancestor chain
    if (ctx.visited.has(resolved)) {
      if (DEBUG) debugLog(`file: ${token} → ${resolved} SKIPPED (cycle detected)`)
      parts.push(text.slice(cursor, start))
      reads.push(Promise.resolve(""))
      cursor = end + 1
      searchFrom = cursor
      continue
    }

    // Raw I/O dedup: same resolved path → reuse cached raw read
    let rawPromise = ctx.readCache.get(resolved)
    if (!rawPromise) {
      rawPromise = readRawFile(resolved, rawPath, token)
      ctx.readCache.set(resolved, rawPromise)
    }

    // Per-caller expansion: each token gets its own expansion with its
    // ancestor chain, so sibling branches don't cross-contaminate.
    const read = rawPromise.then(raw =>
      recursivelyExpand(raw, resolved, baseDir, token, ctx)
    )

    parts.push(text.slice(cursor, start))
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

/** Read raw file content (trimmed), with config-dir fallback for relative paths. */
async function readRawFile(
  resolved: string,
  rawPath: string,
  token: string,
): Promise<string> {
  try {
    const raw = (await Bun.file(resolved).text()).trim()
    if (DEBUG) debugLog(`file: ${token} → ${resolved} (${raw.length} chars)`)
    return raw
  } catch (err: unknown) {
    const code = (err as NodeJS.ErrnoException)?.code
    const canFallback = code === "ENOENT" && (rawPath.startsWith("./") || rawPath.startsWith("../"))
    if (!canFallback) {
      if (DEBUG) {
        if (code === "ENOENT") debugLog(`file: ${token} → ${resolved} DOES NOT EXIST`)
        else debugLog(`file: ${token} → ${resolved} READ ERROR: ${(err as Error).message}`)
      }
      return ""
    }

    const configResolved = path.resolve(CONFIG_DIR, rawPath)
    if (configResolved === resolved) {
      if (DEBUG) debugLog(`file: ${token} → ${resolved} DOES NOT EXIST`)
      return ""
    }

    try {
      const content = (await Bun.file(configResolved).text()).trim()
      if (DEBUG) debugLog(`file: ${token} → ${configResolved} (${content.length} chars) [config dir fallback]`)
      return content
    } catch (err2: unknown) {
      const code2 = (err2 as NodeJS.ErrnoException)?.code
      if (DEBUG) {
        if (code2 === "ENOENT") debugLog(`file: ${token} → ${resolved} DOES NOT EXIST`)
        else debugLog(`file: ${token} → ${configResolved} READ ERROR: ${(err2 as Error).message}`)
      }
      return ""
    }
  }
}

/**
 * Recursively expand tokens in raw file content if depth allows and tokens exist.
 * Creates an immutable snapshot of the visited set with `resolved` added,
 * so sibling tokens can independently visit the same file while ancestor-chain
 * cycles are still broken.
 */
async function recursivelyExpand(
  raw: string,
  resolved: string,
  baseDir: string,
  token: string,
  ctx: ExpandContext,
): Promise<string> {
  if (!hasExpandableToken(raw)) return raw
  const childVisited = new Set(ctx.visited)
  childVisited.add(resolved)
  const expanded = await expand(raw, baseDir, {
    visited: childVisited,
    depth: ctx.depth + 1,
    readCache: ctx.readCache,
  })
  if (DEBUG) debugLog(`file: ${token} → ${resolved} recursive expansion (${expanded.length} chars, depth ${ctx.depth + 1})`)
  return expanded
}
