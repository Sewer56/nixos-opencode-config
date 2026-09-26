/**
 * Put together the prompt sections for a request.
 *
 * Each section starts with a `#` heading. The caller adds the sections to the
 * system prompt.
 */

import type { ToolFacts } from "./facts.ts";
import { buildCommonRules, hasCommonRules } from "./facts.ts";

/** What to include when building the prompt sections. */
export interface SectionInput {
  /** Which tools this request can use. */
  readonly facts: ToolFacts;
  /** Working directory to show in the Environment section. */
  readonly workingDirectory: string;
  /** Platform to show in the Environment section, such as `process.platform`. */
  readonly platform: string;
  /** Extra text to add under Supplemental Context, with a heading for each entry. */
  readonly supplemental?: { name: string; content: string }[];
}

/**
 * Build the sections to add to a request's system prompt.
 *
 * Every request gets an Environment section. Tool Usage Guidelines appears
 * only when there is guidance for the available tools. Each supplemental
 * entry gets its own `##` heading under Supplemental Context.
 *
 * @param input - Available tools, environment details and any extra text.
 * @returns The sections in prompt order, one string per `#` heading.
 */
export function buildSections(input: SectionInput): string[] {
  const sections: string[] = [];

  // Environment
  sections.push(
    [
      "# Environment",
      `Working directory: ${input.workingDirectory}`,
      `Platform: ${input.platform}`,
    ].join("\n"),
  );

  // Tool Usage Guidelines
  const toolSections = buildToolSections(input.facts);
  const commonRules = hasCommonRules(input.facts) ? buildCommonRules(input.facts) : "";
  if (toolSections.length > 0 || commonRules) {
    const parts: string[] = ["# Tool Usage Guidelines"];
    if (commonRules) parts.push("## Common Rules\n" + commonRules);
    parts.push(...toolSections);
    sections.push(parts.join("\n\n"));
  }

  // Supplemental Context
  if (input.supplemental && input.supplemental.length > 0) {
    const parts = ["# Supplemental Context"];
    for (const s of input.supplemental) {
      parts.push(`## ${s.name}\n${s.content}`);
    }
    sections.push(parts.join("\n\n"));
  }

  // Remove extra blank lines and trailing whitespace.
  return sections.map((s) => s.replace(/\n{3,}/g, "\n\n").trimEnd());
}

function buildToolSections(facts: ToolFacts): string[] {
  const sections: string[] = [];
  if (facts.has_shell) sections.push("## `Shell` Tool\n" + buildShellSection());
  if (facts.has_read) sections.push("## `Read` Tool\n" + buildReadSection(facts));
  if (facts.has_write) sections.push("## `Write` Tool\n" + buildWriteSection(facts));
  if (facts.has_edit) sections.push("## `Edit` Tool\n" + buildEditSection());
  if (facts.has_glob) sections.push("## `Glob` Tool\n" + buildGlobSection());
  if (facts.has_grep) sections.push("## `Grep` Tool\n" + buildGrepSection(facts));
  if (facts.has_webfetch) sections.push("## `WebFetch` Tool\n" + buildWebFetchSection());
  if (facts.has_websearch) sections.push("## `WebSearch` Tool\n" + buildWebSearchSection());
  if (facts.has_todowrite) sections.push("## `TodoWrite` Tool\n" + buildTodoWriteSection());
  if (facts.has_subagent) sections.push("## `Subagent` Tool\n" + buildSubagentSection(facts));
  if (facts.has_question) sections.push("## `Question` Tool\n" + buildQuestionSection());
  if (facts.has_lsp) sections.push("## `LSP` Tool\n" + buildLspSection());
  if (facts.has_patch) sections.push("## `ApplyPatch` Tool\n" + buildPatchSection());
  return sections;
}

