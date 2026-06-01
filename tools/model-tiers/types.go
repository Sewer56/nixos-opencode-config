package main

// Tier markers are intentionally plain strings because they are written in
// Markdown frontmatter comments (`# LOW`, `# MED`, `# HIGH`). Keeping exact text
// here makes the regex and JSON file easy to compare with agent files.
const (
	tierLow  = "LOW"
	tierMed  = "MED"
	tierHigh = "HIGH"
)

var tierOrder = []string{tierLow, tierMed, tierHigh}

const workProvider = "sewer-axonhub-work/"

// Config mirrors scripts/model-tiers.json:
//
//	{
//	  "normal": { "LOW": "provider/model", ... },
//	  "work":   { "LOW": "sewer-axonhub-work/model", ... }
//	}
type Config map[string]TierSet

type TierSet map[string]string

// Env stores repo-relative paths after walking upward from the current working
// directory. This lets the tool work from repo root or any nested directory.
type Env struct {
	Root     string
	TierFile string
	AgentDir string
}

// ApplyResult is shared by CLI and TUI previews.
type ApplyResult struct {
	Files map[string]int // absolute path -> changed line count
	Tiers map[string]int // LOW/MED/HIGH -> changed line count
	Lines int
}

type countItem struct {
	Model string
	Count int
}
