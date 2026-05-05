/**
 * Unit tests for file-interp.ts
 *
 * Run: `cd config && bun test plugins/file-interp.test.ts`
 */
import { describe, test, expect, beforeEach, afterEach } from "bun:test"
import { resolvePath, expand, MAX_DEPTH } from "./file-interp"
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

// ── expand: file templates ────────────────────────────────────────────────────

describe("expand: file templates", () => {
  test("replaces file template with file content", async () => {
    const dir = await makeTmpDir({ "marker.txt": "MARKER-ALPHA-7742" })
    cleanup.push(dir)
    const result = await expand(`code: {{ file="./marker.txt" }}`, dir)
    expect(result).toBe("code: MARKER-ALPHA-7742")
  })

  test("trims file content", async () => {
    const dir = await makeTmpDir({ "padded.txt": "  hello world  \n" })
    cleanup.push(dir)
    const result = await expand(`{{ file="./padded.txt" }}`, dir)
    expect(result).toBe("hello world")
  })

  test("~/... resolves to home-relative file", async () => {
    const tmpFile = path.join(os.homedir(), ".file-interp-test-tmp.txt")
    await fsp.writeFile(tmpFile, "HOME_MARKER", "utf8")
    cleanup.push(tmpFile)
    const result = await expand(`{{ file="~/.file-interp-test-tmp.txt" }}`, "/tmp")
    expect(result).toBe("HOME_MARKER")
  })

  test("../... resolves relative to baseDir parent", async () => {
    const parent = await makeTmpDir({ "parent-marker.txt": "PARENT_MARKER" })
    const child = path.join(parent, "subdir")
    await fsp.mkdir(child, { recursive: true })
    cleanup.push(parent)
    const result = await expand(`{{ file="../parent-marker.txt" }}`, child)
    expect(result).toBe("PARENT_MARKER")
  })

  test("nonexistent relative file returns empty string (ENOENT)", async () => {
    const dir = await makeTmpDir({})
    cleanup.push(dir)
    const result = await expand(`before{{ file="./nope.txt" }}after`, dir)
    expect(result).toBe("beforeafter")
  })

  test("nonexistent absolute file returns empty string (ENOENT, no fallback)", async () => {
    const result = await expand(`{{ file="/tmp/file-interp-no-such-absolute.txt" }}`, "/tmp")
    expect(result).toBe("")
  })

  test("multiple file templates in one string", async () => {
    const dir = await makeTmpDir({
      "a.txt": "AAA",
      "b.txt": "BBB",
    })
    cleanup.push(dir)
    const result = await expand(`{{ file="./a.txt" }} and {{ file="./b.txt" }}`, dir)
    expect(result).toBe("AAA and BBB")
  })

  test("supports zero whitespace after opening and before closing braces", async () => {
    const dir = await makeTmpDir({ "marker.txt": "TIGHT" })
    cleanup.push(dir)
    const result = await expand(`{{file="./marker.txt"}}`, dir)
    expect(result).toBe("TIGHT")
  })
})

// ── expand: file args and arg tokens ─────────────────────────────────────────

