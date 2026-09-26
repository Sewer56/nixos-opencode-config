/** Build and insert tool-specific guidance into a request's system prompt. */

import { factsFromToolKeys } from "./facts.ts"
import { buildSections } from "./sections.ts"
import { expandTemplate } from "./template.ts"

/** First line of OpenCode's default base prompt; identifies the part to replace. */
export const BASE_PROMPT_MARKER = "You are an AI agent running in OpenCode"

/** Tags builder output so a second application on the same event is a no-op. */
export const BUILDER_TAG = "<!-- prompt-builder -->"

/**
 * Identify OpenCode's own system parts by their text prefixes.
 *
 * Strip mode removes matching parts. It keeps everything else, including
 * our sections, other plugins' additions and project instructions.
 */
const CORE_PART_PREFIXES = [
  BASE_PROMPT_MARKER,
  "# Your Model",
  "When you create a worktree outside the current working directory",
  "Today's date:",
]

/** Intro line of OpenCode's environment part. */
const ENV_PART_PREFIX = "Here is some useful information about the environment"

/**
 * Trim an environment part down to whatever follows the env block and date.
 *
 * OpenCode packs `<env>`, the date, and project instructions such as
 * AGENTS.md into one system part. Strip mode removes the env block and date
 * but must keep the attached instructions.
 */
function trimEnvironmentPart(text: string): string {
  return text
    .replace(/^Here is some useful information about the environment you are running in:\n/, "")
    .replace(/<env>[\s\S]*?<\/env>\n?/, "")
    .replace(/^Today's date:.*\n?/m, "")
    .replace(/^\n+/, "")
}

/** Tag the first new part so another plugin load can detect it. */
function prependTaggedSections(system: SystemPart[] | undefined, sections: string[]): void {
  const tagged = sections.map((text, index) => (index === 0 ? `${BUILDER_TAG}\n${text}` : text))
  system?.unshift(...tagged.map((text) => ({ type: "text", text })))
}

/** A system prompt part supplied by OpenCode. */
export interface SystemPart {
  type: string
  text?: string
}

export interface SessionEvent {
  system?: SystemPart[]
  /** Available tools keyed by name, including supported aliases. */
  tools?: Record<string, unknown>
  agent?: unknown
}

/** How the builder changed the system prompt. */
export type BasePromptStrategy = "replaced" | "prepended" | "stripped" | "already-applied"

export interface BuilderInput {
  /** Working directory shown in the Environment section. */
  readonly workingDirectory: string
  /** Platform string (e.g. `process.platform`). */
  readonly platform: string
  /** File paths whose expanded content becomes Supplemental Context sections. */
  readonly supplementalFiles?: string[]
  /** Base directory for supplemental template expansion. */
  readonly cwd: string
  /** Remove OpenCode-injected parts instead of replacing the base prompt. */
  readonly stripCore?: boolean
}

/**
 * Update a request's system prompt in place.
 *
 * With `stripCore`, remove known OpenCode prompt parts and keep project
 * instructions attached to the environment part (`stripped`).
 *
 * Otherwise, remove the part with the base-prompt marker (`replaced`), or keep
 * every part if there is no marker (`prepended`). New sections go first.
 *
 * A tagged prompt is left alone (`already-applied`). Without `stripCore`,
 * the caller must supply a `system` array for sections to be inserted.
 *
 * @param event - Mutable session event (`context`, `compaction` or `generate`).
 * @param input - Environment info and optional supplemental file paths.
 * @returns The strategy used and the built section texts.
 */
export async function buildIntoEvent(
  event: SessionEvent,
  input: BuilderInput,
): Promise<{ strategy: BasePromptStrategy; sections: string[] }> {
  // OpenCode may load this plugin twice (config array plus directory
  // discovery); tagged output means this event was already built.
  const alreadyApplied = (event.system ?? []).some(
    (part) => typeof part.text === "string" && part.text.includes(BUILDER_TAG),
  )
  if (alreadyApplied) return { strategy: "already-applied", sections: [] }

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
    prependTaggedSections(event.system, sections)
    return { strategy: "stripped", sections }
  }

  let strategy: BasePromptStrategy = "prepended"
  if (baseIndex >= 0) {
    event.system!.splice(baseIndex, 1)
    strategy = "replaced"
  }
  prependTaggedSections(event.system, sections)
  return { strategy, sections }
}

/**
 * Render supplemental file entries into `{name, content}` sections.
 *
 * Each entry includes a file and expands its argument/environment tokens.
 * The section name is the file's basename without its extension.
 */
async function resolveSupplemental(
  input: BuilderInput,
): Promise<{ name: string; content: string }[] | undefined> {
  const files = input.supplementalFiles
  if (!files || files.length === 0) return undefined

  const rendered: { name: string; content: string }[] = []
  for (const file of files) {
    const content = await expandTemplate(`{{ file="${file}" }}`, { cwd: input.cwd })
    const base = file.split("/").pop() ?? file
    rendered.push({ name: base.replace(/\.[^.]+$/, ""), content })
  }
  return rendered
}
