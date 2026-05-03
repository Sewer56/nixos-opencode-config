/**
 * Unit tests for file-interp.ts
 *
 * Run: `cd config && bun test plugins/file-interp.test.ts`
 */
import { describe, test, expect, beforeEach, afterEach } from "bun:test"
import { resolvePath, expand } from "./file-interp"
import fsp from "node:fs/promises"
import fs from "node:fs"
import os from "node:os"
import path from "node:path"

// ── Test helpers ─────────────────────────────────────────────────────────────

/** Create a temp dir with given files, return the dir path. */
async function makeTmpDir(files: Record<string, string>): Promise<string> {
  const dir = await fsp.mkdtemp(path.join(os.tmpdir(), "file-interp-test-"))
  for (const [rel, content] of Object.entries(files)) {
    const full = path.join(dir, rel)
    await fsp.mkdir(path.dirname(full), { recursive: true })
    await fsp.writeFile(full, content, "utf8")
  }
  return dir
}

/** Track dirs/files to clean up after tests. */
const cleanup: string[] = []

afterEach(async () => {
  while (cleanup.length) {
    await fsp.rm(cleanup.pop()!, { recursive: true, force: true })
  }
})

/** Set an env var and return a restore function. */
function withEnv(key: string, value: string | undefined): () => void {
  const orig = process.env[key]
  if (value === undefined) {
    delete process.env[key]
  } else {
    process.env[key] = value
  }
  return () => {
    if (orig === undefined) {
      delete process.env[key]
    } else {
      process.env[key] = orig
    }
  }
}

// ── resolvePath ───────────────────────────────────────────────────────────────

describe("resolvePath", () => {
  const base = "/project"

  test("~/... resolves to $HOME", () => {
    expect(resolvePath("~/foo", base)).toBe(path.join(os.homedir(), "foo"))
  })

  test("bare ~ resolves to $HOME", () => {
    expect(resolvePath("~", base)).toBe(os.homedir())
  })

  test("./... resolves relative to baseDir", () => {
    expect(resolvePath("./src/main.ts", base)).toBe("/project/src/main.ts")
  })

  test("../... resolves relative to baseDir", () => {
    expect(resolvePath("../sibling/file.txt", base)).toBe(
      path.resolve("/project", "../sibling/file.txt"),
    )
  })

  test("absolute path used as-is", () => {
    expect(resolvePath("/etc/hosts", base)).toBe("/etc/hosts")
  })

  test("bare name (no prefix) resolves relative to baseDir", () => {
    expect(resolvePath("README.md", base)).toBe("/project/README.md")
  })
})

// ── expand: env tokens ────────────────────────────────────────────────────────

describe("expand: {env:...} tokens", () => {
  test("replaces {env:VAR} with the value", async () => {
    const restore = withEnv("FILE_INTERP_TEST_ENV", "hello")
    try {
      const result = await expand("value={env:FILE_INTERP_TEST_ENV}", "/tmp")
      expect(result).toBe("value=hello")
    } finally {
      restore()
    }
  })

  test("replaces {env:VAR} with empty string when unset", async () => {
    const restore = withEnv("FILE_INTERP_DEFINITELY_NOT_SET", undefined)
    try {
      const result = await expand("value=[{env:FILE_INTERP_DEFINITELY_NOT_SET}]", "/tmp")
      expect(result).toBe("value=[]")
    } finally {
      restore()
    }
  })

  test("plain text VARIABLE_NAME is left untouched", async () => {
    const result = await expand("path=GENERAL_RULES_PATH", "/tmp")
    expect(result).toBe("path=GENERAL_RULES_PATH")
  })
})

// ── expand: file tokens ───────────────────────────────────────────────────────

