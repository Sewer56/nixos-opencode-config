package main

import (
	"bytes"
	"fmt"
	"io/fs"
	"os"
	"path/filepath"
	"regexp"
	"sort"
	"strings"
)

// buildModelLineRE constructs a regex that matches one tagged frontmatter model
// line. Tiers are sorted longest-first so that e.g. "HIGH-FAST" matches before
// "HIGH". Each tier name is regexp-quoted to handle special characters.
func buildModelLineRE(tiers []string) *regexp.Regexp {
	// Sort by length descending so longer tiers match first.
	sorted := make([]string, len(tiers))
	copy(sorted, tiers)
	sort.Slice(sorted, func(i, j int) bool {
		return len(sorted[i]) > len(sorted[j])
	})
	var alt strings.Builder
	for i, t := range sorted {
		if i > 0 {
			alt.WriteString("|")
		}
		alt.WriteString(regexp.QuoteMeta(t))
	}
	pattern := fmt.Sprintf(`^(\s*model:\s*)(\S+)(\s*#\s*(%s)\b.*)$`, alt.String())
	return regexp.MustCompile(pattern)
}

type taggedModelLine struct {
	Model string
	Tier  string
}

var modelLineDiscoveryRE = regexp.MustCompile(`^\s*model:\s*\S+\s*#\s*(\S+)\b.*$`)

func discoverTiersFromFiles(env Env) ([]string, error) {
	files, err := agentFiles(env)
	if err != nil {
		return nil, err
	}
	seen := map[string]bool{}
	for _, file := range files {
		data, err := os.ReadFile(file)
		if err != nil {
			return nil, err
		}
		for _, line := range strings.SplitAfter(string(data), "\n") {
			body, _ := splitEOL(line)
			m := modelLineDiscoveryRE.FindStringSubmatch(body)
			if len(m) > 0 {
				seen[m[1]] = true
			}
		}
	}
	tiers := make([]string, 0, len(seen))
	for tier := range seen {
		tiers = append(tiers, tier)
	}
	sort.Strings(tiers)
	return tiers, nil
}

func agentFiles(env Env) ([]string, error) {
	var files []string
	for _, dir := range env.AgentDirs {
		err := filepath.WalkDir(dir, func(path string, d fs.DirEntry, err error) error {
			if err != nil {
				return err
			}
			if d.IsDir() {
				return nil
			}
			if strings.HasSuffix(path, ".md") {
				files = append(files, path)
			}
			return nil
		})
		if err != nil {
			return nil, err
		}
	}
	sort.Strings(files)
	return files, nil
}

func parseTaggedModelLine(line string, re *regexp.Regexp) (taggedModelLine, bool) {
	body, _ := splitEOL(line)
	m := re.FindStringSubmatch(body)
	if len(m) == 0 {
		return taggedModelLine{}, false
	}
	return taggedModelLine{Model: m[2], Tier: m[4]}, true
}

func splitEOL(line string) (string, string) {
	if strings.HasSuffix(line, "\r\n") {
		return strings.TrimSuffix(line, "\r\n"), "\r\n"
	}
	if strings.HasSuffix(line, "\n") {
		return strings.TrimSuffix(line, "\n"), "\n"
	}
	return line, ""
}

// rewriteLine returns the original line unless it is a tagged model assignment
// whose tier maps to a different model.
func rewriteLine(line string, values TierSet, re *regexp.Regexp) (newLine string, tier string, changed bool) {
	body, eol := splitEOL(line)
	m := re.FindStringSubmatch(body)
	if len(m) == 0 {
		return line, "", false
	}
	oldModel := m[2]
	tier = m[4]
	newModel := values[tier]
	if newModel == "" || oldModel == newModel {
		return line, tier, false
	}
	return m[1] + newModel + m[3] + eol, tier, true
}

// rewriteContent is pure and easy to test. File IO is isolated in
// applyProfile below.
func rewriteContent(input string, values TierSet, re *regexp.Regexp) (string, map[string]int, int) {
	changesByTier := map[string]int{}
	var out bytes.Buffer
	linesChanged := 0

	for _, line := range strings.SplitAfter(input, "\n") {
		if line == "" {
			continue
		}
		newLine, tier, changed := rewriteLine(line, values, re)
		out.WriteString(newLine)
		if changed {
			changesByTier[tier]++
			linesChanged++
		}
	}

	return out.String(), changesByTier, linesChanged
}

func applyProfile(env Env, values TierSet, dryRun bool, tierOrder []string, re *regexp.Regexp) (ApplyResult, error) {
	result := ApplyResult{Files: map[string]int{}, Tiers: map[string]int{}}
	files, err := agentFiles(env)
	if err != nil {
		return result, err
	}

	for _, file := range files {
		data, err := os.ReadFile(file)
		if err != nil {
			return result, err
		}
		newText, byTier, changedLines := rewriteContent(string(data), values, re)
		if changedLines == 0 {
			continue
		}
		result.Files[file] = changedLines
		result.Lines += changedLines
		for tier, count := range byTier {
			result.Tiers[tier] += count
		}
		if !dryRun {
			if err := writeFileAtomic(file, []byte(newText)); err != nil {
				return result, err
			}
		}
	}
	return result, nil
}

func writeFileAtomic(path string, data []byte) error {
	tmp := path + ".tmp"
	if err := os.WriteFile(tmp, data, 0o644); err != nil {
		return err
	}
	if err := os.Rename(tmp, path); err != nil {
		_ = os.Remove(tmp)
		return err
	}
	return nil
}

func currentCounts(env Env, tierOrder []string, re *regexp.Regexp) (map[string]map[string]int, error) {
	counts := map[string]map[string]int{}
	for _, tier := range tierOrder {
		counts[tier] = map[string]int{}
	}
	files, err := agentFiles(env)
	if err != nil {
		return nil, err
	}
	for _, file := range files {
		data, err := os.ReadFile(file)
		if err != nil {
			return nil, err
		}
		for _, line := range strings.SplitAfter(string(data), "\n") {
			if parsed, ok := parseTaggedModelLine(line, re); ok {
				counts[parsed.Tier][parsed.Model]++
			}
		}
	}
	return counts, nil
}
