/** Register prompt and tool-description hooks for OpenCode requests. */
import { fileURLToPath } from "node:url"
import { buildIntoEvent, type SessionEvent } from "./src/builder.ts"
import { capturePromptDump, writePromptDump } from "./src/prompt-dump.ts"
import { shortenToolDescriptions } from "./src/tool-descriptions.ts"

type PluginContext = {
  options?: Record<string, unknown>
  location?: { directory?: string }
  session?: {
    hook?: (name: string, handler: (event: SessionEvent) => Promise<void>) => Promise<unknown>
  }
}

const debug = process.env.PROMPT_BUILDER_DEBUG === "1"
const defaultDumpPrefix = fileURLToPath(new URL("./probe", import.meta.url))

export function dumpPrefix(value: string | undefined): string | undefined {
  return value === "1" ? defaultDumpPrefix : value
}

const HOOKED_EVENTS = ["context", "compaction", "generate"] as const

function log(message: string) {
  if (debug) console.log(`[prompt-builder] ${message}`)
}

/** Ignore supplemental entries that are not file paths. */
function readSupplementalOption(options?: Record<string, unknown>): string[] {
  const value = options?.supplemental
  return Array.isArray(value) ? value.filter((p): p is string => typeof p === "string") : []
}

export default {
  id: "prompt-builder",

  /**
   * Build prompts and shorten known tool descriptions for each request.
   *
   * @param ctx - Plugin options, project directory and session hook registration.
   * @returns Resolves when all available request hooks have been registered.
   * @throws If OpenCode rejects a hook registration.
   */
  async setup(ctx: PluginContext) {
    const workingDirectory = ctx.location?.directory ?? process.cwd()
    const supplementalFiles = readSupplementalOption(ctx.options)
    const stripCore = process.env.PROMPT_BUILDER_KEEP_CORE !== "1"

    for (const name of HOOKED_EVENTS) {
      await ctx.session?.hook?.(name, async (event) => {
        const dump = dumpPrefix(process.env.PB_DUMP)
        const before = dump ? capturePromptDump(event) : undefined

        const { strategy, sections } = await buildIntoEvent(event, {
          workingDirectory,
          platform: process.platform,
          supplementalFiles,
          cwd: workingDirectory,
          stripCore,
        })
        shortenToolDescriptions(event.tools)

        if (dump && before) {
          await writePromptDump(dump, name, before, event)
        }
        if (debug) {
          const tools = Object.keys(event.tools ?? {})
          log(`${name}: strategy=${strategy} sections=${sections.length} tools=${tools.length}:${tools.join(",")}`)
        }
      })
    }
    log(`init: workingDirectory=${workingDirectory} supplemental=${supplementalFiles.length} stripCore=${stripCore}`)
  },
}
