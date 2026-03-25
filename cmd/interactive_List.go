package cmd

import (
	"fmt"
	"strings"

	"github.com/charmbracelet/bubbles/help"
	"github.com/charmbracelet/bubbles/key"
	"github.com/charmbracelet/bubbles/spinner"
	"github.com/charmbracelet/bubbles/table"
	tea "github.com/charmbracelet/bubbletea"
	"github.com/charmbracelet/lipgloss"
	"skill_Manag/internal"
	"skill_Manag/styles"
)

type listPhase int

const (
	listPhaseLoading listPhase = iota
	listPhaseBrowse
	listPhaseFilter
	listPhaseDone
)

type listScanDoneMsg struct {
	targets      []internal.Target
	masterSkills map[string]string
	err          error
}

// listModel is the bubbletea model for the skill browser TUI
type listModel struct {
	vault    string
	root     string
	all      []internal.Target
	filtered []int
	selected map[int]bool
	cursor   int
	filter   string
	phase    listPhase
	spinner  spinner.Model
	table    table.Model
	master   map[string]string
	messages []string
	help     help.Model
	showHelp bool
}

func (m listModel) Init() tea.Cmd {
	return tea.Batch(m.spinner.Tick, listScanCmd(m.vault, m.root))
}

func listScanCmd(vault, root string) tea.Cmd {
	return func() tea.Msg {
		var masterSkills map[string]string
		if vault != "" {
			var err error
			masterSkills, err = internal.ReadMasterSkills(vault)
			if err != nil {
				return listScanDoneMsg{err: err}
			}
		}
		targets, err := internal.FindAllSkillTargets(root)
		if err != nil {
			return listScanDoneMsg{err: err}
		}
		return listScanDoneMsg{targets: targets, masterSkills: masterSkills}
	}
}

func (m listModel) Update(msg tea.Msg) (tea.Model, tea.Cmd) {
	switch msg := msg.(type) {
	case listScanDoneMsg:
		if msg.err != nil {
			m.phase = listPhaseDone
			m.messages = []string{styles.Error.Render("✗ " + msg.err.Error())}
			return m, nil
		}
		filtered := make([]int, len(msg.targets))
		for i := range msg.targets {
			filtered[i] = i
		}
		m.all = msg.targets
		m.filtered = filtered
		m.selected = make(map[int]bool)
		m.master = msg.masterSkills
		m.table = buildTable(msg.targets, filtered, m.selected)
		m.phase = listPhaseBrowse
		return m, nil

	case spinner.TickMsg:
		if m.phase == listPhaseLoading {
			var cmd tea.Cmd
			m.spinner, cmd = m.spinner.Update(msg)
			return m, cmd
		}
		return m, nil

	case tea.KeyMsg:
		if m.phase == listPhaseLoading {
			if msg.String() == "ctrl+c" {
				return m, tea.Quit
			}
			return m, nil
		}
		if m.phase == listPhaseFilter {
			return m.updateFilter(msg)
		}
		if m.phase == listPhaseDone {
			return m, tea.Quit
		}
		return m.updateBrowse(msg)
	}

	return m, nil
}

func (m listModel) updateFilter(k tea.KeyMsg) (tea.Model, tea.Cmd) {
	switch k.String() {
	case "esc":
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
		if len(k.String()) == 1 {
			m.filter += k.String()
			m.rebuildFiltered()
			m.clampCursor()
		}
	}
	return m, nil
}

func (m listModel) updateBrowse(k tea.KeyMsg) (tea.Model, tea.Cmd) {
	switch {
	case key.Matches(k, listKeys.Quit):
		return m, tea.Quit
	case key.Matches(k, listKeys.Help):
		m.showHelp = !m.showHelp
	case key.Matches(k, listKeys.Up):
		m.table.MoveUp(1)
		m.cursor = m.table.Cursor()
	case key.Matches(k, listKeys.Down):
		m.table.MoveDown(1)
		m.cursor = m.table.Cursor()
	case key.Matches(k, listKeys.Toggle):
		if len(m.filtered) > 0 {
			idx := m.filtered[m.cursor]
			m.selected[idx] = !m.selected[idx]
			m.table = buildTable(m.all, m.filtered, m.selected)
			m.table.SetCursor(m.cursor)
		}
	case key.Matches(k, listKeys.All):
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
		m.table = buildTable(m.all, m.filtered, m.selected)
		m.table.SetCursor(m.cursor)
	case key.Matches(k, listKeys.Filter):
		m.phase = listPhaseFilter
	case k.String() == "esc":
		m.filter = ""
		m.rebuildFiltered()
		m.cursor = 0
	case key.Matches(k, listKeys.Sync):
		if m.master == nil {
			m.messages = []string{styles.Error.Render("✗ vault not configured — run Setup to set a vault path")}
			m.phase = listPhaseDone
			return m, nil
		}
		m.runSync()
		m.phase = listPhaseDone
	case key.Matches(k, listKeys.Delete):
		m.runDelete()
		m.phase = listPhaseDone
	}
	return m, nil
}

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
				styles.Muted.Render("synced → "+shortPath(target.ProjectPath)),
			))
		}
	}
}

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
				styles.Warning.Render("✗"),
				styles.SkillName.Render(target.SkillName),
				styles.Muted.Render("deleted from "+shortPath(target.ProjectPath)),
			))
		}
	}
}