describe("expand: file template args and {arg:...} tokens", () => {
  test("passes a basic arg into an embedded file", async () => {
    const dir = await makeTmpDir({ "tmpl.txt": "domain={arg:domain}" })
    cleanup.push(dir)
    const result = await expand(`{{ file="./tmpl.txt" domain=correctness }}`, dir)
    expect(result).toBe("domain=correctness")
  })

  test("supports quoted arg values with spaces", async () => {
    const dir = await makeTmpDir({ "tmpl.txt": "value={arg:key}" })
    cleanup.push(dir)
    const result = await expand(`{{ file="./tmpl.txt" key="val with spaces" }}`, dir)
    expect(result).toBe("value=val with spaces")
  })

  test("supports multiple args", async () => {
    const dir = await makeTmpDir({ "tmpl.txt": "{arg:key1}/{arg:key2}" })
    cleanup.push(dir)
    const result = await expand(`{{ file="./tmpl.txt" key1=a key2=b }}`, dir)
    expect(result).toBe("a/b")
  })

  test("supports multiline file templates with whitespace between attrs", async () => {
    const dir = await makeTmpDir({ "tmpl.txt": "{arg:key1}/{arg:key2}" })
    cleanup.push(dir)
    const result = await expand(
      `{{
        file = "./tmpl.txt"
        key1 = "a value"
        key2 = b
      }}`,
      dir,
    )
    expect(result).toBe("a value/b")
  })

  test("decodes common escapes in template args", async () => {
    const dir = await makeTmpDir({ "tmpl.txt": "{arg:lines}|{arg:tab}|{arg:quote}|{arg:slash}" })
    cleanup.push(dir)
    const result = await expand(
      `{{ file="./tmpl.txt" lines="one\\ntwo" tab=a\\tb quote="say \\"hi\\"" slash="a\\\\b" }}`,
      dir,
    )
    expect(result).toBe("one\ntwo|a\tb|say \"hi\"|a\\b")
  })

  test("undefined args resolve to empty string", async () => {
    const dir = await makeTmpDir({ "tmpl.txt": "before[{arg:missing}]after" })
    cleanup.push(dir)
    const result = await expand(`{{ file="./tmpl.txt" }}`, dir)
    expect(result).toBe("before[]after")
  })

  test("args can compose nested file paths", async () => {
    const dir = await makeTmpDir({
      "tmpl.txt": `{{ file="./rules/{arg:topic}.md" }}`,
      "rules/testing.md": "TEST_RULE",
    })
    cleanup.push(dir)
    const result = await expand(`{{ file="./tmpl.txt" topic=testing }}`, dir)
    expect(result).toBe("TEST_RULE")
  })

  test("nested files without args do not inherit parent args", async () => {
    const dir = await makeTmpDir({
      "outer.txt": `outer={arg:key}; inner={{ file="./inner.txt" }}`,
      "inner.txt": "inner={arg:key}",
    })
    cleanup.push(dir)
    const result = await expand(`{{ file="./outer.txt" key=OUTER }}`, dir)
    expect(result).toBe("outer=OUTER; inner=inner=")
  })

  test("nested files receive only their own args", async () => {
    const dir = await makeTmpDir({
      "outer.txt": `outer={arg:key}; inner={{ file="./inner.txt" key=INNER other=2 }}`,
      "inner.txt": "inner={arg:key}/{arg:other}/{arg:missing}",
    })
    cleanup.push(dir)
    const result = await expand(`{{ file="./outer.txt" key=OUTER other=1 }}`, dir)
    expect(result).toBe("outer=OUTER; inner=inner=INNER/2/")
  })

  test("supports quoted file paths with spaces", async () => {
    const dir = await makeTmpDir({ "path with spaces.txt": "{arg:key}" })
    cleanup.push(dir)
    const result = await expand(`{{ file="./path with spaces.txt" key=val }}`, dir)
    expect(result).toBe("val")
  })

  test("duplicate arg keys use the last value", async () => {
    const dir = await makeTmpDir({ "tmpl.txt": "{arg:key}" })
    cleanup.push(dir)
    const result = await expand(`{{ file="./tmpl.txt" key=a key=b }}`, dir)
    expect(result).toBe("b")
  })

  test("arg values containing env tokens remain literal", async () => {
    const dir = await makeTmpDir({ "tmpl.txt": "{arg:key}" })
    cleanup.push(dir)
    const restore = withEnv("FILE_INTERP_ARG_LITERAL", "EXPANDED")
    try {
      const result = await expand(`{{ file="./tmpl.txt" key="{env:FILE_INTERP_ARG_LITERAL}" }}`, dir)
      expect(result).toBe("{env:FILE_INTERP_ARG_LITERAL}")
    } finally {
      restore()
    }
  })

  test("arg values containing file templates remain literal", async () => {
    const dir = await makeTmpDir({
      "tmpl.txt": "{arg:key}",
      "secret.txt": "SHOULD_NOT_EXPAND",
    })
    cleanup.push(dir)
    const result = await expand(`{{ file="./tmpl.txt" key="{{ file=\\\"./secret.txt\\\" }}" }}`, dir)
    expect(result).toBe(`{{ file="./secret.txt" }}`)
  })

  test("arg values containing arg tokens remain literal", async () => {
    const dir = await makeTmpDir({ "tmpl.txt": "{arg:key}" })
    cleanup.push(dir)
    const result = await expand(`{{ file="./tmpl.txt" key="{arg:other}" other=expanded }}`, dir)
    expect(result).toBe("{arg:other}")
  })

  test("file templates work with zero args", async () => {
    const dir = await makeTmpDir({ "plain.txt": "PLAIN" })
    cleanup.push(dir)
    const result = await expand(`{{ file="./plain.txt" }}`, dir)
    expect(result).toBe("PLAIN")
  })
})

// ── expand: mixed tokens ──────────────────────────────────────────────────────

