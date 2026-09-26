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
const ENV_PART_PREFIX = "Here is some useful information about the environment you are running in:\n"
const ENV_BLOCK = /^<env>\n[\s\S]*?\n<\/env>\n(?:\n)?Today's date:[^\n]*(?:\n|$)/

/** Check the parts we need before removing any prompt text. */
function unrecognizedLayout(system: SystemPart[] | undefined, stripCore: boolean): string | undefined {
  if (!system) return "system prompt is missing"

  const base = system.filter((part) => part.type === "text" && part.text?.startsWith(BASE_PROMPT_MARKER))
  if (base.length !== 1) return `expected one OpenCode base prompt, found ${base.length}`
  if (!stripCore) return undefined

  const env = system.filter((part) => part.type === "text" && part.text?.startsWith(ENV_PART_PREFIX))
  if (env.length !== 1 || !ENV_BLOCK.test(env[0]!.text!.slice(ENV_PART_PREFIX.length))) {
    return "unrecognized OpenCode environment part"
  }
  return undefined
}

/**
 * Remove OpenCode's environment details without losing project instructions.
 *
 * OpenCode may put the `<env>` block, date and instructions such as AGENTS.md
 * in the same part. Keep any instructions that follow the environment details.
 */
function trimEnvironmentPart(text: string): string {
  return text.slice(ENV_PART_PREFIX.length).replace(ENV_BLOCK, "").replace(/^\n+/, "")
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

/** Whether the builder replaced, stripped or left OpenCode's prompt alone. */
export type BasePromptStrategy = "replaced" | "stripped" | "skipped"

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
 * Otherwise, replace OpenCode's base prompt (`replaced`). If required parts
 * are unrecognized, leave the event untouched (`skipped`) and report why.
 *
 * The function updates `event.system` in place only for recognized layouts.
 *
 * @param event - Request event to update (`context`, `compaction` or `generate`).
 * @param input - Environment details and optional files to include.
 * @returns The strategy, new sections and a reason when rewriting was skipped.
 */
export async function buildIntoEvent(
  event: SessionEvent,
  input: BuilderInput,
): Promise<{ strategy: BasePromptStrategy; sections: string[]; warning?: string }> {
  const warning = unrecognizedLayout(event.system, !!input.stripCore)
  if (warning) return { strategy: "skipped", sections: [], warning }

  const supplemental = await resolveSupplemental(input)

  const sections = buildSections({
    facts: factsFromToolKeys(Object.keys(event.tools ?? {})),
    workingDirectory: input.workingDirectory,
    platform: input.platform,
    supplemental,
  })

  const baseIndex = (event.system ?? []).findIndex(
    (part) => part.type === "text" && part.text?.startsWith(BASE_PROMPT_MARKER),
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

  event.system!.splice(baseIndex, 1)
  prependSections(event.system, sections)
  return { strategy: "replaced", sections }
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
