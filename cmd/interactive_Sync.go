package cmd

import (
	"fmt"

	tea "github.com/charmbracelet/bubbletea"
	"skill_Manag/internal"
	"skill_Manag/styles"
)

type phase int

const (
	phaseSelect phase = iota
	phaseDone
)

// checkItem represents one selectable skill target in the TUI
type checkItem struct {
	target   internal.Target
	selected bool
}

// model is the bubbletea model for the interactive selection screen
type model struct {
	items   []checkItem
	cursor  int
	master  map[string]string
	dryRun  bool
	phase   phase
	results []internal.SyncResult
}

func (m model) Init() tea.Cmd {
	return nil
}

func (m model) Update(msg tea.Msg) (tea.Model, tea.Cmd) {
	keyMsg, ok := msg.(tea.KeyMsg)
	if !ok {
		return m, nil
	}

	switch keyMsg.String() {
	case "ctrl+c", "q":
		return m, tea.Quit

	case "up", "k":
		if m.cursor > 0 {
			m.cursor--
		}

	case "down", "j":
		if m.cursor < len(m.items)-1 {
			m.cursor++
		}

	case " ":
		// Toggle selection of the item under the cursor
		m.items[m.cursor].selected = !m.items[m.cursor].selected

	case "a":
		// Toggle all — if everything is selected, deselect all, otherwise select all
		allSelected := true
		for _, item := range m.items {
			if !item.selected {
				allSelected = false
				break
			}
		}
		for i := range m.items {
			m.items[i].selected = !allSelected
		}

	case "enter":
		if m.phase == phaseSelect {
			m.phase = phaseDone
			m.syncSelected()
		} else {
			return m, tea.Quit
		}
	}

	return m, nil
}

// syncSelected runs the actual copy for every selected item
func (m *model) syncSelected() {
	for _, item := range m.items {
		if !item.selected {
			continue
		}
		result := internal.SyncSkill(m.master, item.target, m.dryRun)
		m.results = append(m.results, result)
	}
}

func (m model) View() string {
	if m.phase == phaseDone {
		return m.resultsView()
	}
	return m.selectView()
}

func (m model) selectView() string {
	s := styles.Header.Render("Select skills to sync") + "\n"
	s += styles.Muted.Render("↑/↓ navigate   space toggle   a select all   enter confirm   q quit") + "\n\n"

	for i, item := range m.items {
		// Highlight the row under the cursor
		cursor := "  "
		if i == m.cursor {
			cursor = styles.Success.Render("> ")
		}

		checkbox := "[ ]"
		if item.selected {
			checkbox = styles.Success.Render("[✓]")
		}

		row := fmt.Sprintf("%s%s %s  %s",
			cursor,
			checkbox,
			styles.SkillName.Render(item.target.SkillName),
			styles.Muted.Render(item.target.ProjectPath),
		)
		s += row + "\n"
	}

	// Show how many are selected at the bottom
	selected := 0
	for _, item := range m.items {
		if item.selected {
			selected++
		}
	}
	s += fmt.Sprintf("\n%s\n", styles.Muted.Render(fmt.Sprintf("%d / %d selected", selected, len(m.items))))

	return s
}

func (m model) resultsView() string {
	s := styles.Header.Render("Results") + "\n\n"

	for _, result := range m.results {
		if result.Err != nil {
			s += fmt.Sprintf("%s %s  %s\n",
				styles.Error.Render("✗"),
				styles.SkillName.Render(result.Target.SkillName),
				styles.Error.Render(result.Err.Error()),
			)
			continue
		}

		icon := styles.Success.Render("✓")
		verb := "synced"
		if m.dryRun {
			icon = styles.Warning.Render("~")
			verb = "would sync"
		}

		s += fmt.Sprintf("%s %s  %s  %s\n",
			icon,
			styles.SkillName.Render(result.Target.SkillName),
			styles.Muted.Render(result.Target.ProjectPath),
			styles.Muted.Render(fmt.Sprintf("%s (%d files)", verb, len(result.Files))),
		)
	}

	s += "\n" + styles.Muted.Render("Press enter to exit") + "\n"
	return s
}

// runInteractive starts the bubbletea TUI with all targets pre-selected
func runInteractive(targets []internal.Target, masterSkills map[string]string, dryRun bool) error {
	// Pre-select everything — user can deselect what they don't want
	items := make([]checkItem, len(targets))
	for i, t := range targets {
		items[i] = checkItem{target: t, selected: true}
	}

	m := model{
		items:  items,
		master: masterSkills,
		dryRun: dryRun,
		phase:  phaseSelect,
	}

	_, err := tea.NewProgram(m).Run()
	return err
}
