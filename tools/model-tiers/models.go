package main

import (
	"errors"
	"fmt"
	"os/exec"
	"strings"
)

// availableModels shells out to opencode because it is the source of truth for
// merged config plus providers. Parsing stays tiny: one `provider/model` per
// line.
func availableModels(env Env) ([]string, error) {
	cmd := exec.Command("opencode", "models")
	cmd.Dir = env.Root
	out, err := cmd.CombinedOutput()
	if err != nil {
		return nil, fmt.Errorf("opencode models failed: %w\n%s", err, strings.TrimSpace(string(out)))
	}
	return parseModelsOutput(string(out))
}

func parseModelsOutput(output string) ([]string, error) {
	seen := map[string]bool{}
	var models []string
	for _, line := range strings.Split(output, "\n") {
		model := strings.TrimSpace(line)
		if model == "" || !strings.Contains(model, "/") || seen[model] {
			continue
		}
		seen[model] = true
		models = append(models, model)
	}
	if len(models) == 0 {
		return nil, errors.New("opencode models returned no models")
	}
	return models, nil
}

func validateKnown(env Env, models ...string) error {
	knownList, err := availableModels(env)
	if err != nil {
		return err
	}
	known := map[string]bool{}
	for _, model := range knownList {
		known[model] = true
	}
	var unknown []string
	for _, model := range models {
		if !known[model] {
			unknown = append(unknown, model)
		}
	}
	if len(unknown) > 0 {
		return fmt.Errorf("unknown model(s): %s", strings.Join(unknown, ", "))
	}
	return nil
}

// filterModels does case-insensitive token filtering. Query `gpt work` matches
// models containing both tokens in any position.
func filterModels(profile string, models []string, query string) []string {
	query = strings.ToLower(strings.TrimSpace(query))
	tokens := strings.Fields(query)
	var out []string
	for _, model := range models {
		if profile == "work" && !strings.HasPrefix(model, workProvider) {
			continue
		}
		lower := strings.ToLower(model)
		matched := true
		for _, token := range tokens {
			if !strings.Contains(lower, token) {
				matched = false
				break
			}
		}
		if matched {
			out = append(out, model)
		}
	}
	return out
}
