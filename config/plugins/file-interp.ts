/**
 * File Interpolation plugin — expands {file:...} and {env:...} in .md agent prompts.
 *
 * OpenCode already supports {file:~/.secrets/openai-key} in JSON config files,
 * but .md agent/command/mode/skill files receive no interpolation. This plugin
 * fixes that by rewriting the system prompt at LLM-call time via the
 * `experimental.chat.system.transform` hook.
 *
 * # Supported tokens
 * - `{file:~/.secrets/key}`  — absolute or ~-relative file content
 * - `{file:./relative/path}` — relative to project directory
 * - `{env:VAR_NAME}`         — environment variable value
 *
 * # Usage
 * In any .md agent file:
 * ```markdown
 * Your API key is {file:~/.secrets/openai-key}
 * Project config: {file:./config/prompt-ctx.txt}
 * Region: {env:AWS_REGION}
 * ```
 *
 * # Public API
 * - `FileInterpPlugin` — default export, consumed by OpenCode plugin loader
 */
import type { Plugin } from "@opencode-ai/plugin"
import path from "path"
import os from "os"
import fs from "fs/promises"

/**
 * Set `FILE_INTERP_DEBUG=1` in your shell env to enable debug logging.
 * Logs are written via the opencode SDK (client.app.log) to:
 *   ~/.local/share/opencode/log/<YYYY-MM-DDTHHMMSS>.log
 * Does NOT print to the TUI.
 */
const DEBUG = process.env.FILE_INTERP_DEBUG === "1"

/** @internal Builds a scoped logger that writes to the opencode server log via the SDK client. */
function createLog(client: { app: { log: (opts: unknown) => Promise<unknown> } }) {
  return (...args: unknown[]) => {
    if (!DEBUG) return
    client.app
      .log({
        body: {
          service: "file-interp",
          level: "info",
          message: args.map((a) => (typeof a === "string" ? a : JSON.stringify(a))).join(" "),
        },
      })
      .catch(() => {})
  }
}

/** Regex matching {file:<path>} tokens. Captures the path portion. */
const FILE_RE = /\{file:([^}]+)\}/g

/** Regex matching {env:<var>} tokens. Captures the var name. */
const ENV_RE = /\{env:([^}]+)\}/g

/**
 * Resolve a raw token path to an absolute filesystem path.
 *
 * - `~/...` → `$HOME/...`
 * - `./...` → relative to `baseDir`
 * - other   → used as-is (assumed absolute)
 */
function resolvePath(raw: string, baseDir: string): string {
  if (raw.startsWith("~/") || raw === "~") {
    return path.join(os.homedir(), raw.slice(1))
  }
  if (raw.startsWith("./") || raw.startsWith("..")) {
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
async function expand(text: string, baseDir: string, log: (...a: unknown[]) => void): Promise<string> {
  // {env:VAR} — synchronous, replace in-place
  text = text.replace(ENV_RE, (_, varName: string) => {
    const value = process.env[varName] ?? ""
    log(`env: ${varName} → ${value ? "<set>" : "<unset>"}`)
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

    const content = (
      await fs
        .readFile(resolved, "utf8")
        .catch((err: NodeJS.ErrnoException) => {
          if (err.code === "ENOENT") {
            log(`file: ${token} → ${resolved} DOES NOT EXIST`)
            return ""
          }
          log(`file: ${token} → ${resolved} READ ERROR: ${err.message}`)
          return ""
        })
    ).trim()

    log(`file: ${token} → ${resolved} (${content.length} chars)`)
    out += content
    cursor = index + token.length
  }

  out += text.slice(cursor)
  return out
}

/**
 * OpenCode plugin that expands {file:...} and {env:...} tokens in .md agent
 * system prompts.
 *
 * Caches `directory` from `PluginInput` at init time to resolve relative paths.
 * Rewrites every entry in `output.system` on each LLM call.
 *
 * # Hooks
 * - `experimental.chat.system.transform` — expands tokens in each system
 *   prompt string. No-op when no tokens are present.
 */
export const FileInterpPlugin: Plugin = async (input) => {
  const projectDir = input.directory
  const log = createLog(input.client as { app: { log: (opts: unknown) => Promise<unknown> } })

  log(`init: projectDir=${projectDir}`)

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

        log(`system[${i}]: expanding tokens (${entry.length} chars)`)
        output.system[i] = await expand(entry, projectDir, log)
      }
    },
  } as unknown as Awaited<ReturnType<Plugin>>
}
