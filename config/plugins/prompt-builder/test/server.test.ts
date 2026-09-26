import assert from "node:assert/strict"
import { describe, test } from "node:test"
import { mkdtemp, readFile, rm, stat } from "node:fs/promises"
import { tmpdir } from "node:os"
import path from "node:path"
import plugin, { dumpPrefix } from "../server.ts"
import { BUILDER_TAG, type SessionEvent } from "../src/builder.ts"
import { capturePromptDump, writePromptDump } from "../src/prompt-dump.ts"
import originals from "./fixtures/v2.0.16-descriptions.json" with { type: "json" }
import { testCases } from "./cases.ts"

async function handlers() {
  const hooks = new Map<string, (event: SessionEvent) => Promise<void>>()
  await plugin.setup({
    location: { directory: "/project" },
    session: { hook: async (name, handler) => { hooks.set(name, handler) } },
  })
  return hooks
}

describe("request hooks", () => {
  // Construction and core behavior
  test("setup_should_register_all_request_hooks", async () => {
    // Arrange / Act
    const hooks = await handlers()

    // Assert
    assert.deepEqual([...hooks.keys()], ["context", "compaction", "generate"])
  })

  testCases(
    "hook_should_shorten_descriptions_and_keep_contracts",
    [{ name: "context" }, { name: "compaction" }, { name: "generate" }],
    async ({ name }) => {
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
      assert.ok(tools.shell.description.length < originals.shell.length)
      assert.equal(tools.shell.input, input)
      assert.equal(event.tools, tools)
      assert.equal(event.system, system)
      assert.equal(tools.subagent.description, "Available subagents:\n- explorer: inspect code")
      assert.deepEqual(event, afterFirst)
    },
  )
})

describe("contract dumps", () => {
  test("hook_should_use_plugin_directory_when_dump_is_enabled", async () => {
    assert.equal(dumpPrefix("1"), path.join(import.meta.dirname, "..", "probe"))
    assert.equal(dumpPrefix("/tmp/custom"), "/tmp/custom")
    assert.equal(dumpPrefix(undefined), undefined)
  })

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
      assert.equal(raw.read.description, originals.read)
      assert.ok(final.read.description.length < originals.read.length)
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
      assert.equal(raw.read.description, originals.read)
      assert.ok(final.read.description.length < raw.read.description.length)
      assert.deepEqual(final.read.input, raw.read.input)
      assert.equal(await readFile(`${prefix}.context.raw.txt`, "utf8"), "original system")
      assert.ok((await readFile(`${prefix}.context.final.txt`, "utf8")).includes(BUILDER_TAG))
      assert.ok((await readFile(`${prefix}.context.tools.txt`, "utf8")).includes("read\t"))
      assert.equal((await stat(`${prefix}.context.tools.raw.json`)).mode & 0o777, 0o600)
    } finally {
      await rm(directory, { recursive: true, force: true })
    }
  })
})
