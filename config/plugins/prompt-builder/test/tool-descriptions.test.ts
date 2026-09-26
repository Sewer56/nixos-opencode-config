import assert from "node:assert/strict"
import { describe } from "node:test"
import { shortenToolDescriptions } from "../src/tool-descriptions.ts"
import originals from "./fixtures/v2.0.16-descriptions.json" with { type: "json" }
import { testCases } from "./cases.ts"

// Contract text copied from OpenCode v2.0.16, not imported from the rewrite table.
const cases = [
  {
    name: "read",
    guidance: ["images or PDFs", "1-based", "not part of the file", "offset and limit", "larger reads"],
  },
  { name: "write", guidance: ["overwrite", "parent directories", "edit for partial"] },
  {
    name: "edit",
    guidance: ["indentation", "omit Read line-number", "must exist", "match once", "replaceAll is true"],
  },
  { name: "glob", guidance: ["file paths", "glob", "**/*.ts"] },
  {
    name: "grep",
    guidance: ["ripgrep regular expressions", "literal text", "path or include", "line numbers", "previews"],
  },
  { name: "question", guidance: ["free-form", "automatically", "multiple to true", "first", "(Recommended)"] },
  { name: "shell", guidance: ["Quote paths", "full output to a file", "timeout", "Background", "notify"] },
] satisfies { name: keyof typeof originals; guidance: string[] }[]

describe("tool descriptions", () => {
  // Core behavior
  testCases("description_should_shorten_and_keep_guidance", cases, ({ name, guidance }) => {
    // Arrange: a schema may carry new parameters even when text is unchanged.
    const input = Object.freeze({ type: "object", properties: { futureOption: { type: "string" } } })
    const tool = { description: originals[name], input, extra: "keep" }
    const tools = { [name]: tool }

    // Act
    shortenToolDescriptions(tools)
    const afterFirst = tool.description
    shortenToolDescriptions(tools)

    // Assert: preserve identity, schema, other fields and repeat-call behavior.
    assert.ok(tool.description.length < originals[name].length)
    for (const text of guidance) assert.ok(tool.description.includes(text), text)
    assert.equal(tool.description, afterFirst)
    assert.equal(tools[name], tool)
    assert.equal(tool.input, input)
    assert.equal(tool.extra, "keep")
    assert.deepEqual(Object.keys(tools), [name])
  })

  testCases("shell_should_keep_environment", [
    { name: "linux", environment: "Commands run on Linux using bash." },
    { name: "windows", environment: "Commands run on Windows using powershell." },
    { name: "macos", environment: "Commands run on macOS using /bin/zsh." },
  ], ({ environment }) => {
    // Arrange
    const description = originals.shell.replace("output. ", `output. ${environment} `)
    const input = { properties: { workdir: { description: "Command working directory" } } }
    const tools = { shell: { description, input } }

    // Act
    shortenToolDescriptions(tools)

    // Assert
    assert.ok(tools.shell.description.includes(environment))
    assert.ok(tools.shell.description.length < description.length)
    assert.equal(tools.shell.input, input)
  })

  // Edge cases: changed contracts and unrecognized tool shapes must survive.
  testCases("description_should_preserve_extensions", cases, ({ name }) => {
    // Arrange
    const description = `${originals[name]} New worktree option: retain this guidance.`
    const tools = { [name]: { description } }

    // Act
    shortenToolDescriptions(tools)

    // Assert
    assert.equal(tools[name]!.description, description)
  })

  testCases("description_should_remain_unchanged", [
    { name: "unknown_shell", tool: "shell", description: "Execute commands with a new worktree parameter." },
    {
      name: "extended_shell", tool: "shell",
      description: originals.shell.replace(
        "output. ", "output. Commands run on Linux using bash. Extra guidance. ",
      ),
    },
    {
      name: "subagent", tool: "subagent",
      description: "Resume via sessionID. Available subagents:\n- explorer: inspect code",
    },
    {
      name: "session_move", tool: "opencode_session_move",
      description: "Move the session into a worktree.",
    },
    { name: "custom_read", tool: "custom_read", description: originals.read },
    { name: "unknown_read", tool: "read", description: "Read files with custom plugin semantics." },
  ], ({ tool, description }) => {
    // Arrange
    const tools = { [tool]: { description } }

    // Act
    shortenToolDescriptions(tools)

    // Assert
    assert.equal(tools[tool]!.description, description)
  })

  testCases("snapshot_should_remain_unchanged", [
    { name: "absent", tools: undefined },
    { name: "empty", tools: {} },
    { name: "missing_description", tools: { read: {} } },
    { name: "null_tool", tools: { shell: null } },
    { name: "non_string_description", tools: { edit: { description: 1 } } },
  ], ({ tools }) => {
    // Arrange
    const before = structuredClone(tools)

    // Act
    shortenToolDescriptions(tools)

    // Assert
    assert.deepEqual(tools, before)
  })
})
