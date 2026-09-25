import { describe, expect, test } from "bun:test"
import { mkdir, mkdtemp, symlink, writeFile } from "node:fs/promises"
import { tmpdir } from "node:os"
import path from "node:path"
import { canonicalPath, GuardDeny } from "../src/canonical"
import { decideExternal, globToRegExp, stripJsonc } from "../src/rules"
import { extractTargets } from "../src/guard"

async function fixture(): Promise<string> {
  return mkdtemp(path.join(tmpdir(), "path-guard-"))
}

describe("canonicalPath", () => {
  test("canonicalPath_should_return_real_path_when_file_exists", async () => {
    // Arrange
    const dir = await fixture()
    const file = path.join(dir, "real.txt")
    await writeFile(file, "x")

    // Act
    const resolved = await canonicalPath(file)

    // Assert
    expect(resolved).toBe(file)
  })

  test("canonicalPath_should_append_missing_leaf_when_file_does_not_exist", async () => {
    // Arrange
    const dir = await fixture()

    // Act
    const resolved = await canonicalPath(path.join(dir, "new", "file.txt"))

    // Assert
    expect(resolved).toBe(path.join(dir, "new", "file.txt"))
  })

  test("canonicalPath_should_resolve_target_when_symlink_exists", async () => {
    // Arrange
    const dir = await fixture()
    const file = path.join(dir, "real.txt")
    const link = path.join(dir, "link.txt")
    await writeFile(file, "x")
    await symlink(file, link)

    // Act
    const resolved = await canonicalPath(link)

    // Assert
    expect(resolved).toBe(file)
  })

  test("canonicalPath_should_fail_closed_when_symlink_dangles", async () => {
    // Arrange
    const dir = await fixture()
    const link = path.join(dir, "dangling.txt")
    await symlink(path.join(dir, "missing.txt"), link)

    // Act + Assert
    await expect(canonicalPath(link)).rejects.toBeInstanceOf(GuardDeny)
  })

  test("canonicalPath_should_fail_closed_when_symlinks_loop", async () => {
    // Arrange
    const dir = await fixture()
    await symlink(path.join(dir, "b"), path.join(dir, "a"))
    await symlink(path.join(dir, "a"), path.join(dir, "b"))

    // Act + Assert
    await expect(canonicalPath(path.join(dir, "a"))).rejects.toBeInstanceOf(GuardDeny)
  })

  test("canonicalPath_should_fail_closed_when_parent_component_dangles", async () => {
    // Arrange
    const dir = await fixture()
    await symlink(path.join(dir, "gone"), path.join(dir, "broken"))
    await mkdir(path.join(dir, "broken"), { recursive: false }).catch(() => {})

    // Act + Assert
    await expect(canonicalPath(path.join(dir, "broken", "file.txt"))).rejects.toBeInstanceOf(GuardDeny)
  })
})

describe("stripJsonc", () => {
  test("stripJsonc_should_keep_urls_when_removing_comments", () => {
    // Arrange
    const text = `{
      "$schema": "https://opencode.ai/config.json", // schema
      /* block */ "a": 1,
    }`

    // Act
    const parsed = JSON.parse(stripJsonc(text))

    // Assert
    expect(parsed.$schema).toBe("https://opencode.ai/config.json")
    expect(parsed.a).toBe(1)
  })

  test("stripJsonc_should_keep_slashes_when_inside_string", () => {
    // Arrange
    const text = `{"a": "// not a /* comment"}`

    // Act
    const parsed = JSON.parse(stripJsonc(text))

    // Assert
    expect(parsed.a).toBe("// not a /* comment")
  })
})

describe("globToRegExp", () => {
  test("globToRegExp_should_match_subtree_when_pattern_ends_with_globstar", () => {
    // Arrange
    const regex = globToRegExp("/tmp/**")

    // Act + Assert
    expect(regex.test("/tmp/a/b.txt")).toBe(true)
    expect(regex.test("/tmp")).toBe(false)
    expect(regex.test("/etc/passwd")).toBe(false)
  })

  test("globToRegExp_should_not_cross_slash_when_single_star", () => {
    // Arrange
    const regex = globToRegExp("/home/sewer/*")

    // Act + Assert
    expect(regex.test("/home/sewer/file.txt")).toBe(true)
    expect(regex.test("/home/sewer/nested/file.txt")).toBe(false)
  })
})

describe("decideExternal", () => {
  test("decideExternal_should_deny_when_deny_rule_matches", () => {
    // Arrange
    const rules = { "/tmp/**": "allow", "/home/sewer/projects/nixos-secrets/**": "deny" }

    // Act
    const decision = decideExternal("/home/sewer/projects/nixos-secrets/key.txt", rules)

    // Assert
    expect(decision.effect).toBe("deny")
  })

  test("decideExternal_should_allow_when_allow_rule_matches_and_no_deny", () => {
    // Arrange
    const rules = { "/tmp/**": "allow" }

    // Act
    const decision = decideExternal("/tmp/scratch/out.txt", rules)

    // Assert
    expect(decision.effect).toBe("allow")
  })

  test("decideExternal_should_fail_closed_when_rule_asks", () => {
    // Arrange
    const rules = { "*": "ask", "/tmp/**": "allow" }

    // Act
    const decision = decideExternal("/home/sewer/Documents/notes.txt", rules)

    // Assert
    expect(decision.effect).toBe("deny")
  })

  test("decideExternal_should_fail_closed_when_no_rule_matches", () => {
    // Arrange
    const rules = { "/tmp/**": "allow" }

    // Act
    const decision = decideExternal("/etc/cron.d/evil", rules)

    // Assert
    expect(decision.effect).toBe("deny")
  })
})

describe("extractTargets", () => {
  test("extractTargets_should_collect_direct_and_nested_paths", () => {
    // Arrange
    const input = {
      path: "/tmp/a.txt",
      edits: [{ path: "/tmp/b.txt" }, { from: "/tmp/c.txt", to: "/tmp/d.txt" }],
      content: "ignored",
    }

    // Act
    const targets = extractTargets(input)

    // Assert
    expect(targets).toEqual(["/tmp/a.txt", "/tmp/b.txt", "/tmp/c.txt", "/tmp/d.txt"])
  })

  test("extractTargets_should_return_empty_when_no_paths_present", () => {
    // Arrange
    const input = { content: "x" }

    // Act + Assert
    expect(extractTargets(input)).toEqual([])
  })
})
