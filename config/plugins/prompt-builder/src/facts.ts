/** Track the tools and prompt features available for a request. */

/** Which tools the current request can use. */
export interface ToolFacts {
  readonly has_shell: boolean;
  readonly has_read: boolean;
  readonly has_write: boolean;
  readonly has_edit: boolean;
  readonly has_glob: boolean;
  readonly has_grep: boolean;
  readonly has_webfetch: boolean;
  readonly has_websearch: boolean;
  readonly has_todowrite: boolean;
  readonly has_subagent: boolean;
  readonly has_question: boolean;
  readonly has_lsp: boolean;
  readonly has_patch: boolean;
}

/** Match tool names, including aliases, to their flags. */
const TOOL_KEY_MAP: Record<string, keyof ToolFacts> = {
  bash: "has_shell",
  shell: "has_shell",
  read: "has_read",
  write: "has_write",
  edit: "has_edit",
  glob: "has_glob",
  grep: "has_grep",
  webfetch: "has_webfetch",
  websearch: "has_websearch",
  todowrite: "has_todowrite",
  task: "has_subagent",
  subagent: "has_subagent",
  question: "has_question",
  lsp: "has_lsp",
  apply_patch: "has_patch",
  patch: "has_patch",
};

/**
 * Record which tools appear in a session event's `tools` object.
 *
 * When `patch` or `apply_patch` is available, leave out `edit` and `write`
 * guidance so the prompt recommends only one way to change files.
 *
 * @param keys - Tool names from the request.
 * @returns Availability flags for all known tools.
 */
export function factsFromToolKeys(keys: Iterable<string>): ToolFacts {
  const facts = {
    has_shell: false,
    has_read: false,
    has_write: false,
    has_edit: false,
    has_glob: false,
    has_grep: false,
    has_webfetch: false,
    has_websearch: false,
    has_todowrite: false,
    has_subagent: false,
    has_question: false,
    has_lsp: false,
    has_patch: false,
  };
  for (const key of keys) {
    const fact = TOOL_KEY_MAP[key];
    if (fact) facts[fact] = true;
  }
  if (facts.has_patch) {
    facts.has_edit = false;
    facts.has_write = false;
  }
  return facts;
}

/**
 * Check whether the available tools need any shared guidance.
 *
 * @param facts - Tools available in the request.
 * @returns Whether at least one shared rule applies.
 */
export function hasCommonRules(facts: ToolFacts): boolean {
  // Shell and a file tool
  if (
    facts.has_shell &&
    (facts.has_read || facts.has_edit || facts.has_write || facts.has_glob || facts.has_grep)
  ) {
    return true;
  }
  // More than one way to find or read files
  if (facts.has_glob && facts.has_grep) return true;
  if (facts.has_glob && facts.has_read) return true;
  if (facts.has_grep && facts.has_read) return true;
  // Both ways to change files
  if (facts.has_edit && facts.has_write) return true;
  // Reading before changing files
  if (facts.has_read && (facts.has_edit || facts.has_write)) return true;
  return false;
}

/**
 * Write common rules for the tools available in this request.
 *
 * The rules never recommend a tool that the agent cannot use.
 *
 * @param facts - Available tools for this request.
 * @returns Rules separated by newlines, or an empty string if none apply.
 */
export function buildCommonRules(facts: ToolFacts): string {
  const rules: string[] = [];

  // Name only the file tools the agent can use.
  const fileTools = [
    facts.has_glob ? "glob" : null,
    facts.has_grep ? "grep" : null,
    facts.has_read ? "read" : null,
    facts.has_edit ? "edit" : null,
    facts.has_write ? "write" : null,
  ].filter(Boolean);
  if (facts.has_shell && fileTools.length > 0) {
    rules.push(`Prefer \`${fileTools.join("`, `")}\` over \`shell\` for ordinary file work.`);
  }

  // Say which tool to use for each kind of file lookup.
  if (searchToolsPresent(facts)) {
    const parts: string[] = [];
    if (facts.has_glob) parts.push("`glob` for file-name search");
    if (facts.has_grep) parts.push("`grep` for content search");
    if (facts.has_read) parts.push("`read` for file content");
    if (parts.length === 2) {
      rules.push("Use " + parts[0] + " and " + parts[1] + ".");
    } else if (parts.length >= 3) {
      rules.push(
        "Use " + parts.slice(0, -1).join(", ") + ", and " + parts[parts.length - 1]! + ".",
      );
    }
  }

  // Choose between editing and writing.
  if (facts.has_edit && facts.has_write) {
    rules.push("Prefer `edit` for targeted changes and `write` for new files or full rewrites.");
  }

  // Read adds line numbers to its output; they are not part of the file.
  if (facts.has_read && facts.has_edit && facts.has_write) {
    rules.push(
      "Read before `edit` or overwriting with `write`; for `edit`, copy exact text and omit any `{n}: ` prefixes.",
    );
  } else if (facts.has_read && facts.has_edit) {
    rules.push("Read before `edit`, then copy exact text and omit any `{n}: ` prefixes.");
  } else if (facts.has_read && facts.has_write && !facts.has_edit) {
    rules.push("Read before `write` if the file already exists.");
  }

  return rules.join("\n");
}

function searchToolsPresent(facts: ToolFacts): boolean {
  const count = [facts.has_glob, facts.has_grep, facts.has_read].filter(Boolean).length;
  return count >= 2;
}
