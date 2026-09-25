/**
 * Snapshot of which tools a request advertised, used to decide which
 * prompt-builder sections apply.
 *
 * Tool keys arrive from the OpenCode session event's `tools` object, keyed by
 * tool name. Both V1 and V2 names are accepted (bash/shell, task/subagent,
 * apply_patch/patch) so the same facts drive both generators.
 *
 * @module prompt-builder/facts
 */

/** Boolean facts extracted from a per-request tool snapshot. */
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

/** Maps tool names (V1 and V2 spellings) to the fact they set. */
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
 * Derive facts from the keys of a session event's `tools` object.
 *
 * `patch`/`apply_patch` wins over `edit`/`write`: when a patch-style tool is
 * present, the targeted-edit sections are suppressed to avoid teaching the
 * model two conflicting edit mechanisms.
 *
 * @param keys - Tool names present in the request.
 * @returns Facts with every known tool marked present or absent.
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

/** True when any cross-tool rule applies to this combination of tools. */
export function hasCommonRules(facts: ToolFacts): boolean {
  // Bash + at least one file tool
  if (
    facts.has_shell &&
    (facts.has_read || facts.has_edit || facts.has_write || facts.has_glob || facts.has_grep)
  ) {
    return true;
  }
  // Search tools separation
  if (facts.has_glob && facts.has_grep) return true;
  if (facts.has_glob && facts.has_read) return true;
  if (facts.has_grep && facts.has_read) return true;
  // Edit vs write
  if (facts.has_edit && facts.has_write) return true;
  // Read before edit/write
  if (facts.has_read && (facts.has_edit || facts.has_write)) return true;
  return false;
}

/**
 * Build the shared "## Common Rules" body for the present tool combination.
 *
 * Each rule only names tools that are actually present, so a restricted agent
 * never gets told to use a tool it cannot call.
 *
 * @param facts - Tool facts for this request.
 * @returns Newline-joined rules, empty string when none apply.
 */
export function buildCommonRules(facts: ToolFacts): string {
  const rules: string[] = [];

  // Shell vs file tools — only list tools that are actually present
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

  // Search tools separation — build a proper conjunction without double "and"
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

  // Edit vs write
  if (facts.has_edit && facts.has_write) {
    rules.push("Prefer `edit` for targeted changes and `write` for new files or full rewrites.");
  }

  // Read before edit/write — the `{n}: ` prefix comes from the read tool, not the file
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
