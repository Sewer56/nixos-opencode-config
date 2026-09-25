/**
 * @module v2 - OpenCode V2 entry point for the prompt builder.
 *
 * V2 decodes the default export as `{id, setup}`. `setup` registers
 * `session.hook` handlers for the request kinds that carry system parts
 * (`context`, `compaction`, `generate`).
 *
 * Each handler replaces OpenCode's default base prompt with sections
 * derived from the event's per-request tool snapshot, then shortens known
 * V2 tool descriptions without changing their schemas or implementations.
 */
import { buildIntoEvent } from "./builder"
import type { SessionEvent } from "./builder"
import { capturePromptDump, writePromptDump } from "./prompt-dump"
import { shortenToolDescriptions } from "./tool-descriptions"

type SessionDomain = {
  hook?: (name: string, handler: (event: SessionEvent) => Promise<void>) => Promise<unknown>
}
type PluginContext = {
  options?: Record<string, unknown>
  location?: { directory?: string }
  session?: SessionDomain
}

const debug = process.env.PROMPT_BUILDER_DEBUG === "1"

const HOOKED_EVENTS = ["context", "compaction", "generate"] as const

function log(message: string) {
  if (debug) console.log(`[prompt-builder] ${message}`)
}

/** Extract the `supplemental: string[]` option, tolerating a raw options bag. */
function readSupplementalOption(options?: Record<string, unknown>): string[] {
  const value = options?.supplemental
  return Array.isArray(value) ? value.filter((p): p is string => typeof p === "string") : []
}

export default {
  id: "prompt-builder",

  /**
   * Rewrite the base prompt on every request that carries system parts.
   * Options come from `ctx.options`; `supplemental` lists file paths whose
   * template-expanded content becomes "# Supplemental Context" sections.
   */
  async setup(ctx: PluginContext) {
    const workingDirectory = ctx.location?.directory ?? process.cwd()
    const supplementalFiles = readSupplementalOption(ctx.options)
    const stripCore = process.env.PROMPT_BUILDER_KEEP_CORE !== "1"

    for (const name of HOOKED_EVENTS) {
      await ctx.session?.hook?.(name, async (event) => {
        const dump = process.env.PB_DUMP
        const before = dump ? capturePromptDump(event) : undefined

        const { strategy, sections } = await buildIntoEvent(event, {
          workingDirectory,
          platform: process.platform,
          supplementalFiles,
          cwd: workingDirectory,
          stripCore,
        })
        shortenToolDescriptions(event.tools)

        // A duplicate plugin load must not overwrite the original contract dump.
        if (dump && before && strategy !== "already-applied") {
          await writePromptDump(dump, name, before, event)
        }
        log(`${name}: strategy=${strategy} sections=${sections.length} tools=${Object.keys(event.tools ?? {}).length}:${Object.keys(event.tools ?? {}).join(",")}`)
      })
    }
    log(`init: workingDirectory=${workingDirectory} supplemental=${supplementalFiles.length} stripCore=${stripCore}`)
  },
}
