import path from "node:path"
import { canonicalPath, GuardDeny } from "./canonical"
import { decideExternal, loadExternalDirectoryRules } from "./rules"

const MUTATION_TOOLS = new Set(["write", "edit", "apply_patch", "multiedit"])
const DIRECT_KEYS = ["path", "filePath", "destination", "newPath"]
const NESTED_KEYS = ["edits", "moves", "patches", "operations"]
const PATH_KEYS = ["path", "filePath", "from", "to", "destination", "newPath"]

export type ExecuteBeforeEvent = {
  tool: string
  input?: Record<string, unknown>
}

function pushString(targets: string[], value: unknown): void {
  if (typeof value === "string") targets.push(value)
}

/** Collect write-class target paths from a tool input payload. */
export function extractTargets(input: Record<string, unknown>): string[] {
  const targets: string[] = []

  for (const key of DIRECT_KEYS) pushString(targets, input[key])
  for (const key of NESTED_KEYS) {
    const group = input[key]
    if (!Array.isArray(group)) continue
    for (const item of group) {
      if (!item || typeof item !== "object") continue
      for (const pathKey of PATH_KEYS) pushString(targets, (item as Record<string, unknown>)[pathKey])
    }
  }

  return targets
}

function denyMessage(raw: string, reason: string): string {
  return `path-guard: refusing ${raw}: ${reason}`
}

/**
 * Build the execute.before handler enforcing canonical path policy.
 *
 * Relative targets resolve against the instance directory (the daemon cwd
 * differs from the session project). In-worktree targets pass; OpenCode's
 * own permission rules already apply to them.
 *
 * Targets resolving outside the worktree must match an external_directory
 * allow rule. Deny, ask and unmatched targets throw so the tool call aborts
 * before execution.
 */
export function makeGuard(
  configFile: string,
  directory: string,
  root: Promise<string>,
  debug = false,
) {
  return async function guard(event: ExecuteBeforeEvent): Promise<void> {
    if (!MUTATION_TOOLS.has(event.tool)) return
    const rawTargets = extractTargets(event.input ?? {})
    if (rawTargets.length === 0) return

    const worktreeRoot = await root
    const rules = await loadExternalDirectoryRules(configFile)

    for (const relative of rawTargets) {
      const raw = path.isAbsolute(relative) ? relative : path.resolve(directory, relative)
      let canonical: string
      try {
        canonical = await canonicalPath(raw)
      } catch (error) {
        const reason = error instanceof GuardDeny ? error.message : String(error)
        throw new Error(denyMessage(raw, reason))
      }

      const insideWorktree =
        canonical === worktreeRoot || canonical.startsWith(`${worktreeRoot}${path.sep}`)
      if (insideWorktree) {
        if (debug) console.error(`path-guard: allow in-worktree ${raw} -> ${canonical}`)
        continue
      }

      const decision = decideExternal(canonical, rules)
      if (decision.effect === "deny") throw new Error(denyMessage(raw, decision.reason))
      if (debug) console.error(`path-guard: allow external ${raw} -> ${canonical}`)
    }
  }
}
