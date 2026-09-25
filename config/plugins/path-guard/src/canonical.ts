import { readlink, realpath } from "node:fs/promises"
import path from "node:path"

/** Permission denial raised by path resolution; the message reaches the model. */
export class GuardDeny extends Error {}

function errorCode(error: unknown): string | undefined {
  return (error as NodeJS.ErrnoException | undefined)?.code
}

async function realpathOrNull(target: string): Promise<string | null> {
  try {
    return await realpath(target)
  } catch (error) {
    if (errorCode(error) === "ENOENT") return null
    // ELOOP and similar failures matter for authorization: fail closed.
    throw new GuardDeny(`cannot resolve ${target}: ${errorCode(error) ?? "unknown error"}`)
  }
}

/**
 * Resolve a permission target to its physical path.
 *
 * Missing suffixes are appended to the nearest existing ancestor, so a file
 * that does not exist yet still canonicalizes. A symlink whose target is
 * gone fails closed: a dangling link is not a missing leaf. This is an
 * authorization snapshot; it does not defend against concurrent symlink
 * replacement.
 *
 * @throws GuardDeny when resolution fails or a dangling symlink is found.
 */
export async function canonicalPath(target: string): Promise<string> {
  let current = path.resolve(target)
  const missing: string[] = []

  for (;;) {
    const resolved = await realpathOrNull(current)
    if (resolved !== null) return path.join(resolved, ...missing.reverse())

    // ENOENT with a readable link target means the link itself dangles.
    let isLink = false
    try {
      await readlink(current)
      isLink = true
    } catch (error) {
      const code = errorCode(error)
      if (code !== "ENOENT" && code !== "EINVAL") {
        throw new GuardDeny(`cannot inspect ${current}: ${code ?? "unknown error"}`)
      }
    }
    if (isLink) throw new GuardDeny(`dangling symlink at ${current}`)

    const parent = path.dirname(current)
    if (parent === current) throw new GuardDeny(`cannot resolve permission path: ${target}`)
    missing.push(path.basename(current))
    current = parent
  }
}
