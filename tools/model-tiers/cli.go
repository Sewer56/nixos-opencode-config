package main

import (
	"errors"
	"flag"
	"fmt"
	"io"
	"os"
	"strings"
)

// runCLI dispatches both script-friendly commands and the interactive TUI.
// No global state here: Env is passed in so tests can point commands at a temp
// repo if needed.
func runCLI(env Env, args []string) error {
	if len(args) == 0 {
		return runTUI(env, "")
	}

	switch args[0] {
	case "tui":
		profile, err := parseOptionalProfileFlag("tui", args[1:])
		if err != nil {
			return err
		}
		return runTUI(env, profile)
	case "configure":
		profile, err := parseConfigureArgs(args[1:])
		if err != nil {
			return err
		}
		return runTUI(env, profile)
	case "status":
		return cmdStatus(env, os.Stdout)
	case "models":
		return cmdModels(env, os.Stdout, args[1:])
	case "apply":
		return cmdApply(env, os.Stdout, args[1:])
	case "work":
		return cmdApply(env, os.Stdout, append([]string{"work"}, args[1:]...))
	case "set":
		return cmdSet(env, os.Stdout, args[1:])
	case "help", "-h", "--help":
		printHelp(os.Stdout)
		return nil
	default:
		return fmt.Errorf("unknown command: %s", args[0])
	}
}

func parseOptionalProfileFlag(name string, args []string) (string, error) {
	fs := flag.NewFlagSet(name, flag.ContinueOnError)
	fs.SetOutput(io.Discard)
	profile := fs.String("profile", "", "initial profile")
	if err := fs.Parse(args); err != nil {
		return "", err
	}
	if fs.NArg() != 0 {
		return "", fmt.Errorf("usage: opencode-model-tiers %s [--profile normal|work]", name)
	}
	return *profile, nil
}

func parseConfigureArgs(args []string) (string, error) {
	fs := flag.NewFlagSet("configure", flag.ContinueOnError)
	fs.SetOutput(io.Discard)
	profileFlag := fs.String("profile", "", "initial profile")
	if err := fs.Parse(args); err != nil {
		return "", err
	}
	if fs.NArg() > 1 {
		return "", errors.New("usage: opencode-model-tiers configure [profile]")
	}
	if fs.NArg() == 1 {
		return fs.Arg(0), nil
	}
	return *profileFlag, nil
}

func printHelp(w io.Writer) {
	fmt.Fprint(w, `opencode-model-tiers

Usage:
  opencode-model-tiers                         open TUI
  opencode-model-tiers tui [--profile work]    open TUI
  opencode-model-tiers configure [profile]     open TUI
  opencode-model-tiers status                  show config and current assignments
  opencode-model-tiers models [--work]         list opencode models
  opencode-model-tiers apply <profile> [--dry-run]
  opencode-model-tiers work [--dry-run]        shortcut for apply work
  opencode-model-tiers set <profile> <tier> <model> [--no-validate]

Profiles: normal, work
Tiers: LOW, MED, HIGH

TUI keys:
  h/l or ←/→   switch profile
  j/k or ↑/↓   switch tier/model
  enter         choose tier model
  type          filter models in picker
  s             save tier config
  a             apply selected profile to agent files
  q             quit
`)
}

func cmdStatus(env Env, w io.Writer) error {
	cfg, err := loadConfig(env)
	if err != nil {
		return err
	}
	fmt.Fprintf(w, "tier file: %s\n", rel(env, env.TierFile))
	for _, profile := range sortedProfiles(cfg) {
		fmt.Fprintf(w, "\n[%s]\n", profile)
		for _, tier := range tierOrder {
			fmt.Fprintf(w, "%-4s %s\n", tier, cfg[profile][tier])
		}
	}

	counts, err := currentCounts(env)
	if err != nil {
		return err
	}
	fmt.Fprintf(w, "\ncurrent marked agent models: %s\n", rel(env, env.AgentDir))
	for _, tier := range tierOrder {
		total := 0
		for _, count := range counts[tier] {
			total += count
		}
		fmt.Fprintf(w, "%-4s %d\n", tier, total)
		for _, item := range sortedCounts(counts[tier]) {
			fmt.Fprintf(w, "     %3d %s\n", item.Count, item.Model)
		}
	}
	return nil
}

