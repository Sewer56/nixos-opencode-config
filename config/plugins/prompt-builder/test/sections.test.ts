import { expect, test } from "bun:test"
import { factsFromToolKeys } from "../src/facts"
import { buildSections } from "../src/sections"

test.each([
  ["shell", "non-zero exit codes are reported", "Non-zero exit codes not shown"],
  ["read", "Images and PDFs are supported", "Binary files cannot be read"],
  ["edit", "`oldString`", "`old_string`"],
  ["glob", "set `limit`", "sorted newest first"],
  ["subagent", "`sessionID` to resume", "stateless"],
  ["question", "one or more questions", "one clarifying question"],
])("guidance_should_describe_v2_behavior_when_%s", (tool, expected, outdated) => {
  // Arrange
  const input = {
    facts: factsFromToolKeys([tool]),
    workingDirectory: "/project",
    platform: "linux",
  }

  // Act
  const text = buildSections(input).join("\n")

  // Assert
  expect(text).toContain(expected)
  expect(text).not.toContain(outdated)
})
