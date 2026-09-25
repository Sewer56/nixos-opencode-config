import { describe, expect, test } from "bun:test"
import { mkdtemp, readFile, rm, stat } from "node:fs/promises"
import { tmpdir } from "node:os"
import path from "node:path"
import plugin from "../src/v2"
import { BUILDER_TAG, type SessionEvent } from "../src/builder"
import { capturePromptDump, writePromptDump } from "../src/prompt-dump"
import originals from "./fixtures/v2.0.16-descriptions.json"

async function handlers() {
  const hooks = new Map<string, (event: SessionEvent) => Promise<void>>()
  await plugin.setup({
    location: { directory: "/project" },
    session: { hook: async (name, handler) => { hooks.set(name, handler) } },
  })
  return hooks
}

describe("V2 hooks", () => {
  // Construction and core behavior
  test("setup_should_register_all_request_hooks", async () => {
    // Arrange / Act
    const hooks = await handlers()

    // Assert
    expect([...hooks.keys()]).toEqual(["context", "compaction", "generate"])
  })

  test.each(["context", "compaction", "generate"])(
    "hook_should_shorten_descriptions_and_keep_contracts_when_%s",
    async (name) => {
      // Arrange: an earlier builder invocation must not prevent tool rewriting.
      const hooks = await handlers()
      const input = { properties: { workdir: { type: "string" }, background: { type: "boolean" } } }
      const tools = {
        shell: { description: originals.shell, input },
        subagent: { description: "Available subagents:\n- explorer: inspect code", input: {} },
      }
      const system = [{ type: "text", text: `${BUILDER_TAG}\n# Environment\nAlready built` }]
      const event: SessionEvent = { system, tools }

      // Act
      await hooks.get(name)!(event)
      const afterFirst = structuredClone(event)
      await hooks.get(name)!(event)

      // Assert
      expect(tools.shell.description.length).toBeLessThan(originals.shell.length)
      expect(tools.shell.input).toBe(input)
      expect(event.tools).toBe(tools)
      expect(event.system).toBe(system)
      expect(tools.subagent.description).toBe("Available subagents:\n- explorer: inspect code")
      expect(event).toEqual(afterFirst)
    },
  )
})

describe("contract dumps", () => {
  test("hook_should_keep_original_dump_when_loaded_twice", async () => {
    // Arrange
    const directory = await mkdtemp(path.join(tmpdir(), "pb-hook-dump-"))
    const previous = process.env.PB_DUMP
    const prefix = path.join(directory, "probe")
    const event: SessionEvent = {
      system: [],
      tools: { read: { description: originals.read, input: { type: "object" } } },
    }

    try {
      process.env.PB_DUMP = prefix
      const first = await handlers()
      const second = await handlers()

      // Act
      await first.get("context")!(event)
      await second.get("context")!(event)

      // Assert
      const raw = JSON.parse(await readFile(`${prefix}.context.tools.raw.json`, "utf8"))
      const final = JSON.parse(await readFile(`${prefix}.context.tools.final.json`, "utf8"))
      expect(raw.read.description).toBe(originals.read)
      expect(final.read.description.length).toBeLessThan(originals.read.length)
    } finally {
      if (previous === undefined) delete process.env.PB_DUMP
      else process.env.PB_DUMP = previous
      await rm(directory, { recursive: true, force: true })
    }
  })

  test("dump_should_capture_complete_before_and_after_contracts", async () => {
    // Arrange
    const directory = await mkdtemp(path.join(tmpdir(), "pb-contracts-"))
    const prefix = path.join(directory, "probe")
    const event: SessionEvent = {
      system: [{ type: "text", text: "original system" }],
      tools: { read: { description: originals.read, input: { type: "object" } } },
    }
    const before = capturePromptDump(event)

    try {
      // Act
      const hooks = await handlers()
      await hooks.get("context")!(event)
      await writePromptDump(prefix, "context", before, event)

      // Assert
      const raw = JSON.parse(await readFile(`${prefix}.context.tools.raw.json`, "utf8"))
      const final = JSON.parse(await readFile(`${prefix}.context.tools.final.json`, "utf8"))
      expect(raw.read.description).toBe(originals.read)
      expect(final.read.description.length).toBeLessThan(raw.read.description.length)
      expect(final.read.input).toEqual(raw.read.input)
      expect(await readFile(`${prefix}.context.raw.txt`, "utf8")).toBe("original system")
      expect(await readFile(`${prefix}.context.final.txt`, "utf8")).toContain(BUILDER_TAG)
      expect(await readFile(`${prefix}.context.tools.txt`, "utf8")).toContain("read\t")
      expect((await stat(`${prefix}.context.tools.raw.json`)).mode & 0o777).toBe(0o600)
    } finally {
      await rm(directory, { recursive: true, force: true })
    }
  })
})
