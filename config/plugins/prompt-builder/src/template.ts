/**
 * Minimal template engine for supplemental context, self-contained so this
 * plugin carries no dependency on the md-expand package.
 *
 * Supports the token subset that supplemental files need:
 * `{{ file="path" }}` includes (relative to cwd), `{{arg:key}}` from the
 * options map, and `{{env:NAME}}` from the environment.
 *
 * Missing files expand to a visible error line instead of throwing,
 * matching md-expand's keep-the-failure-visible behavior.
 * @module prompt-builder/template
 */

export interface TemplateOptions {
  /** Base directory for relative file includes. */
  readonly cwd: string
  /** Values for `{{arg:key}}` tokens. */
  readonly args?: Record<string, string>
  /** Maximum include nesting depth; guards include cycles. */
  readonly maxDepth?: number
}

const FILE_TOKEN = /\{\{\s*file\s*=\s*"([^"]+)"\s*\}\}/g
const ARG_TOKEN = /\{\{\s*arg:([^}\s]+)\s*\}\}/g
const ENV_TOKEN = /\{\{\s*env:([A-Za-z_][A-Za-z0-9_]*)\s*\}\}/g

/**
 * Expand template tokens in `text`.
 *
 * @param text - Raw text, possibly containing tokens.
 * @param options - Cwd, arg values and depth limit.
 * @returns The text with every supported token replaced.
 */
export async function expandTemplate(text: string, options: TemplateOptions): Promise<string> {
  const expanded = await expandFiles(text, options, 0)
  return expandVars(expanded, options)
}

async function expandFiles(text: string, options: TemplateOptions, depth: number): Promise<string> {
  if (depth >= (options.maxDepth ?? 10)) return text

  const match = FILE_TOKEN.exec(text)
  if (!match) return text

  const before = text.slice(0, match.index)
  const after = text.slice(match.index + match[0].length)
  let content: string
  try {
    const { resolve } = await import("node:path")
    const { readFile } = await import("node:fs/promises")
    content = await readFile(resolve(options.cwd, match[1]!), "utf8")
  } catch (err) {
    content = `{{ file="${match[1]}" }}: unreadable (${(err as Error).message})`
  }

  const expandedAfter = await expandFiles(after, options, depth + 1)
  return before + content + expandedAfter
}

function expandVars(text: string, options: TemplateOptions): string {
  return text
    .replace(ARG_TOKEN, (_, key: string) => options.args?.[key] ?? "")
    .replace(ENV_TOKEN, (_, name: string) => process.env[name] ?? "")
}
