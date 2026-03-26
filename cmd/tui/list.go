package tui

import (
	"fmt"
	"strings"

	"github.com/charmbracelet/bubbles/help"
	"github.com/charmbracelet/bubbles/key"
	"github.com/charmbracelet/bubbles/paginator"
	"github.com/charmbracelet/bubbles/spinner"
	tea "github.com/charmbracelet/bubbletea"
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

type listModel struct {
	vault       string
	root        string
	all         []internal.Target
	filtered    []int
	selected    map[int]bool
	cursor      int
	filter      string
	phase       listPhase
	spinner     spinner.Model
	paginator   paginator.Model
	master      map[string]string
	messages    []string
	help        help.Model
	showHelp    bool
	linkHovered bool
	backHovered bool
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
		m.paginator.SetTotalPages(max(1, len(filtered)))
		m.phase = listPhaseBrowse
		return m, nil

	case spinner.TickMsg:
		if m.phase == listPhaseLoading {
			var cmd tea.Cmd
			m.spinner, cmd = m.spinner.Update(msg)
			return m, cmd
		}
		return m, nil

	case tea.MouseMsg:
		var openURL, goBack bool
		m.linkHovered, m.backHovered, openURL, goBack = handleHeaderMouse(msg, m.linkHovered, m.backHovered, true)
		if openURL {
			openBrowser("https://javedab.com")
		}
		if goBack {
			return m, tea.Quit
		}
		// browseView layout: row 3=title, row 4=col header (no filter) / filter line (with filter),
		// then col header, then divider → items at row 6 (no filter) or row 7 (with filter).
		if m.phase == listPhaseBrowse {
			itemsStart := 6
			if m.filter != "" || m.phase == listPhaseFilter {
				itemsStart = 7
			}
			if msg.Y >= itemsStart {
				idx := msg.Y - itemsStart
				start, end := m.paginator.GetSliceBounds(len(m.filtered))
				if idx >= 0 && idx < end-start {
					switch msg.Action {
					case tea.MouseActionMotion:
						m.cursor = idx
					case tea.MouseActionPress:
						if msg.Button == tea.MouseButtonLeft {
							m.cursor = idx
							filteredIdx := m.filtered[start+idx]
							m.selected[filteredIdx] = !m.selected[filteredIdx]
						}
					}
				}
			}
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
	case "backspace":
		if len(m.filter) > 0 {
			m.filter = m.filter[:len(m.filter)-1]
			m.rebuildFiltered()
		}
	case "enter":
		m.phase = listPhaseBrowse
	case "ctrl+c":
		return m, tea.Quit
	default:
		if len(k.String()) == 1 {
			m.filter += k.String()
			m.rebuildFiltered()
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
		if m.cursor > 0 {
			m.cursor--
		} else if !m.paginator.OnFirstPage() {
			m.paginator.PrevPage()
			m.cursor = pageSize - 1
		}
	case key.Matches(k, listKeys.Down):
		start, end := m.paginator.GetSliceBounds(len(m.filtered))
		if m.cursor < end-start-1 {
			m.cursor++
		} else if !m.paginator.OnLastPage() {
			m.paginator.NextPage()
			m.cursor = 0
		}
	case key.Matches(k, listKeys.Toggle):
		if len(m.filtered) > 0 {
			start, _ := m.paginator.GetSliceBounds(len(m.filtered))
			idx := m.filtered[start+m.cursor]
			m.selected[idx] = !m.selected[idx]
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
	case key.Matches(k, listKeys.Filter):
		m.phase = listPhaseFilter
	case k.String() == "esc":
		m.filter = ""
		m.rebuildFiltered()
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
	header := styles.AppHeader("list", m.linkHovered, m.backHovered) + "\n\n"
	switch m.phase {
	case listPhaseLoading:
		return "\n" + header + "  " + m.spinner.View() + "  " + styles.Muted.Render("Scanning projects…") + "\n"
	case listPhaseDone:
		return "\n" + header + m.doneView()
	default:
		return "\n" + header + m.browseView()
	}
}

func (m listModel) browseView() string {
	selectedCount := 0
	for _, sel := range m.selected {
		if sel {
			selectedCount++
		}
	}

	title := fmt.Sprintf("Skills — %d installed", len(m.all))
	if m.filter != "" {
		title = fmt.Sprintf("Skills — %d matching %q", len(m.filtered), m.filter)
	}
	divider := styles.Muted.Render(strings.Repeat("─", 54))
	colHdr := "  " + styles.Muted.Render("[·]") + " " +
		styles.Muted.Render(fmt.Sprintf("%-22s  %s", "skill", "project"))

	s := styles.Header.Render(title) + "   " +
		styles.Muted.Render(fmt.Sprintf("%d / %d selected", selectedCount, len(m.filtered))) + "\n"
	if m.phase == listPhaseFilter {
		s += styles.Warning.Render("filter: "+m.filter+"▌") + "\n"
	} else if m.filter != "" {
		s += styles.Muted.Render("filter: "+m.filter+"  esc to clear") + "\n"
	}
	s += colHdr + "\n"
	s += "  " + divider + "\n"

	start, end := m.paginator.GetSliceBounds(len(m.filtered))
	for i, idx := range m.filtered[start:end] {
		t := m.all[idx]
		cursor := "  "
		if i == m.cursor {
			cursor = styles.SkillName.Render("> ")
		}
		checkbox := "[ ]"
		if m.selected[idx] {
			checkbox = styles.SkillName.Render("[✓]")
		}
		s += fmt.Sprintf("%s%s %-22s  %s\n",
			cursor, checkbox,
			styles.SkillName.Render(t.SkillName),
			styles.Muted.Render(shortPath(t.ProjectPath)),
		)
	}

	s += "  " + divider + "\n"
	if m.paginator.TotalPages > 1 {
		s += "\n" + styles.Muted.Render(m.paginator.View()) + "\n"
	}
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
	s += "\n" + m.help.ShortHelpView([]key.Binding{listKeys.Quit})
	return s
}

func (m *listModel) rebuildFiltered() {
	m.filtered = m.filtered[:0]
	for i, t := range m.all {
		if m.filter == "" || strings.Contains(strings.ToLower(t.SkillName), strings.ToLower(m.filter)) {
			m.filtered = append(m.filtered, i)
		}
	}
	m.paginator.SetTotalPages(max(1, len(m.filtered)))
	m.cursor = 0
	m.paginator.Page = 0
}

// RunList opens the interactive skill browser TUI.
func RunList(vault, root string) error {
	sp := spinner.New()
	sp.Spinner = spinner.Points
	sp.Style = styles.SkillName

	pg := paginator.New()
	pg.Type = paginator.Dots
	pg.PerPage = pageSize
	pg.ActiveDot = styles.SkillName.Render("•")
	pg.InactiveDot = styles.Muted.Render("·")

	m := listModel{
		vault:     vault,
		root:      root,
		phase:     listPhaseLoading,
		spinner:   sp,
		paginator: pg,
		help:      help.New(),
	}
	_, err := tea.NewProgram(m, tea.WithAltScreen(), tea.WithMouseAllMotion()).Run()
	return err
}
