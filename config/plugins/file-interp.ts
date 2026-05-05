/**
 * File Interpolation plugin — recursively expands {arg:...}, {env:...}, and
 * `{{ file="..." }}` templates in .md agent prompts.
 *
 * OpenCode already supports native secret-file interpolation in JSON config files,
 * but .md agent/command/mode/skill files receive no interpolation. This plugin
 * fixes that by rewriting the system prompt at LLM-call time via the
 * `experimental.chat.system.transform` hook.
 *
 * Imported file content is recursively expanded up to `MAX_DEPTH` levels.
 * Cycles resolve to empty string. Relative paths resolve against the
 * original `baseDir`, not the file they appear in.
 *
 * Agents and commands load into the system prompt; templates in those files are
 * expanded before the LLM sees them. Plain-text variable references like
 * `GENERAL_RULES_PATH` are left untouched — only {arg:...}, {env:...}, and
 * `{{ file="..." }}` match.
 *
 * # Supported tokens
 * - `{{ file="~/.secrets/key" }}`  — absolute or ~-relative file content
 * - `{{ file="./relative/path" }}` — relative to project directory; falls
 *   back to config directory when the file is not found
 * - `{{ file="./path" key=val key2="val with spaces" }}` — file with caller args
 * - `{env:VAR_NAME}`               — environment variable value
 * - `{arg:key}`                    — caller-provided arg value inside embedded files
 *
 * # Arg rules
 * - `{arg:key}` expands to the value passed by the embedding `{{ file="..." key=val }}` call
 * - Undefined `{arg:...}` resolves to empty string
 * - Arg values are literal strings; tokens inside arg values are not expanded
 * - Args do not cascade: nested `{{ file="..." }}` calls start with empty args unless they provide their own
 * - Expansion order: {arg:} → {env:} → `{{ file="..." }}`
 *
 * # Template style
 * - Use `{{ file="..." }}` for file imports; legacy colon syntax is not supported
 * - `file` must be the first attribute for fast detection
 * - Any whitespace is allowed after `{{`, between attributes, and before `}}`
 * - Prefer multiline templates when passing args:
 *   ```markdown
 *   {{
 *     file="./path/to/template.txt"
 *     arg1="text with spaces"
 *     arg2=text2
 *   }}
 *   ```
 * - Quote values containing whitespace; unquoted values end at whitespace or `}}`
 * - Args decode common escapes: `\\n`, `\\r`, `\\t`, `\\b`, `\\f`, `\\v`, `\\"`, `\\\\`
 *
 * # Raw inlining
 * Inlined content is recursively expanded, then spliced into the prompt.
 * At `MAX_DEPTH`, `{{ file="..." }}` templates are left literal; `{env:...}` still expands.
 *
 * # Usage
 * In any .md agent file:
 * ```markdown
 * Your API key is {{ file="~/.secrets/openai-key" }}
 * Project config: {{ file="./config/prompt-ctx.txt" }}
 * Region: {env:AWS_REGION}
 * System rules: {{ file="./config/rules/general.md" }}
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
 * OpenCode plugin that expands {arg:...}, {env:...}, and `{{ file="..." }}` templates
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

/** Maximum recursion depth for nested file-template expansion. Exported for tests. */
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
 *    so `{env:FOO}` or `{{ file="./x" }}` inside the value must stay untouched.
 * 2. Non-file arg values of `{{ file="./path" key="{env:FOO}" }}` while
 *    scanning the caller text. Those tokens belong to the arg string, not to
 *    the caller prompt.
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

/** OpenCode config directory — fallback base for relative file-template paths. */
const CONFIG_DIR = path.dirname(
  path.dirname(new URL(import.meta.url).pathname.replace(/^\/([A-Za-z]:)/, "$1")),
)

/** `$HOME`, captured once instead of asking `os` per token. */
const HOME_DIR = os.homedir()

/** Token prefixes. Keep exact: no plain `$VAR`, `%VAR%`, or bare names. */
const TOKEN_START = "{"
const FILE_TEMPLATE_START = "{{"
const FILE_TEMPLATE_END = "}}"
const FILE_ATTR = "file"
const ENV_PREFIX = "{env:"
const ARG_PREFIX = "{arg:"
const TOKEN_END = "}"

