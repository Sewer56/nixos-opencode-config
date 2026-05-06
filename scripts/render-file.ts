#!/usr/bin/env bun
//
// Render the final expanded output of a template file.
//
// Usage:
//   bun run scripts/render-file.ts <path>              # render to stdout
//   bun run scripts/render-file.ts <path> -o <out>      # render to file
//   bun run scripts/render-file.ts agent/_docs/reviewers/clarity.md
//   bun run scripts/render-file.ts agent/_branding/reviewers/positioning.md -o /tmp/positioning.txt
//
// The script resolves paths relative to the config/ directory.
// Absolute paths are used as-is.
//

import fsp from "node:fs/promises"
import path from "node:path"
import { fileURLToPath } from "node:url"
import { expand } from "../config/plugins/file-interp"

const SCRIPT_DIR = path.dirname(fileURLToPath(import.meta.url))
const CONFIG_DIR = path.resolve(SCRIPT_DIR, "../config")

function resolveFilePath(input: string): string {
  if (path.isAbsolute(input)) return input
  return path.resolve(CONFIG_DIR, input)
}

async function main(): Promise<void> {
  const args = process.argv.slice(2)

  if (args.length === 0 || args[0] === "--help" || args[0] === "-h") {
    console.log(`Usage: bun run scripts/render-file.ts <path> [-o <output>]

Renders the fully expanded output of a template file, resolving all
{{ file= }}, {{arg:}}, {{env:}}, and {{ if= }} tokens.

Arguments:
  <path>         Template file path (relative to config/ or absolute)
  -o <output>    Write result to file instead of stdout

Examples:
  bun run scripts/render-file.ts agent/_docs/reviewers/clarity.md
  bun run scripts/render-file.ts agent/_branding/reviewers/positioning.md -o /tmp/out.txt
`)
    return
  }

  const filePath = resolveFilePath(args[0])

  // Parse -o flag
  let outputPath: string | undefined
  const oIdx = args.indexOf("-o")
  if (oIdx !== -1 && args[oIdx + 1]) {
    outputPath = path.resolve(process.cwd(), args[oIdx + 1])
  }

  // Read and expand
  const source = await fsp.readFile(filePath, "utf8")
  const expanded = await expand(source, CONFIG_DIR)

  if (outputPath) {
    await fsp.writeFile(outputPath, expanded, "utf8")
    console.error(`Wrote ${expanded.split("\n").length} lines to ${outputPath}`)
  } else {
    process.stdout.write(expanded)
  }
}

if (import.meta.main) {
  main().catch((err: unknown) => {
    console.error(err instanceof Error ? err.message : String(err))
    process.exitCode = 1
  })
}
