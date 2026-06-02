package main

const workProvider = "sewer-axonhub-work/"

// Config mirrors scripts/model-tiers.json:
//
//	{
//	  "normal": { "LOW": "provider/model", ... },
//	  "work":   { "LOW": "sewer-axonhub-work/model", ... }
//	}
type Config map[string]TierSet

type TierSet map[string]string

// LoadedConfig pairs a loaded profile map with the canonical tier order
// discovered from scripts/model-tiers.json at load time.
type LoadedConfig struct {
	TierOrder []string
	Profiles  Config
}

// Env stores repo-relative paths after walking upward from the current working
// directory. This lets the tool work from repo root or any nested directory.
type Env struct {
	Root      string
	TierFile  string
	AgentDirs []string
}

// ApplyResult is shared by CLI and TUI previews.
type ApplyResult struct {
	Files map[string]int // absolute path -> changed line count
	Tiers map[string]int // tier name -> changed line count
	Lines int
}

type countItem struct {
	Model string
	Count int
}