func (m listModel) View() string {
	switch m.phase {
	case listPhaseLoading:
		return "\n  " + m.spinner.View() + "  " + styles.Muted.Render("Scanning projects…") + "\n"
	case listPhaseDone:
		return m.doneView()
	default:
		return m.browseView()
	}
}

func (m listModel) browseView() string {
	selectedCount := 0
	for _, sel := range m.selected {
		if sel {
			selectedCount++
		}
	}

	header := fmt.Sprintf("Skills — %d installed", len(m.all))
	if m.filter != "" {
		header = fmt.Sprintf("Skills — %d matching %q", len(m.filtered), m.filter)
	}
	if selectedCount > 0 {
		header += fmt.Sprintf("  [%s]", styles.Success.Render(fmt.Sprintf("%d selected", selectedCount)))
	}
	s := styles.Header.Render(header) + "\n"

	if m.phase == listPhaseFilter {
		s += styles.Warning.Render("Filter: "+m.filter+"▌") + "\n"
	} else if m.filter != "" {
		s += styles.Muted.Render("Filter: "+m.filter+"  (esc to clear)") + "\n"
	}
	s += "\n"
	s += m.table.View() + "\n"
	s += "\n" + m.helpView()
	return s
}

func (m listModel) helpView() string {
	if m.showHelp {
		return m.help.FullHelpView(listKeys.FullHelp())
	}
	return m.help.ShortHelpView(listKeys.ShortHelp())
}

func (m listModel) doneView() string {
	s := styles.Header.Render("Results") + "\n\n"
	s += strings.Join(m.messages, "\n") + "\n"
	s += "\n" + styles.Muted.Render("Press any key to continue") + "\n"
	return s
}

func (m *listModel) rebuildFiltered() {
	m.filtered = m.filtered[:0]
	for i, t := range m.all {
		if m.filter == "" || strings.Contains(strings.ToLower(t.SkillName), strings.ToLower(m.filter)) {
			m.filtered = append(m.filtered, i)
		}
	}
	m.table = buildTable(m.all, m.filtered, m.selected)
}

func (m *listModel) clampCursor() {
	if m.cursor >= len(m.filtered) {
		m.cursor = max(0, len(m.filtered)-1)
	}
	m.table.SetCursor(m.cursor)
}

func buildTable(all []internal.Target, filtered []int, selected map[int]bool) table.Model {
	cols := []table.Column{
		{Title: "", Width: 3},   // checkbox
		{Title: "Skill", Width: 26},
		{Title: "Project", Width: 36},
	}

	rows := make([]table.Row, len(filtered))
	for i, idx := range filtered {
		t := all[idx]
		check := "[ ]"
		if selected[idx] {
			check = "[✓]"
		}
		rows[i] = table.Row{check, t.SkillName, shortPath(t.ProjectPath)}
	}

	tableStyles := table.DefaultStyles()
	tableStyles.Header = tableStyles.Header.
		Foreground(lipgloss.Color("#626262")).
		BorderStyle(lipgloss.NormalBorder()).
		BorderForeground(lipgloss.Color("#333333")).
		Bold(false)
	tableStyles.Selected = tableStyles.Selected.
		Foreground(lipgloss.Color("#87CEEB")).
		Background(lipgloss.Color("#1a1a2e")).
		Bold(false)

	t := table.New(
		table.WithColumns(cols),
		table.WithRows(rows),
		table.WithFocused(true),
		table.WithHeight(15),
		table.WithStyles(tableStyles),
	)
	return t
}

func runInteractiveList(vault, root string) error {
	sp := spinner.New()
	sp.Spinner = spinner.Points
	sp.Style = styles.SkillName

	m := listModel{
		vault:   vault,
		root:    root,
		phase:   listPhaseLoading,
		spinner: sp,
		help:    help.New(),
	}

	_, err := tea.NewProgram(m, tea.WithAltScreen()).Run()
	return err
}
