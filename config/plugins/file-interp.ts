/**
 * File Interpolation plugin — recursively expands {arg:...}, {env:...}, and
 * {file:...} in .md agent prompts.
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
 * `GENERAL_RULES_PATH` are left untouched — only {arg:...}, {env:...}, and
 * {file:...} match.
 *
 * # Supported tokens
 * - `{file:~/.secrets/key}`  — absolute or ~-relative file content
 * - `{file:./relative/path}` — relative to project directory; falls
 *   back to config directory when the file is not found
 * - `{file:./path|key=val key2="val with spaces"}` — file with caller args
 * - `{env:VAR_NAME}`         — environment variable value
 * - `{arg:key}`              — caller-provided arg value inside embedded files
 *
 * # Arg rules
 * - `{arg:key}` expands to the value passed by the embedding `{file:...|key=val}` call
 * - Undefined `{arg:...}` resolves to empty string
 * - Arg values are literal strings; tokens inside arg values are not expanded
 * - Args do not cascade: nested `{file:}` calls start with empty args unless they provide their own
 * - Expansion order: {arg:} → {env:} → {file:}
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
 * OpenCode plugin that expands {arg:...}, {env:...}, and {file:...} tokens
 * in .md agent system prompts.
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

/** Shared context for a single expand() call tree — carries cycle guard, read cache, and args. */
interface ExpandContext {
  /** Resolved absolute paths of ancestor files in the current recursion chain. */
  visited: Set<string>
  depth: number
  /** Raw I/O cache keyed by resolved absolute path (content before recursive expansion). */
  readCache: Map<string, Promise<string>>
  /** Caller-provided args scoped to this expansion level. */
  args: Map<string, string>
}

/**
 * Half-open `[start, end)` text span that must not be scanned for tokens.
 *
 * Used for two cases:
 * 1. Text inserted from `{arg:key}` values. Arg values are literal by design,
 *    so `{env:FOO}` or `{file:./x}` inside the value must stay untouched.
 * 2. The arg tail of `{file:./path|key={env:FOO}}` while scanning the caller
 *    text. Those tokens belong to the arg string, not to the caller prompt.
 */
interface ProtectedRange {
  start: number
  end: number
}

/**
 * Replacement metadata for one sync token substitution.
 *
 * Protected ranges are offsets into the current string. If `{env:LONG_NAME}`
 * before a protected range becomes `x`, following protected offsets shift left.
 * This compact record lets `remapProtectedRanges` adjust only when needed.
 */
interface ReplacementRange {
  start: number
  end: number
  length: number
}

/** Result of sync token expansion that must preserve protected arg-literal spans. */
interface SyncExpandResult {
  text: string
  protectedRanges: ProtectedRange[]
}

/** Shared immutable empty maps/ranges to avoid hot-path allocations. Never mutate. */
const EMPTY_ARGS = new Map<string, string>()
const EMPTY_RANGES: ProtectedRange[] = []

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
const ARG_PREFIX = "{arg:"
const TOKEN_END = "}"
const ARG_SEPARATOR = "|"

