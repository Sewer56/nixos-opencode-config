package main

import (
	"errors"
	"fmt"
	"strings"

	"github.com/charmbracelet/bubbles/textinput"
	tea "github.com/charmbracelet/bubbletea"
	"github.com/charmbracelet/lipgloss"
)

// The TUI has two modes:
//   - tier view: choose profile/tier, save, or apply
//   - model picker: type to filter `opencode models`, select one model
//
// The app mutates in-memory Config first. Pressing `s` saves model-tiers.json;
// pressing `a` applies the selected profile to agent files.
type mode int

const (
	modeMain mode = iota
	modePicker
)

type appModel struct {
	env        Env
	cfg        Config
	profiles   []string
	models     []string
	counts     map[string]map[string]int
	profileIdx int
	tierIdx    int
	mode       mode
	input      textinput.Model
	pickIdx    int
	message    string
	width      int
}

var (
	titleStyle    = lipgloss.NewStyle().Bold(true).Foreground(lipgloss.Color("63"))
	activeStyle   = lipgloss.NewStyle().Bold(true).Foreground(lipgloss.Color("42"))
	selectedStyle = lipgloss.NewStyle().Foreground(lipgloss.Color("229")).Background(lipgloss.Color("63"))
	helpStyle     = lipgloss.NewStyle().Foreground(lipgloss.Color("240"))
	errorStyle    = lipgloss.NewStyle().Foreground(lipgloss.Color("196"))
)

func runTUI(env Env, initialProfile string) error {
	m, err := newAppModel(env, initialProfile)
	if err != nil {
		return err
	}
	_, err = tea.NewProgram(m, tea.WithAltScreen()).Run()
	return err
}

func newAppModel(env Env, initialProfile string) (appModel, error) {
	cfg, err := loadConfig(env)
	if err != nil {
		return appModel{}, err
	}
	models, err := availableModels(env)
	if err != nil {
		return appModel{}, err
	}
	counts, err := currentCounts(env)
	if err != nil {
		return appModel{}, err
	}
	profiles := sortedProfiles(cfg)
	if len(profiles) == 0 {
		return appModel{}, errors.New("no profiles configured")
	}

	profileIdx := 0
	if initialProfile != "" {
		found := false
		for index, profile := range profiles {
			if profile == initialProfile {
				profileIdx = index
				found = true
				break
			}
		}
		if !found {
			return appModel{}, fmt.Errorf("unknown profile: %s", initialProfile)
		}
	}

	input := textinput.New()
	input.Placeholder = "filter models"
	input.CharLimit = 128
	input.Width = 60

	return appModel{
		env:        env,
		cfg:        cfg,
		profiles:   profiles,
		models:     models,
		counts:     counts,
		profileIdx: profileIdx,
		input:      input,
	}, nil
}

func (m appModel) Init() tea.Cmd { return textinput.Blink }

func (m appModel) Update(msg tea.Msg) (tea.Model, tea.Cmd) {
	switch msg := msg.(type) {
	case tea.WindowSizeMsg:
		m.width = msg.Width
		m.input.Width = max(30, min(80, msg.Width-8))
		return m, nil
	case tea.KeyMsg:
		if m.mode == modePicker {
			return m.updatePicker(msg)
		}
		return m.updateMain(msg)
	}
	return m, nil
}

func (m appModel) updateMain(msg tea.KeyMsg) (tea.Model, tea.Cmd) {
	switch msg.String() {
	case "q", "ctrl+c":
		return m, tea.Quit
	case "h", "left":
		m.profileIdx = (m.profileIdx - 1 + len(m.profiles)) % len(m.profiles)
	case "l", "right":
		m.profileIdx = (m.profileIdx + 1) % len(m.profiles)
	case "k", "up":
		m.tierIdx = (m.tierIdx - 1 + len(tierOrder)) % len(tierOrder)
	case "j", "down":
		m.tierIdx = (m.tierIdx + 1) % len(tierOrder)
	case "enter", " ":
		m.mode = modePicker
		m.pickIdx = 0
		m.input.SetValue("")
		m.input.Focus()
		m.message = ""
		return m, textinput.Blink
	case "s":
		if err := m.save(); err != nil {
			m.message = "save failed: " + err.Error()
		} else {
			m.message = "saved " + rel(m.env, m.env.TierFile)
		}
	case "a":
		if err := m.apply(); err != nil {
			m.message = "apply failed: " + err.Error()
		} else {
			m.message = "applied " + m.profile()
		}
	}
	return m, nil
}

func (m appModel) updatePicker(msg tea.KeyMsg) (tea.Model, tea.Cmd) {
	filtered := m.filteredModels()
	switch msg.String() {
	case "ctrl+c":
		return m, tea.Quit
	case "esc":
		m.mode = modeMain
		m.input.Blur()
		return m, nil
	case "enter":
		if len(filtered) > 0 {
			m.cfg[m.profile()][m.tier()] = filtered[m.pickIdx]
			m.mode = modeMain
			m.input.Blur()
		}
		return m, nil
	case "up", "ctrl+p":
		if len(filtered) > 0 {
			m.pickIdx = (m.pickIdx - 1 + len(filtered)) % len(filtered)
		}
		return m, nil
	case "down", "ctrl+n":
		if len(filtered) > 0 {
			m.pickIdx = (m.pickIdx + 1) % len(filtered)
		}
		return m, nil
	case "pgup":
		m.pickIdx = max(0, m.pickIdx-10)
		return m, nil
	case "pgdown":
		m.pickIdx = min(max(0, len(filtered)-1), m.pickIdx+10)
		return m, nil
	}

	old := m.input.Value()
	var cmd tea.Cmd
	m.input, cmd = m.input.Update(msg)
	if m.input.Value() != old {
		m.pickIdx = 0
	}
	filtered = m.filteredModels()
	if m.pickIdx >= len(filtered) {
		m.pickIdx = max(0, len(filtered)-1)
	}
	return m, cmd
}

