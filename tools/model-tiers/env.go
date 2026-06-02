package main

import (
	"fmt"
	"os"
	"path/filepath"
)

// findEnv walks upward until it finds repo markers used by this tool. Avoid
// hardcoded absolute paths so `go run`, `nix run`, and built binaries all work.
func findEnv() (Env, error) {
	cwd, err := os.Getwd()
	if err != nil {
		return Env{}, err
	}

	for dir := cwd; ; dir = filepath.Dir(dir) {
		env := Env{
			Root:     dir,
			TierFile: filepath.Join(dir, "scripts", "model-tiers.json"),
		}
		// Collect all agent directories that exist.
		for _, candidate := range []string{
			filepath.Join(dir, "config", "agent"),
			filepath.Join(dir, ".opencode", "agent"),
		} {
			if dirExists(candidate) {
				env.AgentDirs = append(env.AgentDirs, candidate)
			}
		}
		if fileExists(env.TierFile) && len(env.AgentDirs) > 0 {
			return env, nil
		}
		if parent := filepath.Dir(dir); parent == dir {
			break
		}
	}

	return Env{}, fmt.Errorf("could not find repo root from %s (need scripts/model-tiers.json and config/agent)", cwd)
}

func fileExists(path string) bool {
	info, err := os.Stat(path)
	return err == nil && !info.IsDir()
}

func dirExists(path string) bool {
	info, err := os.Stat(path)
	return err == nil && info.IsDir()
}

func rel(env Env, path string) string {
	value, err := filepath.Rel(env.Root, path)
	if err != nil {
		return path
	}
	return value
}
