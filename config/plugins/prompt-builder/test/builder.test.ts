import { describe, expect, test } from "bun:test"
import { mkdtemp, rm, writeFile } from "node:fs/promises"
import { tmpdir } from "node:os"
import path from "node:path"

import { buildIntoEvent, BASE_PROMPT_MARKER } from "../src/builder"
import type { SessionEvent } from "../src/builder"

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
    const event = fakeEvent()

    const result = await buildIntoEvent(event, baseInput)

    expect(result.strategy).toBe("replaced")
    expect(event.system!.some((p) => p.text?.includes(BASE_PROMPT_MARKER))).toBe(false)
    expect(event.system!.at(-1)!.text).toBe("agent prompt part")
    expect(event.system![0]!.text).toContain("# Environment")
    expect(event.system!.map((p) => p.text).join("\n")).toContain("# Tool Usage Guidelines")
  })

  test("sections_should_prepend_when_base_prompt_absent", async () => {
    const event = fakeEvent()
    event.system = [{ type: "text", text: "custom agent prompt" }]

    const result = await buildIntoEvent(event, baseInput)

    expect(result.strategy).toBe("prepended")
    expect(event.system!.at(-1)!.text).toBe("custom agent prompt")
    expect(event.system![0]!.text).toContain("# Environment")
  })

  test.each([
    ["edit_and_write_suppressed_when_apply_patch_present", { apply_patch: {}, read: {}, glob: {} }, ["## `Edit` Tool", "## `Write` Tool"], "## `ApplyPatch` Tool"],
    ["read_before_edit_wording_when_both_present", { read: {}, edit: {} }, [], "Read before `edit`"],
    ["task_section_lists_present_search_tools", { task: {}, glob: {}, grep: {} }, ["## `Read` Tool"], "`glob`, `grep`"],
    ["v1_task_alias_maps_to_subagent_section", { task: {} }, [], "## `Subagent` Tool"],
  ])("sections_should_%s", async (_name, tools, absent, expected) => {
    const event = fakeEvent()
    event.tools = tools

    const result = await buildIntoEvent(event, baseInput)

    const joined = result.sections.join("\n")
    for (const fragment of absent) expect(joined).not.toContain(fragment)
    expect(joined).toContain(expected)
  })

  test("supplemental_should_render_when_file_option_set", async () => {
    const dir = await mkdtemp(path.join(tmpdir(), "pb-"))
    try {
      await writeFile(path.join(dir, "extra.md"), "supplemental body")
      const event = fakeEvent()

      const result = await buildIntoEvent(event, { ...baseInput, cwd: dir, supplementalFiles: ["extra.md"] })

      const joined = result.sections.join("\n")
      expect(joined).toContain("# Supplemental Context")
      expect(joined).toContain("## extra\nsupplemental body")
    } finally {
      await rm(dir, { recursive: true, force: true })
    }
  })
})

describe("buildIntoEvent idempotency", () => {
  test("buildIntoEvent_should_not_duplicate_when_applied_twice", async () => {
    // Arrange
    const event = {
      system: [{ type: "text", text: `${BASE_PROMPT_MARKER} rest of base` }],
      tools: { write: {}, read: {} },
    }
    const input = { ...baseInput }

    // Act
    await buildIntoEvent(event, input)
    const afterFirst = JSON.stringify(event.system)
    const result = await buildIntoEvent(event, input)

    // Assert
    expect(result.strategy).toBe("already-applied")
    expect(result.sections).toEqual([])
    expect(JSON.stringify(event.system)).toBe(afterFirst)
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
    expect(result.strategy).toBe("stripped")
    const texts = event.system!.map((part) => part.text)
    expect(texts.some((text) => text!.includes("AI agent running in OpenCode"))).toBe(false)
    expect(texts.some((text) => text!.includes("# Your Model"))).toBe(false)
    expect(texts.some((text) => text!.includes("<env>"))).toBe(false)
    expect(texts.some((text) => text!.includes("worktree outside"))).toBe(false)
    expect(texts.some((text) => text!.includes("AGENTS.md project instructions"))).toBe(true)
    expect(texts.some((text) => text!.includes("CAVEMAN MODE ACTIVE"))).toBe(true)
    expect(texts[0]!.startsWith("<!-- prompt-builder -->")).toBe(true)
  })

  test("buildIntoEvent_should_keep_instructions_attached_to_env_part_when_stripCore", async () => {
    // Arrange: OpenCode packs AGENTS.md content into the environment part.
    const event = {
      system: [
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
    expect(texts[texts.length - 1]).toBe("Repo map for this project.")
    expect(texts.some((text) => text.includes("<env>"))).toBe(false)
    expect(texts.some((text) => text.includes("Today's date"))).toBe(false)
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
    expect(result.strategy).toBe("replaced")
    expect(event.system!.some((part) => part.text!.includes("# Your Model"))).toBe(true)
  })
})
