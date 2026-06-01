package main

import (
	"bytes"
	"encoding/json"
	"fmt"
	"os"
	"strings"
)

func loadConfig(env Env) (Config, error) {
	data, err := os.ReadFile(env.TierFile)
	if err != nil {
		return nil, err
	}
	var cfg Config
	if err := json.Unmarshal(data, &cfg); err != nil {
		return nil, err
	}
	return cfg, validateConfig(cfg)
}

// saveConfig writes atomically so a failed write never leaves the tier file
// half-written.
func saveConfig(env Env, cfg Config) error {
	if err := validateConfig(cfg); err != nil {
		return err
	}
	data, err := marshalConfig(cfg)
	if err != nil {
		return err
	}
	tmp := env.TierFile + ".tmp"
	if err := os.WriteFile(tmp, data, 0o644); err != nil {
		return err
	}
	return os.Rename(tmp, env.TierFile)
}

// marshalConfig keeps scripts/model-tiers.json human-friendly. encoding/json
// sorts map keys alphabetically, which would write HIGH/LOW/MED. This writer
// keeps tier order as LOW/MED/HIGH and profile order stable.
func marshalConfig(cfg Config) ([]byte, error) {
	profiles := sortedProfiles(cfg)
	var out bytes.Buffer
	out.WriteString("{\n")
	for profileIndex, profile := range profiles {
		profileJSON, err := json.Marshal(profile)
		if err != nil {
			return nil, err
		}
		out.WriteString("  ")
		out.Write(profileJSON)
		out.WriteString(": {\n")
		for tierIndex, tier := range tierOrder {
			modelJSON, err := json.Marshal(cfg[profile][tier])
			if err != nil {
				return nil, err
			}
			out.WriteString("    \"")
			out.WriteString(tier)
			out.WriteString("\": ")
			out.Write(modelJSON)
			if tierIndex != len(tierOrder)-1 {
				out.WriteString(",")
			}
			out.WriteString("\n")
		}
		out.WriteString("  }")
		if profileIndex != len(profiles)-1 {
			out.WriteString(",")
		}
		out.WriteString("\n")
	}
	out.WriteString("}\n")
	return out.Bytes(), nil
}

func validateConfig(cfg Config) error {
	if len(profileNames(cfg)) == 0 {
		return fmt.Errorf("tier config must contain at least one profile")
	}
	for profile, values := range cfg {
		// Allow JSON Schema-style metadata keys such as "$schema" without
		// treating them as model profiles.
		if strings.HasPrefix(profile, "$") {
			continue
		}
		if values == nil {
			return fmt.Errorf("profile %q must be an object", profile)
		}
		for _, tier := range tierOrder {
			if strings.TrimSpace(values[tier]) == "" {
				return fmt.Errorf("profile %q missing %s", profile, tier)
			}
		}
	}
	return nil
}

// validateWork is intentionally strict: work mode should never accidentally use
// personal-provider models.
func validateWork(values TierSet) error {
	var bad []string
	for _, tier := range tierOrder {
		model := values[tier]
		if !strings.HasPrefix(model, workProvider) {
			bad = append(bad, fmt.Sprintf("%s=%s", tier, model))
		}
	}
	if len(bad) > 0 {
		return fmt.Errorf("work profile must use %s models: %s", workProvider, strings.Join(bad, ", "))
	}
	return nil
}

func profileNames(cfg Config) []string {
	profiles := make([]string, 0, len(cfg))
	for profile := range cfg {
		if strings.HasPrefix(profile, "$") {
			continue
		}
		profiles = append(profiles, profile)
	}
	return profiles
}

func cloneTierSet(values TierSet) TierSet {
	copyValues := TierSet{}
	for _, tier := range tierOrder {
		copyValues[tier] = values[tier]
	}
	return copyValues
}