describe("expand: {file:...} tokens", () => {
  test("replaces {file:./...} with file content", async () => {
    const dir = await makeTmpDir({ "marker.txt": "MARKER-ALPHA-7742" })
    cleanup.push(dir)
    const result = await expand(`code: {file:./marker.txt}`, dir)
    expect(result).toBe("code: MARKER-ALPHA-7742")
  })

  test("trims file content", async () => {
    const dir = await makeTmpDir({ "padded.txt": "  hello world  \n" })
    cleanup.push(dir)
    const result = await expand(`{file:./padded.txt}`, dir)
    expect(result).toBe("hello world")
  })

  test("{file:~/...} resolves to home-relative file", async () => {
    const tmpFile = path.join(os.homedir(), ".file-interp-test-tmp.txt")
    await fsp.writeFile(tmpFile, "HOME_MARKER", "utf8")
    cleanup.push(tmpFile)
    const result = await expand(`{file:~/.file-interp-test-tmp.txt}`, "/tmp")
    expect(result).toBe("HOME_MARKER")
  })

  test("{file:../...} resolves relative to baseDir parent", async () => {
    const parent = await makeTmpDir({ "parent-marker.txt": "PARENT_MARKER" })
    const child = path.join(parent, "subdir")
    await fsp.mkdir(child, { recursive: true })
    cleanup.push(parent)
    const result = await expand(`{file:../parent-marker.txt}`, child)
    expect(result).toBe("PARENT_MARKER")
  })

  test("{file:./nonexistent} returns empty string (ENOENT)", async () => {
    const dir = await makeTmpDir({})
    cleanup.push(dir)
    const result = await expand(`before{file:./nope.txt}after`, dir)
    expect(result).toBe("beforeafter")
  })

  test("{file:/absolute/nonexistent} returns empty string (ENOENT, no fallback)", async () => {
    const result = await expand(`{file:/tmp/file-interp-no-such-absolute.txt}`, "/tmp")
    expect(result).toBe("")
  })

  test("multiple {file:...} tokens in one string", async () => {
    const dir = await makeTmpDir({
      "a.txt": "AAA",
      "b.txt": "BBB",
    })
    cleanup.push(dir)
    const result = await expand(`{file:./a.txt} and {file:./b.txt}`, dir)
    expect(result).toBe("AAA and BBB")
  })
})

// ── expand: mixed tokens ──────────────────────────────────────────────────────

describe("expand: mixed {env:...} and {file:...}", () => {
  test("env and file tokens both expanded in same string", async () => {
    const dir = await makeTmpDir({ "name.txt": "Alice" })
    cleanup.push(dir)
    const restore = withEnv("FILE_INTERP_TEST_REGION", "us-west")
    try {
      const result = await expand(
        `name={file:./name.txt} region={env:FILE_INTERP_TEST_REGION}`,
        dir,
      )
      expect(result).toBe("name=Alice region=us-west")
    } finally {
      restore()
    }
  })
})

// ── expand: no tokens ─────────────────────────────────────────────────────────

describe("expand: no tokens", () => {
  test("returns text unchanged when no tokens present", async () => {
    const result = await expand("plain text with no tokens", "/tmp")
    expect(result).toBe("plain text with no tokens")
  })

  test("returns text unchanged when only plain variable references", async () => {
    const result = await expand("path=GENERAL_RULES_PATH home=$HOME", "/tmp")
    expect(result).toBe("path=GENERAL_RULES_PATH home=$HOME")
  })
})

// ── expand: config-dir fallback ────────────────────────────────────────────────

describe("expand: config-dir fallback", () => {
  test("{file:./...} falls back to CONFIG_DIR when not found in baseDir", async () => {
    // CONFIG_DIR is derived from import.meta.url in file-interp.ts, so it
    // resolves to the actual config/ directory. Place a temp fixture there.
    const configDir = path.dirname(
      path.dirname(new URL(import.meta.url).pathname.replace(/^\/([A-Za-z]:)/, "$1")),
    )
    const fixtureRel = "plugins/.test-fallback-marker.txt"
    const fixtureAbs = path.join(configDir, fixtureRel)
    await fsp.writeFile(fixtureAbs, "FALLBACK_MARKER", "utf8")
    cleanup.push(fixtureAbs)

    // Use a baseDir where the file does NOT exist
    const fakeDir = await makeTmpDir({})
    cleanup.push(fakeDir)

    const result = await expand(`{file:./${fixtureRel}}`, fakeDir)
    expect(result).toBe("FALLBACK_MARKER")
  })

  test("fallback NOT used when file exists in baseDir", async () => {
    const configDir = path.dirname(
      path.dirname(new URL(import.meta.url).pathname.replace(/^\/([A-Za-z]:)/, "$1")),
    )
    const fixtureRel = "plugins/.test-fallback-primary.txt"
    const configFixture = path.join(configDir, fixtureRel)
    await fsp.writeFile(configFixture, "SHOULD_NOT_SEE_THIS", "utf8")
    cleanup.push(configFixture)

    // Create the file IN baseDir — this should win
    const dir = await makeTmpDir({ [fixtureRel]: "PRIMARY_MARKER" })
    cleanup.push(dir)

    const result = await expand(`{file:./${fixtureRel}}`, dir)
    expect(result).toBe("PRIMARY_MARKER")
  })
})

// ── expand: regex lastIndex safety ────────────────────────────────────────────

describe("expand: regex lastIndex safety", () => {
  test("calling expand twice on same strings works (no stale lastIndex)", async () => {
    const dir = await makeTmpDir({ "x.txt": "X" })
    cleanup.push(dir)
    const text = `value={file:./x.txt}`

    const result1 = await expand(text, dir)
    const result2 = await expand(text, dir)
    expect(result1).toBe("value=X")
    expect(result2).toBe("value=X")
  })
})