/** Fast transform gate. Exact file expansion still requires a closing `}}` later. */
function hasExpandableToken(text: string): boolean {
  let start = text.indexOf(TOKEN_START)
  while (start !== -1) {
    const next = text.charCodeAt(start + 1)
    if (next === 123 && startsFileTemplate(text, start)) return true // {
    if (next === 101 && text.startsWith(ENV_PREFIX, start)) return true // e
    if (next === 97 && text.startsWith(ARG_PREFIX, start)) return true // a
    start = text.indexOf(TOKEN_START, start + 1)
  }
  return false
}

/** Fast check for `{{ file=... }}`. Requires `file` first by style rule. */
function startsFileTemplate(text: string, start: number): boolean {
  if (!text.startsWith(FILE_TEMPLATE_START, start)) return false
  let i = start + FILE_TEMPLATE_START.length
  while (i < text.length && isTemplateSpace(text.charCodeAt(i))) i++
  if (!text.startsWith(FILE_ATTR, i)) return false
  i += FILE_ATTR.length
  while (i < text.length && isTemplateSpace(text.charCodeAt(i))) i++
  return text.charCodeAt(i) === 61 // =
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
 * Expand {arg:key}, {env:VAR}, and `{{ file="path" }}` templates in `text`.
 *
 * Arg substitution runs first (synchronous), then environment substitution
 * (synchronous), then file substitution (async reads). File content is
 * recursively expanded if it contains further {arg:...}, {env:...}, or
 * file templates, up to MAX_DEPTH levels deep. At depth ≥ MAX_DEPTH,
 * file templates are left as literal text; {arg:...} and {env:...} tokens
 * are still expanded. Cycles are detected via ancestor-path tracking — a template
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
  let hasFile = text.includes(FILE_TEMPLATE_START)
  if (!hasArg && !hasEnv && !hasFile) return text

  let protectedRanges = EMPTY_RANGES

  if (hasArg) {
    const argResult = expandArgTokens(text, ctx.args)
    text = argResult.text
    protectedRanges = argResult.protectedRanges
  }

  // Preserve existing semantics: env substitution runs before file substitution,
  // so env values may intentionally inject file templates. Env/arg tokens
  // inside non-file template args are shielded so arg values remain literal.
  if (hasEnv) {
    const envResult = expandEnvTokens(text, protectedRanges)
    text = envResult.text
    protectedRanges = envResult.protectedRanges
  }

  if (!hasFile) hasFile = text.includes(FILE_TEMPLATE_START)
  if (!hasFile) return text
  // Depth gate: at MAX_DEPTH, leave file templates as literal text.
  if (ctx.depth >= MAX_DEPTH) return text
  return expandFileTokens(text, baseDir, ctx, protectedRanges)
}

/**
 * Expand `{arg:key}` tokens with manual scanning.
 *
 * Notes:
 * - Runs before env/file expansion so args can compose later file paths, e.g.
 *   `{{ file="./rules/{arg:topic}.md" }}`.
 * - Skips `{arg:...}` text inside non-file template args because those tokens are
 *   literal arg values for the callee, not caller-level tokens.
 * - Marks inserted arg values as protected when they contain braces; later
 *   env/file passes must not expand token-looking text from arg values.
 */
