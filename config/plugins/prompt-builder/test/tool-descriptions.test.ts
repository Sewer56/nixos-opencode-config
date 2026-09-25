import { describe, expect, test } from "bun:test"
import { shortenToolDescriptions } from "../src/tool-descriptions"
import originals from "./fixtures/v2.0.16-descriptions.json"

// Contract text copied from OpenCode v2.0.16, not imported from the rewrite table.
const cases = [
  { name: "read", guidance: ["images or PDFs", "1-based", "not file content", "offset/limit", "larger reads"] },
  { name: "write", guidance: ["overwrite", "parent directories", "edit for partial"] },
  { name: "edit", guidance: ["indentation", "omit Read line-number", "must exist", "match once", "replaceAll=true"] },
  { name: "glob", guidance: ["file paths", "glob", "**/*.ts"] },
  { name: "grep", guidance: ["ripgrep regex", "literal text", "path/include", "line numbers", "previews"] },
  { name: "question", guidance: ["free-form", "automatically", "multiple=true", "first", "(Recommended)"] },
  { name: "shell", guidance: ["Quote paths", "full-output file", "timeout", "background", "notify"] },
] satisfies { name: keyof typeof originals; guidance: string[] }[]

describe("tool descriptions", () => {
  // Core behavior
  test.each(cases)("description_should_shorten_and_keep_guidance_when_$name", ({ name, guidance }) => {
    // Arrange: a schema may carry new parameters even when text is unchanged.
    const input = Object.freeze({ type: "object", properties: { futureOption: { type: "string" } } })
    const tool = { description: originals[name], input, extra: "keep" }
    const tools = { [name]: tool }

    // Act
    shortenToolDescriptions(tools)
    const afterFirst = tool.description
    shortenToolDescriptions(tools)

    // Assert: preserve identity, schema, other fields and repeat-call behavior.
    expect(tool.description.length).toBeLessThan(originals[name].length)
    for (const text of guidance) expect(tool.description).toContain(text)
    expect(tool.description).toBe(afterFirst)
    expect(tools[name]).toBe(tool)
    expect(tool.input).toBe(input)
    expect(tool.extra).toBe("keep")
    expect(Object.keys(tools)).toEqual([name])
  })

  test.each([
    ["Commands run on Linux using bash.", "Linux"],
    ["Commands run on Windows using powershell.", "Windows"],
    ["Commands run on macOS using /bin/zsh.", "macOS"],
  ])("shell_should_keep_environment_when_%s", (environment, platform) => {
    // Arrange
    const description = originals.shell.replace("output. ", `output. ${environment} `)
    const input = { properties: { workdir: { description: "Command working directory" } } }
    const tools = { shell: { description, input } }

    // Act
    shortenToolDescriptions(tools)

    // Assert
    expect(tools.shell.description).toContain(environment)
    expect(tools.shell.description).toContain(platform)
    expect(tools.shell.description.length).toBeLessThan(description.length)
    expect(tools.shell.input).toBe(input)
  })

  // Edge cases: changed contracts and unrecognized tool shapes must survive.
  test.each(cases)("description_should_preserve_extensions_when_$name", ({ name }) => {
    // Arrange
    const description = `${originals[name]} New worktree option: retain this guidance.`
    const tools = { [name]: { description } }

    // Act
    shortenToolDescriptions(tools)

    // Assert
    expect(tools[name]!.description).toBe(description)
  })

  test.each([
    ["shell", "Execute commands with a new worktree parameter."],
    ["shell", originals.shell.replace("output. ", "output. Commands run on Linux using bash. Extra guidance. ")],
    ["subagent", "Resume via sessionID. Available subagents:\n- explorer: inspect code"],
    ["opencode_session_move", "Move the session into a worktree."],
    ["custom_read", originals.read],
    ["read", "Read files with custom plugin semantics."],
  ])("description_should_remain_unchanged_when_%s_is_unrecognized", (name, description) => {
    // Arrange
    const tools = { [name]: { description } }

    // Act
    shortenToolDescriptions(tools)

    // Assert
    expect(tools[name]!.description).toBe(description)
  })

  test.each([
    ["absent", undefined],
    ["empty", {}],
    ["missing_description", { read: {} }],
    ["null_tool", { shell: null }],
    ["non_string_description", { edit: { description: 1 } }],
  ])("snapshot_should_remain_unchanged_when_%s", (_name, tools) => {
    // Arrange
    const before = structuredClone(tools)

    // Act
    shortenToolDescriptions(tools)

    // Assert
    expect(tools).toEqual(before)
  })
})
