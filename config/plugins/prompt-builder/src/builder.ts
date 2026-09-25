/**
 * Applies the tool-conditional prompt builder to one V2 session event.
 * @module prompt-builder/builder
 */

import { factsFromToolKeys } from "./facts"
import { buildSections } from "./sections"
import { expandTemplate } from "./template"

/** First line of OpenCode 2's default base prompt; identifies the part to replace. */
export const BASE_PROMPT_MARKER = "You are an AI agent running in OpenCode"

/** Tags builder output so a second application on the same event is a no-op. */
export const BUILDER_TAG = "<!-- prompt-builder -->"

/**
 * System parts OpenCode 2 injects on its own; strip mode removes them.
 * Matched by text prefix. Anything unlisted (our sections, other plugins'
 * additions, project instructions such as AGENTS.md) is kept.
 */
const CORE_PART_PREFIXES = [
  BASE_PROMPT_MARKER,
  "# Your Model",
  "When you create a worktree outside the current working directory",
  "Today's date:",
]

/** Intro line of OpenCode 2's environment part. */
const ENV_PART_PREFIX = "Here is some useful information about the environment"

/**
 * Trim an environment part down to whatever follows the env block and date.
 *
 * OpenCode 2 packs `<env>`, the date, and project instructions such as
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

/** Minimal structural types for the V2 session events we mutate. */
export interface SystemPart {
  type: string
  text?: string
}

export interface SessionEvent {
  system?: SystemPart[]
  /** Tool snapshot keyed by tool name; V1 and V2 spellings both accepted. */
  tools?: Record<string, unknown>
  agent?: unknown
}

/** Which base-prompt strategy a builder invocation used. */
export type BasePromptStrategy = "replaced" | "prepended" | "stripped" | "already-applied"

export interface BuilderInput {
  /** Working directory announced in the Environment section. */
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
 * Replace the default base prompt with builder sections, in place.
 *
 * When a system part contains the base-prompt marker, that part is removed
 * and the sections replace it (`replaced`).
 *
 * Without the marker nothing is deleted and the sections are prepended
 * instead (`prepended`), so agents with their own `system` prompt keep it.
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
    const tagged = sections.map((text, index) => (index === 0 ? `${BUILDER_TAG}\n${text}` : text))
    event.system.unshift(...tagged.map((text) => ({ type: "text", text })))
    return { strategy: "stripped", sections }
  }

  let strategy: BasePromptStrategy = "prepended"
  if (baseIndex >= 0) {
    event.system!.splice(baseIndex, 1)
    strategy = "replaced"
  }
  const tagged = sections.map((text, index) => (index === 0 ? `${BUILDER_TAG}\n${text}` : text))
  event.system?.unshift(...tagged.map((text) => ({ type: "text", text })))
  return { strategy, sections }
}

/**
 * Render supplemental file entries into `{name, content}` sections.
 *
 * Each entry is a file path whose content goes through the template engine
 * (so files may themselves contain includes); the section name is the
 * basename without extension.
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