function expandArgTokens(text: string, args: Map<string, string>): SyncExpandResult {
  // Arg expansion sees the original caller text. Compute template arg-value
  // spans once so `{{ file="./tmpl" x="{arg:y}" }}` passes literal `{arg:y}`.
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

    // Only allocate protected spans for values that could contain tokens or
    // template closers. Literal `foo` needs no later skip check; `{env:FOO}` does.
    if (value.indexOf(TOKEN_START) !== -1 || value.indexOf(TOKEN_END) !== -1) {
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
 * for the current string so `{{ file="./tmpl" x="{env:FOO}" }}` keeps `{env:FOO}` as a
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
 * Expand `{{ file="path" }}` templates. Reads start during scan and resolve in parallel.
 *
 * File scanning also observes protected arg-value spans. Example:
 * `{{ file="./tmpl" x="{{ file=\"./secret\" }}" }}` should read `./tmpl`,
 * then pass literal `{{ file="./secret" }}` as `x`; it must not read
 * `./secret` at caller level.
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
    const start = text.indexOf(FILE_TEMPLATE_START, searchFrom)
    if (start === -1) break

    protectedIndex = advanceRangeIndex(protectedRanges, protectedIndex, start)
    if (isInRange(protectedRanges, protectedIndex, start)) {
      searchFrom = protectedRanges[protectedIndex].end
      continue
    }

    const parsed = parseFileTemplate(text, start, protectedRanges)
    if (!parsed) {
      searchFrom = start + FILE_TEMPLATE_START.length
      continue
    }

    if (parsed.rawPath.length === 0) {
      searchFrom = start + FILE_TEMPLATE_START.length
      continue
    }

    const { rawPath, args, end } = parsed
    const token = DEBUG ? text.slice(start, end + 1) : ""
    const resolved = resolvePath(rawPath, baseDir)
    if (DEBUG && args.size > 0) {
      debugLog(`file: ${rawPath} ${formatArgsForCall(args)} → ${resolved} (${args.size} args: ${formatArgsForLog(args)})`)
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

interface FileTemplateSpec {
  rawPath: string
  args: Map<string, string>
  /** Inclusive offset of the second `}` in the closing `}}`. */
  end: number
  /** Raw spans for non-file attr values; used only by collectFileArgRanges. */
  argValueRanges?: ProtectedRange[]
}

interface TemplateValue {
  value: string
  valueStart: number
  valueEnd: number
  next: number
}

/**
 * Parse one `{{ file="..." ... }}` template.
 *
 * Grammar stays intentionally small and scanner-only for prompt hot path:
 * - `file` must be the first attribute
 * - attributes are `key=value`; whitespace around `=` is allowed
 * - values are unquoted until whitespace/`}}`, or double-quoted with spaces
 * - common escapes decode in values: `\n`, `\r`, `\t`, `\b`, `\f`, `\v`, `\"`, `\\`
 * - duplicate arg keys use last value; duplicate `file` overwrites path
 */
function parseFileTemplate(
  text: string,
  start: number,
  protectedRanges: ProtectedRange[],
  collectArgRanges = false,
): FileTemplateSpec | undefined {
  if (!text.startsWith(FILE_TEMPLATE_START, start)) return undefined

  let i = start + FILE_TEMPLATE_START.length
  i = skipTemplateSpace(text, i)

  const firstKeyStart = i
  i = scanTemplateKey(text, i)
  if (i === firstKeyStart || text.slice(firstKeyStart, i) !== FILE_ATTR) return undefined

  i = skipTemplateSpace(text, i)
  if (text.charCodeAt(i) !== 61) return undefined // =
  i = skipTemplateSpace(text, i + 1)

  const fileValue = readTemplateValue(text, i, protectedRanges)
  if (!fileValue) return undefined
  let rawPath = fileValue.value
  i = fileValue.next

  let args: Map<string, string> | undefined
  let argValueRanges: ProtectedRange[] | undefined

  while (i < text.length) {
    i = skipTemplateSpace(text, i)
    if (text.startsWith(FILE_TEMPLATE_END, i)) {
      return {
        rawPath,
        args: args ?? EMPTY_ARGS,
        end: i + FILE_TEMPLATE_END.length - 1,
        argValueRanges,
      }
    }

    const keyStart = i
    i = scanTemplateKey(text, i)
    if (i === keyStart) return undefined
    const key = text.slice(keyStart, i)

    i = skipTemplateSpace(text, i)
    if (text.charCodeAt(i) !== 61) return undefined // =
    i = skipTemplateSpace(text, i + 1)

    const value = readTemplateValue(text, i, protectedRanges)
    if (!value) return undefined
    i = value.next

    if (!isValidArgKey(key)) {
      if (DEBUG) debugLog(`template parse: skipped invalid key "${key}"`)
      continue
    }

    if (key === FILE_ATTR) {
      rawPath = value.value
      continue
    }

    if (!args) args = new Map()
    args.set(key, value.value)

    if (collectArgRanges) {
      if (!argValueRanges) argValueRanges = []
      argValueRanges.push({ start: value.valueStart, end: value.valueEnd })
    }
  }

  return undefined
}

function scanTemplateKey(text: string, i: number): number {
  if (i >= text.length || !isArgKeyStart(text.charCodeAt(i))) return i
  i++
  while (i < text.length && isArgKeyChar(text.charCodeAt(i))) i++
  return i
}

function readTemplateValue(
  text: string,
  start: number,
  protectedRanges: ProtectedRange[],
): TemplateValue | undefined {
  if (start > text.length) return undefined
  if (text.charCodeAt(start) === 34) return readQuotedTemplateValue(text, start, protectedRanges) // "
  return readUnquotedTemplateValue(text, start, protectedRanges)
}

function readQuotedTemplateValue(
  text: string,
  quoteStart: number,
  protectedRanges: ProtectedRange[],
): TemplateValue | undefined {
  let i = quoteStart + 1
  let chunkStart = i
  let value: string | undefined
  let protectedIndex = 0

  while (i < text.length) {
    protectedIndex = advanceRangeIndex(protectedRanges, protectedIndex, i)
    if (isInRange(protectedRanges, protectedIndex, i)) {
      i = protectedRanges[protectedIndex].end
      continue
    }

    const code = text.charCodeAt(i)
    if (code === 92) { // \
      if (value === undefined) value = ""
      value += text.slice(chunkStart, i) + decodeTemplateEscape(text.charCodeAt(i + 1))
      i += i + 1 < text.length ? 2 : 1
      chunkStart = i
      continue
    }
    if (code === 34) { // "
      return {
        value: value === undefined ? text.slice(quoteStart + 1, i) : value + text.slice(chunkStart, i),
        valueStart: quoteStart + 1,
        valueEnd: i,
        next: i + 1,
      }
    }
    i++
  }

  return undefined
}

function readUnquotedTemplateValue(
  text: string,
  start: number,
  protectedRanges: ProtectedRange[],
): TemplateValue {
  let i = start
  let protectedIndex = 0
  while (i < text.length) {
    protectedIndex = advanceRangeIndex(protectedRanges, protectedIndex, i)
    if (isInRange(protectedRanges, protectedIndex, i)) {
      i = protectedRanges[protectedIndex].end
      continue
    }

    const code = text.charCodeAt(i)
    if (isTemplateSpace(code) || text.startsWith(FILE_TEMPLATE_END, i)) break
    i++
  }

  const raw = text.slice(start, i)
  return {
    value: raw.indexOf("\\") === -1 ? raw : decodeTemplateEscapes(raw),
    valueStart: start,
    valueEnd: i,
    next: i,
  }
}

function decodeTemplateEscapes(text: string): string {
  let out = ""
  let chunkStart = 0
  for (let i = 0; i < text.length; i++) {
    if (text.charCodeAt(i) !== 92) continue // \
    out += text.slice(chunkStart, i) + decodeTemplateEscape(text.charCodeAt(i + 1))
    i += i + 1 < text.length ? 1 : 0
    chunkStart = i + 1
  }
  return out + text.slice(chunkStart)
}

function decodeTemplateEscape(code: number): string {
  if (Number.isNaN(code)) return "\\"
  switch (code) {
    case 34: return '"'
    case 92: return "\\"
    case 98: return "\b"
    case 102: return "\f"
    case 110: return "\n"
    case 114: return "\r"
    case 116: return "\t"
    case 118: return "\v"
    default: return "\\" + String.fromCharCode(code)
  }
}

/** Template attrs may span lines; any common ASCII whitespace separates items. */
function isTemplateSpace(code: number): boolean {
  return code === 32 || code === 9 || code === 10 || code === 13
}

function skipTemplateSpace(text: string, i: number): number {
  while (i < text.length && isTemplateSpace(text.charCodeAt(i))) i++
  return i
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
 * Find non-file arg value spans in `{{ file="..." key=value }}` templates.
 *
 * Example: in `{{ file="./tmpl" x="{env:FOO}" }}`, this returns the span
 * covering `{env:FOO}`. The env pass ignores it at caller level; later,
 * `parseFileTemplate` passes it as literal `x` to `tmpl`. The `file` value is
 * not protected so `{arg:topic}` and `{env:PATH_PART}` can compose paths.
 */
function collectFileArgRanges(text: string, protectedRanges: ProtectedRange[]): ProtectedRange[] {
  if (text.indexOf(FILE_TEMPLATE_START) === -1) return EMPTY_RANGES

  const ranges: ProtectedRange[] = []
  let searchFrom = 0
  let protectedIndex = 0

  while (true) {
    const start = text.indexOf(FILE_TEMPLATE_START, searchFrom)
    if (start === -1) break

    protectedIndex = advanceRangeIndex(protectedRanges, protectedIndex, start)
    if (isInRange(protectedRanges, protectedIndex, start)) {
      searchFrom = protectedRanges[protectedIndex].end
      continue
    }

    const parsed = parseFileTemplate(text, start, protectedRanges, true)
    if (!parsed) {
      searchFrom = start + FILE_TEMPLATE_START.length
      continue
    }

    if (parsed.argValueRanges) ranges.push(...parsed.argValueRanges)
    searchFrom = parsed.end + 1
  }

  return ranges.length ? ranges : EMPTY_RANGES
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
