package cmd

import (
	"fmt"
	"strings"

	tea "github.com/charmbracelet/bubbletea"
	"skill_Manag/internal"
	"skill_Manag/styles"
)

type listPhase int

const (
	listPhaseBrowse listPhase = iota
	listPhaseFilter
	listPhaseDone
)

// listModel is the bubbletea model for the skill browser TUI
type listModel struct {
	all      []internal.Target // full unfiltered list
	filtered []int             // indices into all that match current filter
	selected map[int]bool      // indices into all — persists across filter changes
	cursor   int               // position within filtered
	filter   string
	phase    listPhase
	master   map[string]string // nil if --source not provided
	messages []string          // result lines shown after sync/delete
}

func (m listModel) Init() tea.Cmd {
	return nil
}

func (m listModel) Update(msg tea.Msg) (tea.Model, tea.Cmd) {
	keyMsg, ok := msg.(tea.KeyMsg)
	if !ok {
		return m, nil
	}

	// Filter mode — capture typing
	if m.phase == listPhaseFilter {
		return m.updateFilter(keyMsg)
	}

	// Results screen — any key exits
	if m.phase == listPhaseDone {
		return m, tea.Quit
	}

	return m.updateBrowse(keyMsg)
}

func (m listModel) updateFilter(key tea.KeyMsg) (tea.Model, tea.Cmd) {
	switch key.String() {
	case "esc":
		// Clear filter and return to browse
		m.filter = ""
		m.phase = listPhaseBrowse
		m.rebuildFiltered()
		m.cursor = 0

	case "backspace":
		if len(m.filter) > 0 {
			m.filter = m.filter[:len(m.filter)-1]
			m.rebuildFiltered()
			m.clampCursor()
		}

	case "enter":
		m.phase = listPhaseBrowse

	case "ctrl+c":
		return m, tea.Quit

	default:
		// Append typed character to filter
		if len(key.String()) == 1 {
			m.filter += key.String()
			m.rebuildFiltered()
			m.clampCursor()
		}
	}

	return m, nil
}

func (m listModel) updateBrowse(key tea.KeyMsg) (tea.Model, tea.Cmd) {
	switch key.String() {
	case "ctrl+c", "q":
		return m, tea.Quit

	case "up", "k":
		if m.cursor > 0 {
			m.cursor--
		}

	case "down", "j":
		if m.cursor < len(m.filtered)-1 {
			m.cursor++
		}

	case " ":
		// Toggle selection of item under cursor
		if len(m.filtered) > 0 {
			idx := m.filtered[m.cursor]
			m.selected[idx] = !m.selected[idx]
		}

	case "a":
		// Select all visible / deselect all visible
		allSelected := true
		for _, idx := range m.filtered {
			if !m.selected[idx] {
				allSelected = false
				break
			}
		}
		for _, idx := range m.filtered {
			m.selected[idx] = !allSelected
		}

	case "/":
		m.phase = listPhaseFilter

	case "esc":
		// Clear filter from browse mode
		m.filter = ""
		m.rebuildFiltered()
		m.cursor = 0

	case "s":
		// Sync selected items from master
		if m.master == nil {
			m.messages = append(m.messages, styles.Error.Render("✗ --source not set — cannot sync without a master skills directory"))
			m.phase = listPhaseDone
			return m, nil
		}
		m.runSync()
		m.phase = listPhaseDone

	case "d":
		// Delete selected items
		m.runDelete()
		m.phase = listPhaseDone
	}

	return m, nil
}

// runSync copies master skills into all selected targets
func (m *listModel) runSync() {
	for idx, sel := range m.selected {
		if !sel {
			continue
		}
		target := m.all[idx]
		result := internal.SyncSkill(m.master, target, false)
		if result.Err != nil {
			m.messages = append(m.messages, fmt.Sprintf("%s %s  %s",
				styles.Error.Render("✗"),
				styles.SkillName.Render(target.SkillName),
				styles.Error.Render(result.Err.Error()),
			))
		} else {
			m.messages = append(m.messages, fmt.Sprintf("%s %s  %s",
				styles.Success.Render("✓"),
				styles.SkillName.Render(target.SkillName),
				styles.Muted.Render("synced → "+target.ProjectPath),
			))
		}
	}
}

