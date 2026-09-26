import assert from "node:assert/strict"
import { factsFromToolKeys } from "../src/facts.ts"
import { buildSections } from "../src/sections.ts"
import { testCases } from "./cases.ts"

testCases("guidance_should_describe_tool_behavior", [
  { name: "shell", expected: "non-zero exit codes are reported", outdated: "Non-zero exit codes not shown" },
  { name: "read", expected: "Images and PDFs are supported", outdated: "Binary files cannot be read" },
  { name: "edit", expected: "`oldString`", outdated: "`old_string`" },
  { name: "glob", expected: "set `limit`", outdated: "sorted newest first" },
  { name: "subagent", expected: "`sessionID` to resume", outdated: "stateless" },
  { name: "question", expected: "one or more questions", outdated: "one clarifying question" },
], ({ name, expected, outdated }) => {
  // Arrange
  const input = {
    facts: factsFromToolKeys([name]),
    workingDirectory: "/project",
    platform: "linux",
  }

  // Act
  const text = buildSections(input).join("\n")

  // Assert
  assert.ok(text.includes(expected))
  assert.ok(!text.includes(outdated))
})