func (m appModel) View() string {
	if m.mode == modePicker {
		return m.viewPicker()
	}
	return m.viewMain()
}

func (m appModel) viewMain() string {
	var b strings.Builder
	b.WriteString(titleStyle.Render("opencode model tiers"))
	b.WriteString("\n\n")

	for i, profile := range m.profiles {
		label := " " + profile + " "
		if i == m.profileIdx {
			b.WriteString(selectedStyle.Render(label))
		} else {
			b.WriteString(label)
		}
		b.WriteString(" ")
	}
	b.WriteString("\n\n")

	profile := m.profile()
	for i, tier := range tierOrder {
		line := fmt.Sprintf("%-4s %s", tier, m.cfg[profile][tier])
		if i == m.tierIdx {
			line = activeStyle.Render("> " + line)
		} else {
			line = "  " + line
		}
		b.WriteString(line)
		b.WriteString("\n")
	}

	result, err := applyProfile(m.env, m.cfg[profile], true)
	b.WriteString("\n")
	if err != nil {
		b.WriteString(errorStyle.Render("preview failed: " + err.Error()))
		b.WriteString("\n")
	} else {
		b.WriteString(fmt.Sprintf("preview: %d line(s), %d file(s) would change\n", result.Lines, len(result.Files)))
		for _, tier := range tierOrder {
			if result.Tiers[tier] > 0 {
				b.WriteString(fmt.Sprintf("  %s: %d -> %s\n", tier, result.Tiers[tier], m.cfg[profile][tier]))
			}
		}
	}

	b.WriteString("\ncurrent marked assignments:\n")
	for _, tier := range tierOrder {
		total := 0
		for _, count := range m.counts[tier] {
			total += count
		}
		b.WriteString(fmt.Sprintf("  %-4s %d", tier, total))
		items := sortedCounts(m.counts[tier])
		if len(items) > 0 {
			b.WriteString("  ")
			limit := min(2, len(items))
			parts := make([]string, 0, limit)
			for _, item := range items[:limit] {
				parts = append(parts, fmt.Sprintf("%d×%s", item.Count, item.Model))
			}
			b.WriteString(strings.Join(parts, ", "))
		}
		b.WriteString("\n")
	}

	if m.message != "" {
		b.WriteString("\n")
		b.WriteString(m.message)
		b.WriteString("\n")
	}
	b.WriteString("\n")
	b.WriteString(helpStyle.Render("h/l profile • j/k tier • enter choose • s save • a apply • q quit"))
	return b.String()
}

func (m appModel) viewPicker() string {
	filtered := m.filteredModels()
	var b strings.Builder
	b.WriteString(titleStyle.Render(fmt.Sprintf("choose %s %s", m.profile(), m.tier())))
	b.WriteString("\n")
	b.WriteString(m.input.View())
	b.WriteString("\n\n")

	if len(filtered) == 0 {
		b.WriteString(errorStyle.Render("no models match"))
		b.WriteString("\n")
	} else {
		start := max(0, m.pickIdx-8)
		end := min(len(filtered), start+18)
		if end-start < 18 {
			start = max(0, end-18)
		}
		for i := start; i < end; i++ {
			line := filtered[i]
			if i == m.pickIdx {
				b.WriteString(selectedStyle.Render(" " + line + " "))
			} else {
				b.WriteString(" " + line)
			}
			b.WriteString("\n")
		}
		b.WriteString(fmt.Sprintf("\n%d/%d\n", m.pickIdx+1, len(filtered)))
	}
	b.WriteString("\n")
	b.WriteString(helpStyle.Render("type filter • ↑/↓ move • enter select • esc back"))
	return b.String()
}

func (m appModel) profile() string { return m.profiles[m.profileIdx] }
func (m appModel) tier() string    { return tierOrder[m.tierIdx] }

func (m appModel) filteredModels() []string {
	return filterModels(m.profile(), m.models, m.input.Value())
}

func (m appModel) save() error {
	if values, ok := m.cfg["work"]; ok {
		if err := validateWork(values); err != nil {
			return err
		}
	}
	return saveConfig(m.env, m.cfg)
}

func (m *appModel) apply() error {
	profile := m.profile()
	if profile == "work" {
		if err := validateWork(m.cfg[profile]); err != nil {
			return err
		}
	}
	result, err := applyProfile(m.env, m.cfg[profile], false)
	if err != nil {
		return err
	}
	m.counts, err = currentCounts(m.env)
	if err != nil {
		return err
	}
	m.message = fmt.Sprintf("applied %s: %d line(s), %d file(s)", profile, result.Lines, len(result.Files))
	return nil
}