/** Fast transform gate. Exact expansion still requires a closing `}` later. */
function hasExpandableToken(text: string): boolean {
  let start = text.indexOf(TOKEN_START)
  while (start !== -1) {
    const next = text.charCodeAt(start + 1)
    if (next === 102 && text.startsWith(FILE_PREFIX, start)) return true // f
    if (next === 101 && text.startsWith(ENV_PREFIX, start)) return true // e
    if (next === 97 && text.startsWith(ARG_PREFIX, start)) return true // a
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
 * Expand {arg:key}, {env:VAR}, and {file:path} tokens in `text`.
 *
 * Arg substitution runs first (synchronous), then environment substitution
 * (synchronous), then file substitution (async reads). File content is
 * recursively expanded if it contains further {arg:...}, {env:...}, or
 * {file:...} tokens, up to MAX_DEPTH levels deep. At depth ≥ MAX_DEPTH,
 * {file:...} tokens are left as literal text; {arg:...} and {env:...} tokens
 * are still expanded. Cycles are detected via ancestor-path tracking — a token
 * referencing a file in its own ancestor chain resolves to empty string.
 *
 * An optional `ExpandContext` carries recursion state (cycle guard, depth
 * counter, raw I/O cache, scoped args). External callers should omit it — a
 * fresh context is created internally.
 */
export async function expand(text: string, baseDir: string, ctx?: ExpandContext): Promise<string> {
  if (text.indexOf(TOKEN_START) === -1) return text

  if (!ctx) ctx = { visited: new Set(), depth: 0, readCache: new Map(), args: EMPTY_ARGS }

  const hasArg = text.includes(ARG_PREFIX)
  const hasEnv = text.includes(ENV_PREFIX)
  let hasFile = text.includes(FILE_PREFIX)
  if (!hasArg && !hasEnv && !hasFile) return text

  let protectedRanges = EMPTY_RANGES

  if (hasArg) {
    const argResult = expandArgTokens(text, ctx.args)
    text = argResult.text
    protectedRanges = argResult.protectedRanges
  }

  // Preserve existing semantics: env substitution runs before file substitution,
  // so env values may intentionally inject {file:...} tokens. Env/arg tokens
  // inside {file:...|args} are shielded so arg values remain literal.
  if (hasEnv) {
    const envResult = expandEnvTokens(text, protectedRanges)
    text = envResult.text
    protectedRanges = envResult.protectedRanges
  }

  if (!hasFile) hasFile = text.includes(FILE_PREFIX)
  if (!hasFile) return text
  // Depth gate: at MAX_DEPTH, leave {file:...} tokens as literal text.
  if (ctx.depth >= MAX_DEPTH) return text
  return expandFileTokens(text, baseDir, ctx, protectedRanges)
}

/**
 * Expand `{arg:key}` tokens with manual scanning.
 *
 * Notes:
 * - Runs before env/file expansion so args can compose later file paths, e.g.
 *   `{file:./rules/{arg:topic}.md}`.
 * - Skips `{arg:...}` text inside `{file:...|args}` because those tokens are
 *   literal arg values for the callee, not caller-level tokens.
 * - Marks inserted arg values as protected when they contain `{`; later env/file
 *   passes must not expand token-looking text from arg values.
 */
function expandArgTokens(text: string, args: Map<string, string>): SyncExpandResult {
  // Arg expansion sees the original caller text. Compute `{file:...|args}`
  // spans once so `{file:./tmpl|x={arg:y}}` passes literal `{arg:y}`.
  const fileArgRanges = collectFileArgRanges(text, EMPTY_RANGES)
  let fileArgIndex = 0
  let out = ""
  let cursor = 0
  let searchFrom = 0
  let changed = false
  const protectedRanges: ProtectedRange[] = []

  while (true) {
    const start = text.indexOf(ARG_PREFIX, searchFrom)
    if (start === -1) break

    fileArgIndex = advanceRangeIndex(fileArgRanges, fileArgIndex, start)
    if (isInRange(fileArgRanges, fileArgIndex, start)) {
      searchFrom = fileArgRanges[fileArgIndex].end
      continue
    }

    const valueStart = start + ARG_PREFIX.length
    const end = text.indexOf(TOKEN_END, valueStart)
    if (end === -1) break

    const key = text.slice(valueStart, end)
    const found = args.has(key)
    const value = found ? args.get(key)! : ""
    if (DEBUG) debugLog(`arg: ${key} → ${found ? value : "<undefined>"}`)

    // Build output lazily. `cursor` is the unflushed slice start from input.
    out += text.slice(cursor, start)
    const outStart = out.length
    out += value

    // Only allocate protected spans for values that could contain tokens.
    // Literal `foo` needs no later skip check; `{env:FOO}` does.
    if (value.indexOf(TOKEN_START) !== -1) {
      protectedRanges.push({ start: outStart, end: outStart + value.length })
    }

    cursor = end + 1
    searchFrom = cursor
    changed = true
  }

  return changed
    ? { text: out + text.slice(cursor), protectedRanges }
    : { text, protectedRanges: EMPTY_RANGES }
}

/**
 * Expand `{env:VAR}` tokens with manual scanning to avoid regex allocation/state.
 *
 * `protectedRanges` come from earlier arg expansion. `fileArgRanges` are added
 * for the current string so `{file:./tmpl|x={env:FOO}}` keeps `{env:FOO}` as a
 * literal callee arg. If replacements happen before protected text, offsets are
 * remapped before returning.
 */
function expandEnvTokens(text: string, protectedRanges: ProtectedRange[]): SyncExpandResult {
  const fileArgRanges = collectFileArgRanges(text, protectedRanges)
  const skipRanges = mergeRanges(protectedRanges, fileArgRanges)
  let skipIndex = 0
  let out = ""
  let cursor = 0
  let searchFrom = 0
  let changed = false
  const replacements: ReplacementRange[] | undefined = protectedRanges.length ? [] : undefined

  while (true) {
    const start = text.indexOf(ENV_PREFIX, searchFrom)
    if (start === -1) break

    skipIndex = advanceRangeIndex(skipRanges, skipIndex, start)
    if (isInRange(skipRanges, skipIndex, start)) {
      searchFrom = skipRanges[skipIndex].end
      continue
    }

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
    // Track offset delta only when there are protected ranges to preserve.
    replacements?.push({ start, end: end + 1, length: value.length })
    cursor = end + 1
    searchFrom = cursor
    changed = true
  }

  if (!changed) return { text, protectedRanges }
  return {
    text: out + text.slice(cursor),
    protectedRanges: replacements
      ? remapProtectedRanges(protectedRanges, replacements)
      : protectedRanges,
  }
}

/**
 * Expand `{file:path}` tokens. Reads start during scan and resolve in parallel.
 *
 * File scanning also observes protected arg-value spans. Example:
 * `{file:./tmpl|x={file:./secret}}` should read `./tmpl`, then pass literal
 * `{file:./secret}` as `x`; it must not read `./secret` at caller level.
 */
async function expandFileTokens(
  text: string,
  baseDir: string,
  ctx: ExpandContext,
  protectedRanges: ProtectedRange[],
): Promise<string> {
  const parts: string[] = []
  const reads: Promise<string>[] = []

  let cursor = 0
  let searchFrom = 0
  let protectedIndex = 0

  while (true) {
    const start = text.indexOf(FILE_PREFIX, searchFrom)
    if (start === -1) break

    protectedIndex = advanceRangeIndex(protectedRanges, protectedIndex, start)
    if (isInRange(protectedRanges, protectedIndex, start)) {
      searchFrom = protectedRanges[protectedIndex].end
      continue
    }

    const valueStart = start + FILE_PREFIX.length
    const end = findFileTokenEnd(text, valueStart, protectedRanges)
    if (end === -1) break

    // Preserve /\{file:([^}]+)\}/ semantics: empty `{file:}` is not a token.
    if (end === valueStart) {
      searchFrom = valueStart
      continue
    }

    const rawSpec = text.slice(valueStart, end)
    const { rawPath, args } = parseFileSpec(rawSpec)
    const token = DEBUG ? text.slice(start, end + 1) : ""
    const resolved = resolvePath(rawPath, baseDir)
    if (DEBUG && args.size > 0) {
      debugLog(`file: ${rawPath}|${formatArgsForCall(args)} → ${resolved} (${args.size} args: ${formatArgsForLog(args)})`)
    }

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
      recursivelyExpand(raw, resolved, baseDir, token, ctx, args)
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

/**
 * Split a `{file:...}` body into path and caller args.
 *
 * Only the first `|` is structural:
 * - `./path`             → path `./path`, empty args
 * - `./path|a=1 b="two"` → path `./path`, parsed args
 *
 * Pipe is chosen as separator because it is not a valid path char on target OSs,
 * so paths may contain spaces without quoting.
 */
function parseFileSpec(rawSpec: string): { rawPath: string, args: Map<string, string> } {
  const pipe = rawSpec.indexOf(ARG_SEPARATOR)
  if (pipe === -1) return { rawPath: rawSpec, args: EMPTY_ARGS }
  return {
    rawPath: rawSpec.slice(0, pipe),
    args: parseArgString(rawSpec.slice(pipe + 1)),
  }
}

/**
 * Parse the arg tail from `{file:./path|...}` into a map.
 *
 * Accepted grammar, intentionally small:
 * - Segment separator: whitespace outside quotes
 * - Key: `[a-zA-Z_][a-zA-Z0-9_-]*`
 * - Value: unquoted text up to whitespace, or quoted `"..."`
 * - Escape: only `\"` inside quoted values
 * - Duplicate key: later value overwrites earlier value
 * - Invalid segment/key: skipped with debug log
 *
 * No regex: this sits on prompt hot path, and a single char-code scanner avoids
 * regex allocation/backtracking while making quoted segments explicit.
 */
function parseArgString(text: string): Map<string, string> {
  // Allocate only when at least one valid arg exists. Most `{file:...}` tokens
  // have no args, so callers usually keep using shared EMPTY_ARGS.
  let args: Map<string, string> | undefined
  let i = 0

  while (i < text.length) {
    while (i < text.length && isArgSpace(text.charCodeAt(i))) i++
    if (i >= text.length) break

    const segmentStart = i
    const keyStart = i

    // Scan the key candidate up to either `=` or whitespace. Whitespace before
    // `=` makes the segment invalid (`foo =bar` is two bad/independent parts),
    // matching the no-quoting/no-escaping-for-keys rule.
    while (i < text.length) {
      const code = text.charCodeAt(i)
      if (code === 61 || isArgSpace(code)) break // = or space
      i++
    }

    if (i >= text.length || text.charCodeAt(i) !== 61) {
      // Segment has no `=` before its separating whitespace. Skip the whole
      // segment so `bad key=ok` still recovers at `key=ok`.
      const invalidEnd = skipArgSegment(text, segmentStart)
      if (DEBUG) debugLog(`arg parse: skipped invalid segment "${text.slice(segmentStart, invalidEnd)}"`)
      i = invalidEnd
      continue
    }

    const key = text.slice(keyStart, i)
    i++ // =

    let value: string
    if (i < text.length && text.charCodeAt(i) === 34) { // "
      // Quoted value: keep spaces, unescape `\"`, stop at next unescaped `"`.
      // Unterminated quotes are lenient: consume rest as value. That preserves
      // caller text better than dropping the segment.
      i++
      let chunkStart = i
      let closed = false
      value = ""
      while (i < text.length) {
        const code = text.charCodeAt(i)
        if (code === 92 && text.charCodeAt(i + 1) === 34) { // \"
          value += text.slice(chunkStart, i) + '"'
          i += 2
          chunkStart = i
          continue
        }
        if (code === 34) {
          value += text.slice(chunkStart, i)
          i++
          closed = true
          break
        }
        i++
      }
      if (!closed) value += text.slice(chunkStart)
    } else {
      // Unquoted value: literal bytes until next whitespace. This means
      // `key={env:FOO}` is stored literally; expansion code protects it later.
      const valueStart = i
      while (i < text.length && !isArgSpace(text.charCodeAt(i))) i++
      value = text.slice(valueStart, i)
    }

    if (!isValidArgKey(key)) {
      if (DEBUG) debugLog(`arg parse: skipped invalid key "${key}"`)
      continue
    }

    if (!args) args = new Map()
    args.set(key, value)
  }

  return args ?? EMPTY_ARGS
}

/**
 * Return the end offset of one malformed arg segment.
 *
 * Used only after parser sees a segment without `=`. We still respect quotes so
 * `bad "not a real arg" key=ok` skips the bad quoted region and resumes at
 * `key=ok`, instead of splitting inside the quotes.
 */
function skipArgSegment(text: string, start: number): number {
  let inQuote = false
  for (let i = start; i < text.length; i++) {
    const code = text.charCodeAt(i)
    if (inQuote) {
      // Escaped quote inside a malformed quoted segment: skip both chars so it
      // does not close the quote.
      if (code === 92 && text.charCodeAt(i + 1) === 34) {
        i++
        continue
      }
      if (code === 34) inQuote = false
      continue
    }
    if (code === 34) {
      inQuote = true
      continue
    }
    if (isArgSpace(code)) return i
  }
  return text.length
}

/** Args are single-line constants; any whitespace separates segments. */
function isArgSpace(code: number): boolean {
  return code === 32 || code === 9 || code === 10 || code === 13
}

/** Validate `[a-zA-Z_][a-zA-Z0-9_-]*` without regex allocation. */
function isValidArgKey(key: string): boolean {
  if (key.length === 0) return false
  if (!isArgKeyStart(key.charCodeAt(0))) return false
  for (let i = 1; i < key.length; i++) {
    if (!isArgKeyChar(key.charCodeAt(i))) return false
  }
  return true
}

function isArgKeyStart(code: number): boolean {
  return code === 95 || (code >= 65 && code <= 90) || (code >= 97 && code <= 122)
}

function isArgKeyChar(code: number): boolean {
  return isArgKeyStart(code) || code === 45 || (code >= 48 && code <= 57)
}

/**
 * Find arg-string spans in `{file:...|args}` so sync expansion can skip them.
 *
 * Example: in `{file:./tmpl|x={env:FOO}}`, this returns the span covering
 * `x={env:FOO}`. The env pass then ignores `{env:FOO}` at caller level; later,
 * `parseArgString` passes it as literal `x` to `tmpl`.
 */
function collectFileArgRanges(text: string, protectedRanges: ProtectedRange[]): ProtectedRange[] {
  if (text.indexOf(FILE_PREFIX) === -1 || text.indexOf(ARG_SEPARATOR) === -1) return EMPTY_RANGES

  const ranges: ProtectedRange[] = []
  let searchFrom = 0
  let protectedIndex = 0

  while (true) {
    const start = text.indexOf(FILE_PREFIX, searchFrom)
    if (start === -1) break

    protectedIndex = advanceRangeIndex(protectedRanges, protectedIndex, start)
    if (isInRange(protectedRanges, protectedIndex, start)) {
      searchFrom = protectedRanges[protectedIndex].end
      continue
    }

    const valueStart = start + FILE_PREFIX.length
    const end = findFileTokenEnd(text, valueStart, protectedRanges)
    if (end === -1) break
    if (end === valueStart) {
      searchFrom = valueStart
      continue
    }

    const pipe = findFirstPipe(text, valueStart, end, protectedRanges)
    if (pipe !== -1) ranges.push({ start: pipe + 1, end })
    searchFrom = end + 1
  }

  return ranges.length ? ranges : EMPTY_RANGES
}

/**
 * Find the closing `}` for a `{file:...}` token body.
 *
 * Plain `text.indexOf("}")` is not enough once args can contain token-looking
 * literals: `{file:./tmpl|x={env:FOO}}` should end at the final brace, not the
 * brace after `FOO`. We count known nested token starts (`{arg:`, `{env:`,
 * `{file:`) and ignore braces inside quoted arg values.
 */
function findFileTokenEnd(text: string, valueStart: number, protectedRanges: ProtectedRange[]): number {
  let nested = 0
  let inQuote = false
  let protectedIndex = 0

  for (let i = valueStart; i < text.length;) {
    protectedIndex = advanceRangeIndex(protectedRanges, protectedIndex, i)
    if (isInRange(protectedRanges, protectedIndex, i)) {
      i = protectedRanges[protectedIndex].end
      continue
    }

    const code = text.charCodeAt(i)
    if (inQuote) {
      if (code === 92 && text.charCodeAt(i + 1) === 34) {
        i += 2
        continue
      }
      if (code === 34) inQuote = false
      i++
      continue
    }

    if (code === 34) {
      inQuote = true
      i++
      continue
    }
    if (code === 123 && startsKnownToken(text, i)) { // {
      // Nested token-looking text inside arg values is literal, but counting it
      // lets the outer file token consume its matching `}` correctly.
      nested++
      i++
      continue
    }
    if (code === 125) { // }
      if (nested > 0) nested--
      else return i
    }
    i++
  }

  return -1
}

function startsKnownToken(text: string, start: number): boolean {
  const next = text.charCodeAt(start + 1)
  return (
    (next === 102 && text.startsWith(FILE_PREFIX, start)) ||
    (next === 101 && text.startsWith(ENV_PREFIX, start)) ||
    (next === 97 && text.startsWith(ARG_PREFIX, start))
  )
}

/** Find the first structural `|` in a file token body. */
function findFirstPipe(
  text: string,
  start: number,
  end: number,
  protectedRanges: ProtectedRange[],
): number {
  let protectedIndex = 0
  for (let i = start; i < end;) {
    protectedIndex = advanceRangeIndex(protectedRanges, protectedIndex, i)
    if (isInRange(protectedRanges, protectedIndex, i)) {
      i = Math.min(protectedRanges[protectedIndex].end, end)
      continue
    }
    if (text.charCodeAt(i) === 124) return i // |
    i++
  }
  return -1
}

/** Advance range cursor while `pos` is after current range. Ranges are sorted. */
function advanceRangeIndex(ranges: ProtectedRange[], index: number, pos: number): number {
  while (index < ranges.length && ranges[index].end <= pos) index++
  return index
}

/** Check whether `pos` falls inside `ranges[index]`. */
function isInRange(ranges: ProtectedRange[], index: number, pos: number): boolean {
  const range = ranges[index]
  return range !== undefined && pos >= range.start && pos < range.end
}

/** Merge two sorted protected-range lists, coalescing overlaps/touches. */
function mergeRanges(a: ProtectedRange[], b: ProtectedRange[]): ProtectedRange[] {
  if (!a.length) return b
  if (!b.length) return a

  const out: ProtectedRange[] = []
  let i = 0
  let j = 0
  while (i < a.length || j < b.length) {
    const takeA = j >= b.length || (i < a.length && a[i].start <= b[j].start)
    const range = takeA ? a[i++] : b[j++]
    const last = out[out.length - 1]
    if (last && range.start <= last.end) {
      if (range.end > last.end) last.end = range.end
    } else {
      out.push({ start: range.start, end: range.end })
    }
  }
  return out
}

/**
 * Shift protected ranges after sync replacements.
 *
 * Replacements never overlap protected ranges because callers skip protected
 * spans. Therefore each protected range only needs cumulative length delta from
 * replacements ending before it.
 */
function remapProtectedRanges(
  ranges: ProtectedRange[],
  replacements: ReplacementRange[],
): ProtectedRange[] {
  if (!ranges.length || !replacements.length) return ranges

  const out: ProtectedRange[] = []
  let replacementIndex = 0
  let delta = 0
  for (const range of ranges) {
    while (replacementIndex < replacements.length && replacements[replacementIndex].end <= range.start) {
      const replacement = replacements[replacementIndex]
      delta += replacement.length - (replacement.end - replacement.start)
      replacementIndex++
    }
    out.push({ start: range.start + delta, end: range.end + delta })
  }
  return out
}

function formatArgsForCall(args: Map<string, string>): string {
  let out = ""
  for (const [key, value] of args) {
    if (out) out += " "
    out += `${key}=${value}`
  }
  return out
}

function formatArgsForLog(args: Map<string, string>): string {
  let out = ""
  for (const [key, value] of args) {
    if (out) out += ", "
    out += `${key}=${value}`
  }
  return out
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
  args: Map<string, string>,
): Promise<string> {
  if (!hasExpandableToken(raw)) return raw
  const childVisited = new Set(ctx.visited)
  childVisited.add(resolved)
  const expanded = await expand(raw, baseDir, {
    visited: childVisited,
    depth: ctx.depth + 1,
    readCache: ctx.readCache,
    args,
  })
  if (DEBUG) debugLog(`file: ${token} → ${resolved} recursive expansion (${expanded.length} chars, depth ${ctx.depth + 1})`)
  return expanded
}
