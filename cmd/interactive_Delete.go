package cmd

import (
	"fmt"

	tea "github.com/charmbracelet/bubbletea"
	"skill_Manag/internal"
	"skill_Manag/styles"
)

// deleteItem represents one selectable skill installation in the delete TUI
type deleteItem struct {
	target   internal.Target
	selected bool
}

// deleteModel is the bubbletea model for the interactive delete screen
type deleteModel struct {
	items   []deleteItem
	cursor  int
	dryRun  bool
	phase   phase // reuses phase type from interactive_Sync.go
	results []internal.DeleteResult
}

func (m deleteModel) Init() tea.Cmd {
	return nil
}

func (m deleteModel) Update(msg tea.Msg) (tea.Model, tea.Cmd) {
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
		m.items[m.cursor].selected = !m.items[m.cursor].selected

	case "a":
		// Toggle all
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
			m.deleteSelected()
		} else {
			return m, tea.Quit
		}
	}

	return m, nil
}

// deleteSelected runs the actual deletion for every selected item
func (m *deleteModel) deleteSelected() {
	for _, item := range m.items {
		if !item.selected {
			continue
		}
		result := internal.DeleteSkill(item.target, m.dryRun)
		m.results = append(m.results, result)
	}
}

func (m deleteModel) View() string {
	if m.phase == phaseDone {
		return m.deleteResultsView()
	}
	return m.deleteSelectView()
}

func (m deleteModel) deleteSelectView() string {
	s := styles.Error.Render("Select skills to delete") + "\n"
	s += styles.Muted.Render("↑/↓ navigate   space toggle   a select all   enter confirm   q quit") + "\n\n"

	for i, item := range m.items {
		cursor := "  "
		if i == m.cursor {
			cursor = styles.Error.Render("> ")
		}

		// Start with nothing selected — safer default for a destructive action
		checkbox := "[ ]"
		if item.selected {
			checkbox = styles.Error.Render("[✗]")
		}

		row := fmt.Sprintf("%s%s %s  %s",
			cursor,
			checkbox,
			styles.SkillName.Render(item.target.SkillName),
			styles.Muted.Render(item.target.ProjectPath),
		)
		s += row + "\n"
	}

	selected := 0
	for _, item := range m.items {
		if item.selected {
			selected++
		}
	}
	s += fmt.Sprintf("\n%s\n", styles.Muted.Render(fmt.Sprintf("%d / %d selected", selected, len(m.items))))

	return s
}

func (m deleteModel) deleteResultsView() string {
	s := styles.Header.Render("Delete results") + "\n\n"

	for _, result := range m.results {
		if result.Err != nil {
			s += fmt.Sprintf("%s %s  %s\n",
				styles.Error.Render("✗"),
				styles.SkillName.Render(result.Target.SkillName),
				styles.Error.Render(result.Err.Error()),
			)
			continue
		}

		icon := styles.Error.Render("✗")
		verb := "deleted from"
		if m.dryRun {
			icon = styles.Warning.Render("~")
			verb = "would delete from"
		}

		s += fmt.Sprintf("%s %s  %s\n",
			icon,
			styles.SkillName.Render(result.Target.SkillName),
			styles.Muted.Render(verb+" "+result.Target.ProjectPath),
		)
	}

	s += "\n" + styles.Muted.Render("Press enter to exit") + "\n"
	return s
}

// runInteractiveDelete starts the bubbletea TUI for skill deletion.
// Nothing is pre-selected — safer default for a destructive action.
func runInteractiveDelete(targets []internal.Target, dryRun bool) error {
	items := make([]deleteItem, len(targets))
	for i, t := range targets {
		items[i] = deleteItem{target: t, selected: false}
	}

	m := deleteModel{
		items:  items,
		dryRun: dryRun,
		phase:  phaseSelect,
	}

	_, err := tea.NewProgram(m).Run()
	return err
}
