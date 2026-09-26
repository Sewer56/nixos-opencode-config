import assert from "node:assert/strict"
import { describe, test } from "node:test"
import { mkdtemp, rm, writeFile } from "node:fs/promises"
import { tmpdir } from "node:os"
import path from "node:path"

import { buildIntoEvent, BASE_PROMPT_MARKER, type SessionEvent } from "../src/builder.ts"
import { testCases } from "./cases.ts"

const baseInput = { workingDirectory: "/w", platform: "linux", cwd: "/w" }

function fakeEvent(): SessionEvent {
  return {
    system: [
      { type: "text", text: `${BASE_PROMPT_MARKER}, a coding agent harness.\n\n# Harness` },
      { type: "text", text: "agent prompt part" },
    ],
    tools: {
      read: { description: "read" },
      edit: { description: "edit" },
      write: { description: "write" },
      glob: { description: "glob" },
      grep: { description: "grep" },
      shell: { description: "shell" },
      task: { description: "task" },
    },
    agent: "build",
  }
}

describe("buildIntoEvent", () => {
  test("base_part_should_be_replaced_when_marker_present", async () => {
    // Arrange
    const event = fakeEvent()

    // Act
    const result = await buildIntoEvent(event, baseInput)

    // Assert
    assert.equal(result.strategy, "replaced")
    assert.equal(event.system!.some((p) => p.text?.includes(BASE_PROMPT_MARKER)), false)
    assert.equal(event.system!.at(-1)!.text, "agent prompt part")
    assert.ok(event.system![0]!.text!.includes("# Environment"))
    assert.ok(event.system!.map((p) => p.text).join("\n").includes("# Tool Usage Guidelines"))
  })

  test("sections_should_skip_when_base_prompt_absent", async () => {
    // Arrange
    const event = fakeEvent()
    event.system = [{ type: "text", text: "custom agent prompt" }]

    // Act
    const result = await buildIntoEvent(event, baseInput)

    // Assert
    assert.equal(result.strategy, "skipped")
    assert.match(result.warning!, /expected one OpenCode base prompt, found 0/)
    assert.deepEqual(result.sections, [])
    assert.equal(event.system!.at(-1)!.text, "custom agent prompt")
    assert.equal(event.system!.length, 1)
  })

  testCases("sections_should_select_available_guidance", [
    {
      name: "apply_patch_present", tools: { apply_patch: {}, read: {}, glob: {} },
      absent: ["## `Edit` Tool", "## `Write` Tool"], expected: "## `ApplyPatch` Tool",
    },
    {
      name: "read_and_edit_present", tools: { read: {}, edit: {} },
      absent: [], expected: "Read before `edit`",
    },
    {
      name: "task_and_search_tools_present", tools: { task: {}, glob: {}, grep: {} },
      absent: ["## `Read` Tool"], expected: "`glob`, `grep`",
    },
    {
      name: "task_alias_present", tools: { task: {} },
      absent: [], expected: "## `Subagent` Tool",
    },
  ], async ({ tools, absent, expected }) => {
    // Arrange
    const event = fakeEvent()
    event.tools = tools

    // Act
    const result = await buildIntoEvent(event, baseInput)

    // Assert
    const joined = result.sections.join("\n")
    for (const fragment of absent) assert.ok(!joined.includes(fragment))
    assert.ok(joined.includes(expected))
  })

  test("supplemental_should_render_when_file_option_set", async () => {
    // Arrange
    const dir = await mkdtemp(path.join(tmpdir(), "pb-"))
    try {
      await writeFile(path.join(dir, "extra.md"), 'supplemental body\n{{env:TEST_PB_TOKEN}}\n')
      const event = fakeEvent()

      // Act
      const result = await buildIntoEvent(event, { ...baseInput, cwd: dir, supplementalFiles: ["extra.md"] })

      // Assert
      const joined = result.sections.join("\n")
      assert.ok(joined.includes("# Supplemental Context"))
      assert.ok(joined.includes('## extra\nsupplemental body\n{{env:TEST_PB_TOKEN}}'))
    } finally {
      await rm(dir, { recursive: true, force: true })
    }
  })

  test("supplemental_should_report_unreadable_file_when_missing", async () => {
    // Arrange
    const dir = await mkdtemp(path.join(tmpdir(), "pb-"))
    try {
      const event = fakeEvent()

      // Act
      const result = await buildIntoEvent(event, { ...baseInput, cwd: dir, supplementalFiles: ["missing.md"] })

      // Assert
      assert.ok(result.sections.join("\n").includes("## missing\nmissing.md: unreadable ("))
    } finally {
      await rm(dir, { recursive: true, force: true })
    }
  })
})

