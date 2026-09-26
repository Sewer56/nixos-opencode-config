/** Capture the system prompt and advertised tool contracts for local diagnostics. */
import { writeFile } from "node:fs/promises"
import type { SessionEvent } from "./builder.ts"

interface PromptSnapshot {
  system: string
  tools: string
}

/**
 * Capture a request before the builder changes it.
 *
 * @param event - Request containing system parts and JSON-compatible tool schemas.
 * @returns Serialized copies that remain unchanged when the event is mutated.
 */
export function capturePromptDump(event: SessionEvent): PromptSnapshot {
  return { system: systemText(event), tools: JSON.stringify(event.tools ?? {}, null, 2) }
}

/**
 * Write before/after prompts, complete tool contracts and a character-count table.
 *
 * @param prefix - Output path prefix; its parent directory must already exist.
 * @param name - Request hook name used in output filenames.
 * @param before - Snapshot taken before rewriting the request.
 * @param event - Rewritten request.
 * @returns Resolves once all diagnostic files have been written.
 * @throws If a diagnostic file cannot be written; check the prefix and permissions.
 */
export async function writePromptDump(
  prefix: string,
  name: string,
  before: PromptSnapshot,
  event: SessionEvent,
): Promise<void> {
  const after = capturePromptDump(event)
  const sizes = Object.entries(event.tools ?? {}).map(([tool, definition]) => {
    const info = definition as { description?: string; input?: unknown } | null
    const description = String(info?.description ?? "")
    const input = JSON.stringify(info?.input ?? {})
    return `${tool}\t${description.length + input.length}\t(desc ${description.length}, input ${input.length})`
  })

  // Dumps can contain project instructions and private tool descriptions.
  const files = {
    "raw.txt": before.system,
    "final.txt": after.system,
    "tools.raw.json": before.tools,
    "tools.final.json": after.tools,
    "tools.txt": sizes.join("\n"),
  }
  await Promise.all(Object.entries(files).map(([suffix, content]) =>
    writeFile(`${prefix}.${name}.${suffix}`, content, { mode: 0o600 }),
  ))
}

function systemText(event: SessionEvent): string {
  return (event.system ?? []).map((part) => part.text ?? "").join("\n\n<<<PART>>>\n\n")
}