function buildShellSection(): string {
  return [
    "- Use it for terminal work (git, package managers, test runners, docker) and shell-native search/filter jobs the specialized tools do not handle well.",
    "- Output combines stdout and stderr; non-zero exit codes are reported.",
    "- For independent commands, make parallel `shell` calls. For dependent commands, use one call with `&&`.",
    "- Quote paths that contain spaces.",
  ].join("\n");
}

function buildReadSection(facts: ToolFacts): string {
  const lines: string[] = [];
  lines.push("- Returns `{n}: text`. Lines over 2000 chars are truncated.");
  if (facts.has_glob && facts.has_shell) {
    lines.push(
      "- Reads files and directories. Use `glob` to find files or `shell` for directory listings.",
    );
  } else if (facts.has_glob) {
    lines.push("- Reads files and directories. Use `glob` to find files.");
  } else if (facts.has_shell) {
    lines.push("- Reads files and directories. Use `shell` for directory listings.");
  } else {
    lines.push("- Reads files and directories.");
  }
  lines.push("- Missing files return an error. Images and PDFs are supported; other binary files are rejected.");
  lines.push("- Read related files in parallel when useful.");
  return lines.join("\n");
}

function buildWriteSection(facts: ToolFacts): string {
  const lines = ["- Existing files are overwritten."];
  if (!facts.has_edit) {
    lines.push("- Use this for new files or full rewrites, not small edits.");
  }
  return lines.join("\n");
}

function buildEditSection(): string {
  return "- `oldString` must be non-empty, differ from `newString`, and match once unless `replaceAll` is true.";
}

function buildGlobSection(): string {
  return [
    "- Supports *, **, ?, [abc], and {a,b}.",
    "- Returns absolute paths, without sorting by modification time.",
    "- Defaults to 100 results; set `limit` to change the cap. Excess matches are truncated.",
  ].join("\n");
}

function buildGrepSection(facts: ToolFacts): string {
  const lines: string[] = [];
  lines.push(
    "- `pattern` must not be empty. Search is single-line only; there is no multiline matching.",
  );
  lines.push("- Returns matches grouped by file.");
  if (facts.has_shell) {
    lines.push("- Use this instead of shell `grep`/`rg`.");
  }
  if (!facts.has_glob && !facts.has_read) {
    lines.push("- Use it for content search, not file-name search or full-file inspection.");
  }
  return lines.join("\n");
}

function buildWebFetchSection(): string {
  return [
    "- Fetches a URL and returns the page converted to Markdown text.",
    "- Use it for documentation or reference pages cited in the task.",
  ].join("\n");
}

function buildWebSearchSection(): string {
  return [
    "- Runs a web search and returns summarized results with their sources.",
    "- Prefer it over `webfetch` when you do not have a specific URL.",
  ].join("\n");
}

function buildTodoWriteSection(): string {
  return [
    "- Replaces the whole todo list; send the complete list every time.",
    "- Use it to track multi-step work and to report progress state to the user.",
  ].join("\n");
}

function buildSubagentSection(facts: ToolFacts): string {
  const lines = [
    "- Delegate substantive work. New sessions need full context; pass the returned `sessionID` to resume one.",
  ];
  const localTools: string[] = [];
  if (facts.has_read) localTools.push("read");
  if (facts.has_glob) localTools.push("glob");
  if (facts.has_grep) localTools.push("grep");
  if (localTools.length > 0) {
    lines.push(
      `- Do not use it when \`${localTools.join("`, `")}\` on one or a few files is enough.`,
    );
  }
  lines.push("- Results are private to you; summarize for the user.");
  return lines.join("\n");
}

function buildQuestionSection(): string {
  return "- Ask one or more questions to resolve unclear requirements or get user decisions.";
}

function buildLspSection(): string {
  return "- Query language-server diagnostics, definitions or hover information for workspace files.";
}

function buildPatchSection(): string {
  return "- Apply a patch to files using the `*** Begin Patch` format. Use this instead of edit/write for GPT models.";
}
