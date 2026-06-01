package main

import (
	"os"
	"path/filepath"
	"strings"
	"testing"
)

func TestRewriteContentPreservesTierMarkersAndComments(t *testing.T) {
	input := strings.Join([]string{
		"---",
		"model: old-low # LOW",
		"model: old-med    # MED keep this comment",
		"  model: old-high # HIGH\r",
		"model: unmarked",
		"description: leave me",
		"---",
		"",
	}, "\n")

	output, byTier, changed := rewriteContent(input, TierSet{
		tierLow:  "new-low",
		tierMed:  "new-med",
		tierHigh: "new-high",
	})

	if changed != 3 {
		t.Fatalf("changed lines = %d, want 3", changed)
	}
	for _, tier := range tierOrder {
		if byTier[tier] != 1 {
			t.Fatalf("%s changes = %d, want 1", tier, byTier[tier])
		}
	}

	want := strings.Join([]string{
		"---",
		"model: new-low # LOW",
		"model: new-med    # MED keep this comment",
		"  model: new-high # HIGH\r",
		"model: unmarked",
		"description: leave me",
		"---",
		"",
	}, "\n")
	if output != want {
		t.Fatalf("rewrite mismatch\nwant:\n%q\ngot:\n%q", want, output)
	}
}

func TestApplyProfileDryRunDoesNotWrite(t *testing.T) {
	env := makeTestEnv(t)
	agent := filepath.Join(env.AgentDir, "agent.md")
	writeFile(t, agent, "model: old # LOW\n")

	result, err := applyProfile(env, TierSet{tierLow: "new", tierMed: "med", tierHigh: "high"}, true)
	if err != nil {
		t.Fatal(err)
	}
	if result.Lines != 1 || len(result.Files) != 1 || result.Tiers[tierLow] != 1 {
		t.Fatalf("unexpected result: %+v", result)
	}
	if got := readFile(t, agent); got != "model: old # LOW\n" {
		t.Fatalf("dry run wrote file: %q", got)
	}

	result, err = applyProfile(env, TierSet{tierLow: "new", tierMed: "med", tierHigh: "high"}, false)
	if err != nil {
		t.Fatal(err)
	}
	if result.Lines != 1 {
		t.Fatalf("changed lines = %d, want 1", result.Lines)
	}
	if got := readFile(t, agent); got != "model: new # LOW\n" {
		t.Fatalf("apply output = %q", got)
	}
}

func TestCurrentCountsIgnoresUnmarkedModels(t *testing.T) {
	env := makeTestEnv(t)
	writeFile(t, filepath.Join(env.AgentDir, "a.md"), "model: low # LOW\nmodel: nope\n")
	writeFile(t, filepath.Join(env.AgentDir, "nested", "b.md"), "model: med # MED\n")

	counts, err := currentCounts(env)
	if err != nil {
		t.Fatal(err)
	}
	if counts[tierLow]["low"] != 1 {
		t.Fatalf("LOW low count = %d, want 1", counts[tierLow]["low"])
	}
	if counts[tierMed]["med"] != 1 {
		t.Fatalf("MED med count = %d, want 1", counts[tierMed]["med"])
	}
	if counts[tierLow]["nope"] != 0 {
		t.Fatalf("unmarked model counted")
	}
}

func makeTestEnv(t *testing.T) Env {
	t.Helper()
	root := t.TempDir()
	env := Env{
		Root:     root,
		TierFile: filepath.Join(root, "scripts", "model-tiers.json"),
		AgentDir: filepath.Join(root, "config", "agent"),
	}
	if err := os.MkdirAll(filepath.Dir(env.TierFile), 0o755); err != nil {
		t.Fatal(err)
	}
	if err := os.MkdirAll(env.AgentDir, 0o755); err != nil {
		t.Fatal(err)
	}
	writeFile(t, env.TierFile, `{
  "normal": { "LOW": "low", "MED": "med", "HIGH": "high" },
  "work": { "LOW": "sewer-axonhub-work/low", "MED": "sewer-axonhub-work/med", "HIGH": "sewer-axonhub-work/high" }
}
`)
	return env
}

func writeFile(t *testing.T, path string, text string) {
	t.Helper()
	if err := os.MkdirAll(filepath.Dir(path), 0o755); err != nil {
		t.Fatal(err)
	}
	if err := os.WriteFile(path, []byte(text), 0o644); err != nil {
		t.Fatal(err)
	}
}

func readFile(t *testing.T, path string) string {
	t.Helper()
	data, err := os.ReadFile(path)
	if err != nil {
		t.Fatal(err)
	}
	return string(data)
}
