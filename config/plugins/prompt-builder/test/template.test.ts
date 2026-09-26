import assert from "node:assert/strict"
import { mkdtemp, rm, writeFile } from "node:fs/promises"
import { tmpdir } from "node:os"
import { join } from "node:path"
import { expandTemplate } from "../src/template.ts"
import { testCases } from "./cases.ts"

testCases("file_should_expand", [
  { name: "readable", present: true, expected: "Before hello world after" },
  { name: "missing", present: false, expected: '{{ file="note.md" }}: unreadable (' },
], async ({ present, expected }) => {
  // Arrange
  const cwd = await mkdtemp(join(tmpdir(), "pb-template-"))

  try {
    if (present) await writeFile(join(cwd, "note.md"), "hello {{arg:name}}")

    // Act
    const text = await expandTemplate('Before {{ file="note.md" }} after', {
      cwd,
      args: { name: "world" },
    })

    // Assert
    assert.ok(text.includes(expected))
  } finally {
    await rm(cwd, { recursive: true, force: true })
  }
})
