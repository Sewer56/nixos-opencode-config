/** Shorten known tool descriptions without changing their input schemas. */

// OpenCode v2.0.16, commit 3a103fe0aff726a4edc7492f03f7b88195d9e4c9.
// Only replace a complete match. Keep unfamiliar wording until it has been checked.
const descriptions = {
  read: {
    original: [
      "Read the contents of a file or directory. Supports text files, images, and PDFs.",
      "Images and PDFs are presented directly to the model.",
      "Each text line is prefixed by its 1-based line number as <line>: <content>.",
      "The prefix is for reference and is not part of the file content.",
      "Directory entries are returned one per line.",
      "Use offset and limit to read large files or directories in sections.",
      "Prefer one larger read over many small slices, and use grep to find specific content in large files.",
    ].join(" "),
    concise: [
      "Read text files, directories, images or PDFs. Images and PDFs are shown directly.",
      "Text lines have 1-based line-number prefixes that are not part of the file. Directory entries use one line each.",
      "Use offset and limit to read in sections. Prefer larger reads; use grep to find specific content.",
    ].join(" "),
  },
  write: {
    original: [
      "Writes a file to the local filesystem, overwriting if one exists.",
      "Missing parent directories are created automatically.",
      "Use this tool to create new files or overwrite existing files. For partial changes, use the edit tool instead.",
    ].join("\n\n"),
    concise: "Create or overwrite a file, creating missing parent directories. Use edit for partial changes.",
  },
  edit: {
    original: [
      "Edit the contents of a file by finding and replacing exact text.",
      "When editing text from Read output, preserve the exact indentation (tabs or spaces) and omit the line-number prefix, such as `1: `.",
      "Never include the prefix in oldString or newString.",
      "The edit fails if oldString is not found.",
      "By default, oldString must identify a UNIQUE location.",
      "Multiple matches FAIL unless replaceAll is true.",
      "Add more surrounding context to disambiguate, or set replaceAll to true to replace every occurrence.",
      "Use replaceAll when the change should apply to every occurrence, such as renaming a variable.",
    ].join(" "),
    concise: [
      "Replace matching text. Preserve indentation and omit Read line-number prefixes.",
      "oldString must exist and match once unless replaceAll is true. Add context for a unique match, or use replaceAll for every occurrence.",
    ].join(" "),
  },
  glob: {
    original: 'Search file paths using a glob pattern (examples: "**/*.ts", "src/**/*.tsx").',
    concise: "Find file paths matching a glob pattern, such as **/*.ts or src/**/*.tsx.",
  },
  grep: {
    original: [
      "Search file contents using ripgrep's regular expression syntax or literal text matching.",
      "Use it to locate specific code, symbols, or text patterns, and narrow searches with `path` or `include`.",
      "Returns matching file paths, line numbers, and line previews.",
    ].join(" "),
    concise: [
      "Search file contents with ripgrep regular expressions or literal text. Narrow the search with path or include.",
      "Returns file paths, line numbers and previews.",
    ].join(" "),
  },
  question: {
    original: [
      "Use this tool when you need to ask the user questions during execution. This allows you to:",
      "1. Gather user preferences or requirements",
      "2. Clarify ambiguous instructions",
      "3. Get decisions on implementation choices as you work",
      "4. Offer choices to the user about what direction to take.",
      "",
      "Usage notes:",
      '- A "Type your own answer" option is added automatically; don\'t include a separate option for free form answers',
      "- Set `multiple: true` to allow selecting more than one option",
      '- If you recommend a specific option, make that the first option in the list and add "(Recommended)" at the end of the label',
    ].join("\n"),
    concise: [
      "Ask the user questions with choices; free-form answers are added automatically.",
      "Set multiple to true to allow more than one choice.",
      'Put the recommended option first and end its label with "(Recommended)".',
    ].join(" "),
  },
} as const
const descriptionEntries = Object.entries(descriptions)

const shellOpening = "Execute a shell command and return its output."
const shellRemainder = [
  "Quote file paths containing spaces or special characters.",
  "Prefer dedicated tools over shell commands when possible.",
  "When output is large, the full result is saved to a file and a truncated preview is returned.",
  "Rely on automatic truncation unless filtering the output is more useful.",
  "Commands accept an optional timeout, background commands have no timeout by default.",
  "Background commands return immediately, and you will be notified when they complete.",
].join(" ")
const shellOriginal = `${shellOpening} ${shellRemainder}`
const shellConcise = [
  "Quote paths with spaces or special characters; prefer dedicated tools.",
  "Large output returns a preview and saves the full output to a file. Filter only when useful.",
  "Set a timeout if needed. Background commands return immediately, have no timeout by default and notify you when they finish.",
].join(" ")

/**
 * Shorten recognized descriptions in place; leave unknown wording unchanged.
 *
 * The shell's runtime OS/shell sentence is kept verbatim. Subagent descriptions,
 * tool objects and input schemas are not replaced. Repeated calls are harmless.
 *
 * @param tools - Mutable tool definitions from a request; may be absent.
 * @returns No value. Only recognized tools' description fields are updated.
 */
export function shortenToolDescriptions(tools?: Record<string, unknown>): void {
  if (!tools) return

  for (const [name, { original, concise }] of descriptionEntries) {
    const tool = tools[name]
    if (isTool(tool) && tool.description === original) tool.description = concise
  }

  const shell = tools.shell
  if (isTool(shell)) shell.description = shortenShell(shell.description)
}

function isTool(value: unknown): value is { description: string } {
  return typeof value === "object" && value !== null &&
    "description" in value && typeof value.description === "string"
}

function shortenShell(description: string): string {
  let environment = ""
  if (description !== shellOriginal) {
    // The built-in shell hook adds this sentence before user plugin hooks run.
    if (!description.startsWith(`${shellOpening} `) ||
        !description.endsWith(` ${shellRemainder}`)) return description

    environment = description.slice(shellOpening.length + 1, -shellRemainder.length - 1)
    if (!/^Commands run on [^.\r\n]+ using [^.\r\n]+\.$/.test(environment)) return description
  }

  return `Run a shell command and return its output. ${environment ? `${environment} ` : ""}${shellConcise}`
}
