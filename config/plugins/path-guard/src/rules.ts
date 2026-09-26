import { readFile, stat } from "node:fs/promises"

export type RuleEffect = "allow" | "ask" | "deny"
export type ExternalRules = Record<string, RuleEffect>

export type Decision = { effect: "allow" } | { effect: "deny"; reason: string }

/** Remove comments outside string literals and trailing commas. */
export function stripJsonc(text: string): string {
  let out = ""
  let index = 0
  let inString = false

  while (index < text.length) {
    const char = text[index]
    const next = text[index + 1]

    if (inString) {
      out += char
      if (char === "\\") {
        out += next ?? ""
        index += 2
        continue
      }
      if (char === '"') inString = false
      index += 1
      continue
    }

    if (char === '"') {
      inString = true
      out += char
      index += 1
      continue
    }
    if (char === "/" && next === "/") {
      while (index < text.length && text[index] !== "\n") index += 1
      continue
    }
    if (char === "/" && next === "*") {
      index += 2
      while (index < text.length && !(text[index] === "*" && text[index + 1] === "/")) index += 1
      index += 2
      continue
    }
    if (char === "}" || char === "]") {
      out = out.replace(/,\s*$/, "")
      out += char
      index += 1
      continue
    }

    out += char
    index += 1
  }

  return out
}

const REGEX_SPECIALS = new Set([".", "+", "^", "$", "(", ")", "[", "]", "{", "}", "|", "\\"])

/** Translate an external_directory glob (`*`, `**`, `?`) into an anchored RegExp. */
export function globToRegExp(pattern: string): RegExp {
  let source = "^"
  for (let index = 0; index < pattern.length; index += 1) {
    const char = pattern[index]
    if (char === "*") {
      if (pattern[index + 1] === "*") {
        source += ".*"
        index += 1
      } else {
        source += "[^/]*"
      }
      continue
    }
    if (char === "?") {
      source += "[^/]"
      continue
    }
    source += REGEX_SPECIALS.has(char) ? `\\${char}` : char
  }
  return new RegExp(`${source}$`)
}

const regexCache = new Map<string, RegExp>()

function matcher(pattern: string): RegExp {
  let regex = regexCache.get(pattern)
  if (!regex) {
    regex = globToRegExp(pattern)
    regexCache.set(pattern, regex)
  }
  return regex
}

/**
 * Decide one external target from the external_directory rule map.
 *
 * Deny wins over every matching rule. Otherwise the last matching rule wins,
 * so an explicit ask can protect a file inside a broadly allowed directory.
 * Ask and unmatched targets fail closed because the guard cannot prompt.
 */
export function decideExternal(target: string, rules: ExternalRules): Decision {
  const matches: Array<[string, RuleEffect]> = []
  for (const [pattern, effect] of Object.entries(rules)) {
    if (effect !== "allow" && effect !== "ask" && effect !== "deny") continue
    if (matcher(pattern).test(target)) matches.push([pattern, effect])
  }

  if (matches.some(([, effect]) => effect === "deny")) {
    const pattern = matches.find(([, effect]) => effect === "deny")![0]
    return { effect: "deny", reason: `canonical target ${target} matches deny rule ${pattern}` }
  }
  if (matches.at(-1)?.[1] === "allow") return { effect: "allow" }
  return {
    effect: "deny",
    reason: `canonical target ${target} is outside the worktree with no allow rule (ask/unmatched fail closed)`,
  }
}

let cache: { file: string; mtimeMs: number; rules: ExternalRules } | null = null

/** Load external path rules from the ordered permissions array. */
export async function loadExternalDirectoryRules(configFile: string): Promise<ExternalRules> {
  const mtimeMs = (await stat(configFile)).mtimeMs
  if (cache && cache.file === configFile && cache.mtimeMs === mtimeMs) return cache.rules

  const parsed = JSON.parse(stripJsonc(await readFile(configFile, "utf8")))
  const rules: ExternalRules = {}
  if (Array.isArray(parsed?.permissions)) {
    for (const rule of parsed.permissions) {
      if (rule?.action !== "external_directory" || typeof rule.resource !== "string") continue
      if (rule.effect === "allow" || rule.effect === "ask" || rule.effect === "deny") {
        rules[rule.resource] = rule.effect
      }
    }
  }

  cache = { file: configFile, mtimeMs, rules }
  return rules
}
