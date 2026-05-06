#!/usr/bin/env bun
import fsp from "node:fs/promises"
import path from "node:path"
import { fileURLToPath } from "node:url"
import {
  expandWithDiagnostics,
  type ExpansionDiagnostic,
} from "../config/plugins/file-interp"

const SCRIPT_DIR = path.dirname(fileURLToPath(import.meta.url))
const CONFIG_DIR = path.resolve(SCRIPT_DIR, "../config")
const SKIP_DIRS = new Set([".git", ".logs", "node_modules"])
const FILE_TEMPLATE_RE = /\{\{\s*file\s*=/g
const INLINE_CONDITIONAL_RE = /\{\{\s*(?:if\s*=|endif\b)/g
const ARG_TOKEN_RE = /\{\{arg:/g
const ENV_TOKEN_RE = /\{\{env:/g

interface ValidationFailure {
  file: string
  line: number
  column: number
  message: string
}

async function main(): Promise<void> {
  const roots = process.argv.slice(2).map((arg) => path.resolve(process.cwd(), arg))
  const files = await collectTemplateFiles(roots.length ? roots : [CONFIG_DIR])
  const failures: ValidationFailure[] = []

  for (const file of files) {
    const source = await fsp.readFile(file, "utf8")
    const result = await expandWithDiagnostics(source, CONFIG_DIR)
    const rel = path.relative(CONFIG_DIR, file) || path.basename(file)

    for (const diagnostic of result.diagnostics) {
      // Skip missing-file diagnostics caused by unresolved arg tokens in paths
      // (standalone validation has no caller context, so cascaded args resolve to empty)
      if (diagnostic.kind === "missing-file" && diagnostic.message.includes("FILE_INTERP_EMPTY")) continue
      failures.push(formatDiagnostic(rel, source, diagnostic))
    }

    FILE_TEMPLATE_RE.lastIndex = 0
    let match: RegExpExecArray | null
    while ((match = FILE_TEMPLATE_RE.exec(result.text)) !== null) {
      const loc = lineColumn(result.text, match.index)
      failures.push({
        file: rel,
        line: loc.line,
        column: loc.column,
        message: "unexpanded file template remains after render",
      })
    }

    INLINE_CONDITIONAL_RE.lastIndex = 0
    while ((match = INLINE_CONDITIONAL_RE.exec(result.text)) !== null) {
      const loc = lineColumn(result.text, match.index)
      failures.push({
        file: rel,
        line: loc.line,
        column: loc.column,
        message: "unexpanded inline conditional remains after render",
      })
    }

    ARG_TOKEN_RE.lastIndex = 0
    while ((match = ARG_TOKEN_RE.exec(result.text)) !== null) {
      const loc = lineColumn(result.text, match.index)
      failures.push({
        file: rel,
        line: loc.line,
        column: loc.column,
        message: "unexpanded arg token remains after render",
      })
    }

    ENV_TOKEN_RE.lastIndex = 0
    while ((match = ENV_TOKEN_RE.exec(result.text)) !== null) {
      const loc = lineColumn(result.text, match.index)
      failures.push({
        file: rel,
        line: loc.line,
        column: loc.column,
        message: "unexpanded env token remains after render",
      })
    }
  }

  if (failures.length) {
    for (const failure of failures) {
      console.error(`${failure.file}:${failure.line}:${failure.column}: ${failure.message}`)
    }
    console.error(`file-interp validation failed: ${failures.length} issue(s) in ${files.length} file(s)`)
    process.exitCode = 1
    return
  }

  console.log(`file-interp validation passed: ${files.length} file(s)`)
}

async function collectTemplateFiles(roots: string[]): Promise<string[]> {
  const files: string[] = []
  for (const root of roots) {
    await collectTemplateFilesFrom(root, files)
  }
  files.sort()
  return files
}

async function collectTemplateFilesFrom(target: string, files: string[]): Promise<void> {
  const stat = await fsp.stat(target)
  if (stat.isFile()) {
    if (isTemplateFile(target)) files.push(target)
    return
  }
  if (!stat.isDirectory()) return

  const name = path.basename(target)
  if (SKIP_DIRS.has(name)) return

  const entries = await fsp.readdir(target, { withFileTypes: true })
  entries.sort((a, b) => a.name.localeCompare(b.name))
  for (const entry of entries) {
    const full = path.join(target, entry.name)
    if (entry.isDirectory()) {
      if (!SKIP_DIRS.has(entry.name)) await collectTemplateFilesFrom(full, files)
      continue
    }
    if (entry.isFile() && isTemplateFile(entry.name)) files.push(full)
  }
}

function isTemplateFile(file: string): boolean {
  const ext = path.extname(file)
  return ext === ".md" || ext === ".txt"
}

function formatDiagnostic(
  file: string,
  source: string,
  diagnostic: ExpansionDiagnostic,
): ValidationFailure {
  const index = locateDiagnostic(source, diagnostic)
  const loc = lineColumn(source, index)
  return {
    file,
    line: loc.line,
    column: loc.column,
    message: `${diagnostic.kind}: ${diagnostic.message}`,
  }
}

function locateDiagnostic(source: string, diagnostic: ExpansionDiagnostic): number {
  if (diagnostic.token) {
    const tokenIndex = source.indexOf(diagnostic.token)
    if (tokenIndex !== -1) return tokenIndex
  }
  if (diagnostic.rawPath) {
    const pathIndex = source.indexOf(diagnostic.rawPath)
    if (pathIndex !== -1) return pathIndex
  }
  return 0
}

function lineColumn(text: string, index: number): { line: number; column: number } {
  let line = 1
  let lineStart = 0
  for (let i = 0; i < index; i++) {
    const code = text.charCodeAt(i)
    if (code !== 10 && code !== 13) continue
    if (code === 13 && text.charCodeAt(i + 1) === 10) i++
    line++
    lineStart = i + 1
  }
  return { line, column: index - lineStart + 1 }
}

if (import.meta.main) {
  main().catch((err: unknown) => {
    console.error(err instanceof Error ? err.message : String(err))
    process.exitCode = 1
  })
}