func cmdModels(env Env, w io.Writer, args []string) error {
	fs := flag.NewFlagSet("models", flag.ContinueOnError)
	fs.SetOutput(io.Discard)
	workOnly := fs.Bool("work", false, "show only work provider models")
	if err := fs.Parse(args); err != nil {
		return err
	}
	if fs.NArg() != 0 {
		return errors.New("usage: opencode-model-tiers models [--work]")
	}
	models, err := availableModels(env)
	if err != nil {
		return err
	}
	for _, model := range models {
		if *workOnly && !strings.HasPrefix(model, workProvider) {
			continue
		}
		fmt.Fprintln(w, model)
	}
	return nil
}

func cmdApply(env Env, w io.Writer, args []string) error {
	dryRun, positional, err := takeBoolFlag(args, "dry-run")
	if err != nil {
		return err
	}
	if len(positional) != 1 {
		return errors.New("usage: opencode-model-tiers apply <profile> [--dry-run]")
	}

	profile := positional[0]
	cfg, err := loadConfig(env)
	if err != nil {
		return err
	}
	values, ok := cfg[profile]
	if !ok {
		return fmt.Errorf("unknown profile: %s", profile)
	}
	if profile == "work" {
		if err := validateWork(values); err != nil {
			return err
		}
	}

	result, err := applyProfile(env, values, dryRun)
	if err != nil {
		return err
	}
	for _, path := range sortedFileKeys(result.Files) {
		fmt.Fprintf(w, "%s: %d\n", rel(env, path), result.Files[path])
	}
	action := "updated"
	if dryRun {
		action = "would update"
	}
	fmt.Fprintf(w, "%s: %d line(s), %d file(s)\n", action, result.Lines, len(result.Files))
	for _, tier := range tierOrder {
		if result.Tiers[tier] > 0 {
			fmt.Fprintf(w, "  %s: %d -> %s\n", tier, result.Tiers[tier], values[tier])
		}
	}
	return nil
}

func cmdSet(env Env, w io.Writer, args []string) error {
	noValidate, positional, err := takeBoolFlag(args, "no-validate")
	if err != nil {
		return err
	}
	if len(positional) != 3 {
		return errors.New("usage: opencode-model-tiers set <profile> <tier> <model> [--no-validate]")
	}

	profile := positional[0]
	tier := strings.ToUpper(positional[1])
	model := positional[2]
	if !contains(tierOrder, tier) {
		return fmt.Errorf("unknown tier: %s", tier)
	}

	cfg, err := loadConfig(env)
	if err != nil {
		return err
	}
	values, ok := cfg[profile]
	if !ok {
		return fmt.Errorf("unknown profile: %s", profile)
	}
	if !noValidate {
		if err := validateKnown(env, model); err != nil {
			return err
		}
	}

	nextValues := cloneTierSet(values)
	nextValues[tier] = model
	if profile == "work" {
		if err := validateWork(nextValues); err != nil {
			return err
		}
	}
	cfg[profile] = nextValues
	if err := saveConfig(env, cfg); err != nil {
		return err
	}

	fmt.Fprintf(w, "updated: %s\n", rel(env, env.TierFile))
	for _, t := range tierOrder {
		fmt.Fprintf(w, "%-4s %s\n", t, nextValues[t])
	}
	return nil
}

// takeBoolFlag accepts a boolean flag before or after positional arguments.
// Standard flag.FlagSet stops parsing at the first positional arg, but our
// README examples prefer `apply normal --dry-run`.
func takeBoolFlag(args []string, name string) (bool, []string, error) {
	flagText := "--" + name
	value := false
	positional := make([]string, 0, len(args))
	for _, arg := range args {
		switch arg {
		case flagText:
			value = true
		default:
			if strings.HasPrefix(arg, flagText+"=") {
				return false, nil, fmt.Errorf("%s does not take a value", flagText)
			}
			if strings.HasPrefix(arg, "-") {
				return false, nil, fmt.Errorf("unknown flag: %s", arg)
			}
			positional = append(positional, arg)
		}
	}
	return value, positional, nil
}
