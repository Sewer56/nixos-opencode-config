package main

import (
	"bytes"
	"encoding/json"
	"fmt"
	"os"
	"sort"
	"strings"
)

func loadConfig(env Env) (*LoadedConfig, error) {
	data, err := os.ReadFile(env.TierFile)
	if err != nil {
		return nil, err
	}
	var cfg Config
	if err := json.Unmarshal(data, &cfg); err != nil {
		return nil, err
	}
	tierOrder := deriveTierOrder(env, cfg)
	if err := validateConfig(cfg, tierOrder); err != nil {
		return nil, err
	}
	return &LoadedConfig{TierOrder: tierOrder, Profiles: cfg}, nil
}

// deriveTierOrder extracts the canonical tier list from three sources:
//  1. $tierOrder from config (primary order, if present)
//  2. Tier keys from all profiles
//  3. Tiers discovered by scanning agent markdown files for # <TIER> tags
//
// The result is deduplicated: $tierOrder entries first (in their order),
// then remaining profile tiers alphabetically, then remaining discovered
// tiers alphabetically.
func deriveTierOrder(env Env, cfg Config) []string {
	// 1. Read $tierOrder from config.
	var ordered []string
	if order, ok := cfg["$tierOrder"]; ok {
		ordered = make([]string, len(order))
		for k, v := range order {
			var idx int
			if _, err := fmt.Sscanf(k, "%d", &idx); err != nil {
				continue
			}
			if idx >= 0 && idx < len(ordered) {
				ordered[idx] = v
			}
		}
	}

	// 2. Collect tier keys from all profiles.
	profileTiers := map[string]bool{}
	for profile, values := range cfg {
		if strings.HasPrefix(profile, "$") {
			continue
		}
		for tier := range values {
			profileTiers[tier] = true
		}
	}

	// 3. Discover tiers from agent files.
	discovered, _ := discoverTiersFromFiles(env)

	// 4. Build union, preserving $tierOrder first.
	seen := map[string]bool{}
	var result []string
	for _, tier := range ordered {
		if tier != "" && !seen[tier] {
			seen[tier] = true
			result = append(result, tier)
		}
	}
	remaining := make([]string, 0, len(profileTiers))
	for tier := range profileTiers {
		if !seen[tier] {
			remaining = append(remaining, tier)
		}
	}
	sort.Strings(remaining)
	for _, tier := range remaining {
		seen[tier] = true
		result = append(result, tier)
	}
	for _, tier := range discovered {
		if !seen[tier] {
			seen[tier] = true
			result = append(result, tier)
		}
	}
	return result
}

// saveConfig writes atomically so a failed write never leaves the tier file
// half-written.
func saveConfig(env Env, loaded *LoadedConfig) error {
	if err := validateConfig(loaded.Profiles, loaded.TierOrder); err != nil {
		return err
	}
	data, err := marshalConfig(loaded.Profiles, loaded.TierOrder)
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
func marshalConfig(cfg Config, tierOrder []string) ([]byte, error) {
	profiles := sortedProfiles(cfg)
	var out bytes.Buffer
	out.WriteString("{\n")
	// Emit $tierOrder as the first key.
	out.WriteString(`  "$tierOrder": {`)
	for i, tier := range tierOrder {
		if i > 0 {
			out.WriteString(",")
		}
		fmt.Fprintf(&out, `"%d": "%s"`, i, tier)
	}
	if len(profiles) > 0 {
		out.WriteString("},\n")
	} else {
		out.WriteString("}\n")
	}
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

func validateConfig(cfg Config, tierOrder []string) error {
	if len(profileNames(cfg)) == 0 {
		return fmt.Errorf("tier config must contain at least one profile")
	}

	// All profiles must have identical tier key sets.
	var firstProfile string
	var firstKeys map[string]bool
	for profile, values := range cfg {
		if strings.HasPrefix(profile, "$") {
			continue
		}
		if values == nil {
			return fmt.Errorf("profile %q must be an object", profile)
		}
		keys := map[string]bool{}
		for tier := range values {
			keys[tier] = true
		}
		if firstKeys == nil {
			firstProfile = profile
			firstKeys = keys
		} else {
			if len(keys) != len(firstKeys) {
				return fmt.Errorf("profile %q tier keys differ from %q", profile, firstProfile)
			}
			for tier := range keys {
				if !firstKeys[tier] {
					return fmt.Errorf("profile %q tier keys differ from %q", profile, firstProfile)
				}
			}
		}
	}

	// Every tier that IS defined in profiles must have a non-empty value.
	for profile, values := range cfg {
		if strings.HasPrefix(profile, "$") {
			continue
		}
		for tier := range values {
			if strings.TrimSpace(values[tier]) == "" {
				return fmt.Errorf("profile %q has empty model for %s", profile, tier)
			}
		}
	}
	return nil
}

// validateWork is intentionally strict: work mode should never accidentally use
// personal-provider models.
func validateWork(values TierSet, tierOrder []string) error {
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

func cloneTierSet(values TierSet, tierOrder []string) TierSet {
	copyValues := TierSet{}
	for _, tier := range tierOrder {
		copyValues[tier] = values[tier]
	}
	return copyValues
}