describe("expand: mixed {env:...} and file templates", () => {
  test("env tokens and file templates both expand in same string", async () => {
    const dir = await makeTmpDir({ "name.txt": "Alice" })
    cleanup.push(dir)
    const restore = withEnv("FILE_INTERP_TEST_REGION", "us-west")
    try {
      const result = await expand(
        `name={{ file="./name.txt" }} region={env:FILE_INTERP_TEST_REGION}`,
        dir,
      )
      expect(result).toBe("name=Alice region=us-west")
    } finally {
      restore()
    }
  })

  test("env values may inject file templates (env expands before file)", async () => {
    const dir = await makeTmpDir({ "from-env.txt": "ENV_TO_FILE_MARKER" })
    cleanup.push(dir)
    const restore = withEnv("FILE_INTERP_TEST_FILE_TOKEN", `{{ file="./from-env.txt" }}`)
    try {
      const result = await expand(`value={env:FILE_INTERP_TEST_FILE_TOKEN}`, dir)
      expect(result).toBe("value=ENV_TO_FILE_MARKER")
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

  test("leaves empty token forms unchanged", async () => {
    const result = await expand(`empty {env:} and {{ file="" }}`, "/tmp")
    expect(result).toBe(`empty {env:} and {{ file="" }}`)
  })

  test("leaves unclosed token forms unchanged", async () => {
    const result = await expand(`broken {env:FOO and {{ file="./x.txt"`, "/tmp")
    expect(result).toBe(`broken {env:FOO and {{ file="./x.txt"`)
  })
})

// ── expand: config-dir fallback ────────────────────────────────────────────────

describe("expand: config-dir fallback", () => {
  test("relative file template falls back to CONFIG_DIR when not found in baseDir", async () => {
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

    const result = await expand(`{{ file="./${fixtureRel}" }}`, fakeDir)
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

    const result = await expand(`{{ file="./${fixtureRel}" }}`, dir)
    expect(result).toBe("PRIMARY_MARKER")
  })
})

// ── expand: repeated-call safety ──────────────────────────────────────────────

describe("expand: repeated-call safety", () => {
  test("calling expand twice on same strings works (no stale lastIndex)", async () => {
    const dir = await makeTmpDir({ "x.txt": "X" })
    cleanup.push(dir)
    const text = `value={{ file="./x.txt" }}`

    const result1 = await expand(text, dir)
    const result2 = await expand(text, dir)
    expect(result1).toBe("value=X")
    expect(result2).toBe("value=X")
  })
})

// ── expand: recursive file templates ─────────────────────────────────────────

describe("expand: recursive file templates", () => {
  test("recursively expands file templates in imported content", async () => {
    const dir = await makeTmpDir({
      "inner.txt": "INNER_CONTENT",
      "outer.txt": `outer: {{ file="./inner.txt" }}`,
    })
    cleanup.push(dir)
    const result = await expand(`{{ file="./outer.txt" }}`, dir)
    expect(result).toBe("outer: INNER_CONTENT")
  })

  test("expands multi-level chain (3 deep)", async () => {
    const dir = await makeTmpDir({
      "c.txt": "C_VALUE",
      "b.txt": `B:{{ file="./c.txt" }}`,
      "a.txt": `A:{{ file="./b.txt" }}`,
    })
    cleanup.push(dir)
    const result = await expand(`{{ file="./a.txt" }}`, dir)
    expect(result).toBe("A:B:C_VALUE")
  })

  test("detects self-referential cycle and replaces with empty string", async () => {
    const dir = await makeTmpDir({
      "loop.txt": `start {{ file="./loop.txt" }} end`,
    })
    cleanup.push(dir)
    const result = await expand(`{{ file="./loop.txt" }}`, dir)
    // loop.txt is in visited set when its content is recursively expanded;
    // the inner file template finds it in visited → empty string
    expect(result).toBe("start  end")
  })

  test("missing file within recursive chain resolves to empty string", async () => {
    const dir = await makeTmpDir({
      "outer.txt": `prefix {{ file="./missing.txt" }} suffix`,
    })
    cleanup.push(dir)
    const result = await expand(`{{ file="./outer.txt" }}`, dir)
    expect(result).toBe("prefix  suffix")
  })

  test("detects mutual cycle (A→B→A) and breaks it", async () => {
    const dir = await makeTmpDir({
      "a.txt": `A-{{ file="./b.txt" }}`,
      "b.txt": `B-{{ file="./a.txt" }}`,
    })
    cleanup.push(dir)
    const result = await expand(`{{ file="./a.txt" }}`, dir)
    // a.txt content expanded with visited={a}; b.txt found, expanded with
    // visited={a,b}; b's inner file template finds a in visited → ""
    expect(result).toBe("A-B-")
  })

  test("sibling tokens referencing same file both resolve (diamond pattern)", async () => {
    const dir = await makeTmpDir({
      "shared.txt": "SHARED",
      "top.txt": `{{ file="./shared.txt" }} and {{ file="./shared.txt" }}`,
    })
    cleanup.push(dir)
    const result = await expand(`{{ file="./top.txt" }}`, dir)
    // visited is per-ancestor-chain, not global — sibling tokens each get
    // their own chain, so both resolve shared.txt
    expect(result).toBe("SHARED and SHARED")
  })

  test("expands chain exactly MAX_DEPTH levels deep", async () => {
    const files: Record<string, string> = {}
    for (let i = 0; i < MAX_DEPTH; i++) {
      if (i === MAX_DEPTH - 1) {
        files[`d${i}.txt`] = `LEAF`
      } else {
        files[`d${i}.txt`] = `D${i}:{{ file="./d${i + 1}.txt" }}`
      }
    }
    const dir = await makeTmpDir(files)
    cleanup.push(dir)
    const result = await expand(`{{ file="./d0.txt" }}`, dir)
    expect(result).toContain("LEAF")
  })

  test("at MAX_DEPTH, leaves unexpanded file templates as literal text", async () => {
    // Chain of MAX_DEPTH files; the deepest file contains a file template
    // that should NOT be expanded — left as literal text in the output.
    const files: Record<string, string> = {}
    for (let i = 0; i < MAX_DEPTH; i++) {
      if (i === MAX_DEPTH - 1) {
        files[`e${i}.txt`] = `DEEP_{{ file="./e-leaf.txt" }}_{env:FILE_INTERP_DEPTH_ENV}`
      } else {
        files[`e${i}.txt`] = `E${i}:{{ file="./e${i + 1}.txt" }}`
      }
    }
    // The leaf file exists but should never be read because depth is capped
    files["e-leaf.txt"] = `NEVER_READ`
    const dir = await makeTmpDir(files)
    cleanup.push(dir)
    const restore = withEnv("FILE_INTERP_DEPTH_ENV", "YES")
    try {
      const result = await expand(`{{ file="./e0.txt" }}`, dir)
      expect(result).toContain("DEEP_")                  // deepest content present
      expect(result).toContain(`{{ file="./e-leaf.txt" }}`) // file template left as literal text
      expect(result).toContain("YES")                    // env token still expanded at MAX_DEPTH
      expect(result).not.toContain("NEVER_READ")         // leaf file never expanded
    } finally {
      restore()
    }
  })

  test("expands {env:...} inside recursively imported content", async () => {
    const dir = await makeTmpDir({
      "with-env.txt": "region={env:FILE_INTERP_RECURSE_ENV}",
    })
    cleanup.push(dir)
    const restore = withEnv("FILE_INTERP_RECURSE_ENV", "eu-west")
    try {
      const result = await expand(`{{ file="./with-env.txt" }}`, dir)
      expect(result).toBe("region=eu-west")
    } finally {
      restore()
    }
  })

  test("skips recursive expansion when imported file has no tokens", async () => {
    const dir = await makeTmpDir({
      "plain.txt": "NO_TOKENS_HERE",
    })
    cleanup.push(dir)
    const result = await expand(`{{ file="./plain.txt" }}`, dir)
    expect(result).toBe("NO_TOKENS_HERE")
  })

  test("readCache deduplicates raw I/O for same path across branches", async () => {
    const dir = await makeTmpDir({
      "shared.txt": "SHARED",
      "ref-a.txt": `A:{{ file="./shared.txt" }}`,
      "ref-b.txt": `B:{{ file="./shared.txt" }}`,
      "top.txt": `{{ file="./ref-a.txt" }} | {{ file="./ref-b.txt" }}`,
    })
    cleanup.push(dir)
    const result = await expand(`{{ file="./top.txt" }}`, dir)
    // shared.txt raw I/O is read once via readCache, then independently
    // expanded by ref-a and ref-b (each with their own visited snapshot)
    expect(result).toBe("A:SHARED | B:SHARED")
  })

  test("expansion is independent per ancestor chain (no cross-contamination)", async () => {
    // shared.txt references ref-a.txt; ref-a is in ref-a's ancestor chain
    // but NOT in ref-b's ancestor chain. Both branches should expand
    // shared.txt independently with their own visited sets.
    const dir = await makeTmpDir({
      "shared.txt": `S:{{ file="./ref-a.txt" }}`,
      "ref-a.txt": "A_CONTENT",
      "ref-b.txt": `B:{{ file="./shared.txt" }}`,
      "top.txt": `{{ file="./ref-a.txt" }} | {{ file="./ref-b.txt" }}`,
    })
    cleanup.push(dir)
    const result = await expand(`{{ file="./top.txt" }}`, dir)
    // ref-a branch: visited={ref-a} → "A_CONTENT"
    // ref-b branch: visited={ref-b} → shared → visited={ref-b,shared} →
    //   ref-a file template: ref-a NOT in visited → "A_CONTENT"
    // So shared.txt expands to "S:A_CONTENT" from ref-b's perspective
    expect(result).toBe("A_CONTENT | B:S:A_CONTENT")
  })
})