describe("buildIntoEvent strip mode", () => {
  test("buildIntoEvent_should_strip_core_parts_and_keep_others_when_stripCore", async () => {
    // Arrange
    const event = {
      system: [
        { type: "text", text: `${BASE_PROMPT_MARKER} the base prompt` },
        { type: "text", text: "# Your Model\n- Name: GLM" },
        { type: "text", text: "Here is some useful information about the environment you are running in:\n<env>\n  x\n</env>\n\nToday's date: Fri Sep 25 2026" },
        { type: "text", text: "When you create a worktree outside the current working directory, do things." },
        { type: "text", text: "AGENTS.md project instructions stay." },
        { type: "text", text: "CAVEMAN MODE ACTIVE custom plugin part stays." },
      ],
      tools: { write: {}, read: {} },
    }

    // Act
    const result = await buildIntoEvent(event, { ...baseInput, stripCore: true })

    // Assert
    assert.equal(result.strategy, "stripped")
    const texts = event.system!.map((part) => part.text)
    assert.equal(texts.some((text) => text!.includes("AI agent running in OpenCode")), false)
    assert.equal(texts.some((text) => text!.includes("# Your Model")), false)
    assert.equal(texts.some((text) => text!.includes("<env>")), false)
    assert.equal(texts.some((text) => text!.includes("worktree outside")), false)
    assert.equal(texts.some((text) => text!.includes("AGENTS.md project instructions")), true)
    assert.equal(texts.some((text) => text!.includes("CAVEMAN MODE ACTIVE")), true)
    assert.equal(texts[0]!.startsWith("# Environment"), true)
  })

  test("buildIntoEvent_should_keep_instructions_attached_to_env_part_when_stripCore", async () => {
    // Arrange: OpenCode packs AGENTS.md content into the environment part.
    const event = {
      system: [
        { type: "text", text: `${BASE_PROMPT_MARKER} the base prompt` },
        {
          type: "text",
          text: [
            "Here is some useful information about the environment you are running in:",
            "<env>",
            "  Working directory: /w",
            "</env>",
            "Today's date: Fri Sep 25 2026",
            "Repo map for this project.",
          ].join("\n"),
        },
      ],
      tools: { write: {} },
    }

    // Act
    await buildIntoEvent(event, { ...baseInput, stripCore: true })

    // Assert
    const texts = event.system!.map((part) => part.text) as string[]
    assert.equal(texts[texts.length - 1], "Repo map for this project.")
    assert.equal(texts.some((text) => text.includes("<env>")), false)
    assert.equal(texts.some((text) => text.includes("Today's date")), false)
  })

  test("buildIntoEvent_should_not_strip_when_stripCore_unset", async () => {
    // Arrange
    const event = {
      system: [
        { type: "text", text: `${BASE_PROMPT_MARKER} the base prompt` },
        { type: "text", text: "# Your Model\n- Name: GLM" },
      ],
      tools: { write: {} },
    }

    // Act
    const result = await buildIntoEvent(event, { ...baseInput })

    // Assert
    assert.equal(result.strategy, "replaced")
    assert.equal(event.system!.some((part) => part.text!.includes("# Your Model")), true)
  })

  testCases("buildIntoEvent_should_skip_unsafe_layout_without_mutating", [
    { name: "missing_system", system: undefined, reason: /system prompt is missing/ },
    { name: "changed_base", system: [{ type: "text", text: "Changed base prompt" }], reason: /base prompt, found 0/ },
    { name: "duplicate_base", system: [
      { type: "text", text: BASE_PROMPT_MARKER }, { type: "text", text: BASE_PROMPT_MARKER },
    ], reason: /base prompt, found 2/ },
    { name: "missing_env", system: [{ type: "text", text: BASE_PROMPT_MARKER }], reason: /environment part/ },
    { name: "changed_env", system: [
      { type: "text", text: BASE_PROMPT_MARKER }, { type: "text", text: "Environment layout changed" },
    ], reason: /environment part/ },
    { name: "malformed_env", system: [
      { type: "text", text: BASE_PROMPT_MARKER },
      { type: "text", text: "Here is some useful information about the environment you are running in:\n<env>\nx\n</env>\nProject instructions" },
    ], reason: /environment part/ },
  ], async ({ system, reason }) => {
    // Arrange
    const event: SessionEvent = { system, tools: { read: { description: "original" } } }
    const before = structuredClone(event)

    // Act
    const result = await buildIntoEvent(event, { ...baseInput, stripCore: true })

    // Assert
    assert.equal(result.strategy, "skipped")
    assert.match(result.warning!, reason)
    assert.deepEqual(result.sections, [])
    assert.deepEqual(event, before)
  })
})
