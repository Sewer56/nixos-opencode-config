/** Add guidance for the available tools to each request's system prompt. */

import { readFile } from "node:fs/promises"
import { resolve } from "node:path"
import { factsFromToolKeys } from "./facts.ts"
import { buildSections } from "./sections.ts"

/** Text used to find OpenCode's base prompt. */
export const BASE_PROMPT_MARKER = "You are an AI agent running in OpenCode"

/**
 * These prefixes identify the OpenCode parts removed in strip mode.
 *
 * Other parts stay in place, including additions from plugins and project
 * instructions.
 */
const CORE_PART_PREFIXES = [
  BASE_PROMPT_MARKER,
  "# Your Model",
  "When you create a worktree outside the current working directory",
  "Today's date:",
]

/** Text used to find OpenCode's environment part. */
const ENV_PART_PREFIX = "Here is some useful information about the environment"

/**
 * Remove OpenCode's environment details without losing project instructions.
 *
 * OpenCode may put the `<env>` block, date and instructions such as AGENTS.md
 * in the same part. Keep any instructions that follow the environment details.
 */
function trimEnvironmentPart(text: string): string {
  return text
    .replace(/^Here is some useful information about the environment you are running in:\n/, "")
    .replace(/<env>[\s\S]*?<\/env>\n?/, "")
    .replace(/^Today's date:.*\n?/m, "")
    .replace(/^\n+/, "")
}

/** Put new sections ahead of the existing system prompt parts. */
function prependSections(system: SystemPart[] | undefined, sections: string[]): void {
  system?.unshift(...sections.map((text) => ({ type: "text", text })))
}

/** One part of the system prompt supplied by OpenCode. */
export interface SystemPart {
  type: string
  text?: string
}

/** The request data this builder reads and updates. */
export interface SessionEvent {
  system?: SystemPart[]
  /** Tools available for this request, keyed by name or alias. */
  tools?: Record<string, unknown>
  agent?: unknown
}

/** Whether the builder replaced, prepended to or stripped OpenCode's prompt. */
export type BasePromptStrategy = "replaced" | "prepended" | "stripped"

/** The environment details and options used to build prompt sections. */
export interface BuilderInput {
  /** Working directory to show in the Environment section. */
  readonly workingDirectory: string
  /** Platform to show in the Environment section, such as `process.platform`. */
  readonly platform: string
  /** Files to include in Supplemental Context. */
  readonly supplementalFiles?: string[]
  /** Directory used to resolve relative supplemental file paths. */
  readonly cwd: string
  /** Remove OpenCode's prompt parts instead of just its base prompt. */
  readonly stripCore?: boolean
}

/**
 * Add sections to a request's system prompt.
 *
 * With `stripCore`, remove known OpenCode parts but keep project
 * instructions attached to the environment part. This returns `stripped`.
 *
 * Otherwise, replace OpenCode's base prompt if present (`replaced`). If it is
 * missing, leave the existing parts alone (`prepended`). New sections go first.
 *
 * The function updates `event.system` in place. Without `stripCore`, provide a
 * `system` array if you want the new sections inserted into the event.
 *
 * @param event - Request event to update (`context`, `compaction` or `generate`).
 * @param input - Environment details and optional files to include.
 * @returns The strategy used and the text of the new sections.
 */
export async function buildIntoEvent(
  event: SessionEvent,
  input: BuilderInput,
): Promise<{ strategy: BasePromptStrategy; sections: string[] }> {
  const supplemental = await resolveSupplemental(input)

  const sections = buildSections({
    facts: factsFromToolKeys(Object.keys(event.tools ?? {})),
    workingDirectory: input.workingDirectory,
    platform: input.platform,
    supplemental,
  })

  const baseIndex = (event.system ?? []).findIndex(
    (part) => typeof part.text === "string" && part.text.includes(BASE_PROMPT_MARKER),
  )

  if (input.stripCore) {
    const kept: SystemPart[] = []
    for (const part of event.system ?? []) {
      const text = typeof part.text === "string" ? part.text : ""
      if (CORE_PART_PREFIXES.some((prefix) => text.startsWith(prefix))) continue
      if (text.startsWith(ENV_PART_PREFIX)) {
        const remainder = trimEnvironmentPart(text)
        if (remainder) kept.push({ ...part, text: remainder })
        continue
      }
      kept.push(part)
    }
    event.system = kept
    prependSections(event.system, sections)
    return { strategy: "stripped", sections }
  }

  let strategy: BasePromptStrategy = "prepended"
  if (baseIndex >= 0) {
    event.system!.splice(baseIndex, 1)
    strategy = "replaced"
  }
  prependSections(event.system, sections)
  return { strategy, sections }
}

/**
 * Read supplemental files for the prompt's Supplemental Context section.
 *
 * Use each file's name without its extension as a heading. If a file cannot
 * be read, include an error message in place of its text.
 */
async function resolveSupplemental(
  input: BuilderInput,
): Promise<{ name: string; content: string }[] | undefined> {
  const files = input.supplementalFiles
  if (!files || files.length === 0) return undefined

  const rendered: { name: string; content: string }[] = []
  for (const file of files) {
    let content: string
    try {
      content = await readFile(resolve(input.cwd, file), "utf8")
    } catch (error) {
      content = `${file}: unreadable (${(error as Error).message})`
    }
    const base = file.split("/").pop() ?? file
    rendered.push({ name: base.replace(/\.[^.]+$/, ""), content })
  }
  return rendered
}