// runDelete removes all selected skill directories
func (m *listModel) runDelete() {
	for idx, sel := range m.selected {
		if !sel {
			continue
		}
		target := m.all[idx]
		result := internal.DeleteSkill(target, false)
		if result.Err != nil {
			m.messages = append(m.messages, fmt.Sprintf("%s %s  %s",
				styles.Error.Render("✗"),
				styles.SkillName.Render(target.SkillName),
				styles.Error.Render(result.Err.Error()),
			))
		} else {
			m.messages = append(m.messages, fmt.Sprintf("%s %s  %s",
				styles.Error.Render("✗"),
				styles.SkillName.Render(target.SkillName),
				styles.Muted.Render("deleted from "+target.ProjectPath),
			))
		}
	}
}

func (m listModel) View() string {
	if m.phase == listPhaseDone {
		return m.doneView()
	}
	return m.browseView()
}

func (m listModel) browseView() string {
	selectedCount := 0
	for _, sel := range m.selected {
		if sel {
			selectedCount++
		}
	}

	// Header
	header := fmt.Sprintf("Skills — %d installed", len(m.all))
	if m.filter != "" {
		header = fmt.Sprintf("Skills — %d matching %q", len(m.filtered), m.filter)
	}
	if selectedCount > 0 {
		header += fmt.Sprintf("  [%s]", styles.Success.Render(fmt.Sprintf("%d selected", selectedCount)))
	}
	s := styles.Header.Render(header) + "\n"

	// Filter line
	if m.phase == listPhaseFilter {
		s += styles.Warning.Render("Filter: "+m.filter+"▌") + "\n"
	} else if m.filter != "" {
		s += styles.Muted.Render("Filter: "+m.filter+"  (esc to clear)") + "\n"
	} else {
		s += styles.Muted.Render("/ filter   space select   a all   s sync   d delete   q quit") + "\n"
	}
	s += "\n"

	// Item list
	for i, idx := range m.filtered {
		target := m.all[idx]

		cursor := "  "
		if i == m.cursor {
			cursor = styles.Success.Render("> ")
		}

		checkbox := "[ ]"
		if m.selected[idx] {
			checkbox = styles.Success.Render("[✓]")
		}

		s += fmt.Sprintf("%s%s %-25s %s\n",
			cursor,
			checkbox,
			styles.SkillName.Render(target.SkillName),
			styles.Muted.Render(target.ProjectPath),
		)
	}

	return s
}

func (m listModel) doneView() string {
	s := styles.Header.Render("Results") + "\n\n"
	s += strings.Join(m.messages, "\n") + "\n"
	s += "\n" + styles.Muted.Render("Press any key to exit") + "\n"
	return s
}

// rebuildFiltered updates the filtered index list based on current filter string
func (m *listModel) rebuildFiltered() {
	m.filtered = m.filtered[:0]
	for i, t := range m.all {
		if m.filter == "" || strings.Contains(strings.ToLower(t.SkillName), strings.ToLower(m.filter)) {
			m.filtered = append(m.filtered, i)
		}
	}
}

// clampCursor keeps the cursor within bounds after the filtered list changes
func (m *listModel) clampCursor() {
	if m.cursor >= len(m.filtered) {
		m.cursor = max(0, len(m.filtered)-1)
	}
}

func max(a, b int) int {
	if a > b {
		return a
	}
	return b
}

// runInteractiveList starts the skill browser TUI
func runInteractiveList(targets []internal.Target, masterSkills map[string]string) error {
	filtered := make([]int, len(targets))
	for i := range targets {
		filtered[i] = i
	}

	m := listModel{
		all:      targets,
		filtered: filtered,
		selected: make(map[int]bool),
		phase:    listPhaseBrowse,
		master:   masterSkills,
	}

	_, err := tea.NewProgram(m).Run()
	return err
}
