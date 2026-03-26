package tui

import (
	"fmt"
	"sort"
	"strings"

	"github.com/charmbracelet/bubbles/help"
	"github.com/charmbracelet/bubbles/key"
	"github.com/charmbracelet/bubbles/paginator"
	"github.com/charmbracelet/bubbles/progress"
	"github.com/charmbracelet/bubbles/spinner"
	tea "github.com/charmbracelet/bubbletea"
	"skill_Manag/internal"
	"skill_Manag/styles"
)

type pushTickMsg struct{}

func pushTickCmd() tea.Cmd {
	return func() tea.Msg { return pushTickMsg{} }
}

type pushScanDoneMsg struct {
	items     []skillItem
	master    map[string]string
	allSkills []string
	err       error
}

type editItem struct {
	skillName string
	mandatory bool
}

type pushModel struct {
	vault       string
	root        string
	mandatory   []string
	items       []skillItem
	master      map[string]string
	allSkills   []string
	cursor      int
	editing     bool
	editItems   []editItem
	editCursor  int
	phase       phase
	spinner     spinner.Model
	progress    progress.Model
	paginator   paginator.Model
	syncIdx     int
	results     []internal.SyncResult
	help        help.Model
	showHelp    bool
	err         error
	linkHovered bool
	backHovered bool
}

func (m pushModel) Init() tea.Cmd {
	return tea.Batch(m.spinner.Tick, pushScanCmd(m.vault, m.root, m.mandatory))
}

func pushScanCmd(vault, root string, mandatory []string) tea.Cmd {
	return func() tea.Msg {
		master, err := internal.ReadMasterSkills(vault)
		if err != nil {
			return pushScanDoneMsg{err: err}
		}
		// Collect all vault skill names for the edit overlay
		allSkills := make([]string, 0, len(master))
		for name := range master {
			allSkills = append(allSkills, name)
		}
		sort.Strings(allSkills)

		// Filter master to only the mandatory skills that exist in the vault
		pushSkills := make(map[string]string)
		for _, name := range mandatory {
			if path, ok := master[name]; ok {
				pushSkills[name] = path
			}
		}
		if len(pushSkills) == 0 {
			return pushScanDoneMsg{allSkills: allSkills}
		}
		targets, err := internal.FindPushTargets(root, pushSkills)
		if err != nil {
			return pushScanDoneMsg{err: err}
		}
		return pushScanDoneMsg{items: groupSyncTargets(targets), master: pushSkills, allSkills: allSkills}
	}
}

func (m pushModel) Update(msg tea.Msg) (tea.Model, tea.Cmd) {
	switch msg := msg.(type) {
	case pushScanDoneMsg:
		if msg.err != nil {
			m.err = msg.err
			m.phase = phaseDone
			return m, nil
		}
		m.allSkills = msg.allSkills
		if len(msg.items) == 0 {
			m.phase = phaseDone
			return m, nil
		}
		m.items = msg.items
		m.master = msg.master
		m.paginator.SetTotalPages(len(msg.items))
		m.phase = phaseSelect
		return m, nil

	case pushTickMsg:
		if m.phase != phaseSyncing {
			return m, nil
		}
		if m.syncIdx >= len(m.items) {
			m.phase = phaseDone
			return m, nil
		}
		item := m.items[m.syncIdx]
		for _, target := range item.targets {
			result := internal.SyncSkill(m.master, target, false)
			m.results = append(m.results, result)
		}
		m.syncIdx++
		pct := float64(m.syncIdx) / float64(len(m.items))
		var cmds []tea.Cmd
		cmds = append(cmds, m.progress.SetPercent(pct))
		if m.syncIdx < len(m.items) {
			cmds = append(cmds, pushTickCmd())
		} else {
			m.phase = phaseDone
		}
		return m, tea.Batch(cmds...)

	case progress.FrameMsg:
		pm, cmd := m.progress.Update(msg)
		m.progress = pm.(progress.Model)
		return m, cmd

	case spinner.TickMsg:
		if m.phase == phaseLoading {
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
			if m.editing {
				m.editing = false
				return m, nil
			}
			return m, tea.Quit
		}
		// Edit overlay — mouse support for checklist items.
		// editView layout: row 3=title, row 4=divider → items at row 5.
		if m.editing {
			const editItemsStart = 5
			if msg.Y >= editItemsStart {
				idx := msg.Y - editItemsStart
				if idx >= 0 && idx < len(m.editItems) {
					switch msg.Action {
					case tea.MouseActionMotion:
						m.editCursor = idx
					case tea.MouseActionPress:
						if msg.Button == tea.MouseButtonLeft {
							m.editCursor = idx
							m.editItems[idx].mandatory = !m.editItems[idx].mandatory
						}
					}
				}
			}
			return m, nil
		}
		// selectView layout: row 3=title, row 4=col header, row 5=divider → items at row 6.
		if m.phase == phaseSelect {
			const itemsStart = 6
			if msg.Y >= itemsStart {
				idx := msg.Y - itemsStart
				start, end := m.paginator.GetSliceBounds(len(m.items))
				if idx >= 0 && idx < end-start {
					switch msg.Action {
					case tea.MouseActionMotion:
						m.cursor = idx
					case tea.MouseActionPress:
						if msg.Button == tea.MouseButtonLeft {
							m.cursor = idx
							m.items[start+idx].selected = !m.items[start+idx].selected
						}
					}
				}
			}
		}
		return m, nil

	case tea.KeyMsg:
		if m.phase == phaseLoading {
			if msg.String() == "ctrl+c" {
				return m, tea.Quit
			}
			return m, nil
		}
		if m.phase == phaseDone {
			return m, tea.Quit
		}

		// Edit overlay — handle separately
		if m.editing {
			switch msg.String() {
			case "ctrl+c":
				return m, tea.Quit
			case "q", "alt+left", "esc":
				m.editing = false
			case "up", "k":
				if m.editCursor > 0 {
					m.editCursor--
				}
			case "down", "j":
				if m.editCursor < len(m.editItems)-1 {
					m.editCursor++
				}
			case " ":
				if len(m.editItems) > 0 {
					m.editItems[m.editCursor].mandatory = !m.editItems[m.editCursor].mandatory
				}
			case "enter":
				// Collect new mandatory and save
				newMandatory := []string{}
				for _, item := range m.editItems {
					if item.mandatory {
						newMandatory = append(newMandatory, item.skillName)
					}
				}
				if err := SaveVaultMandatory(m.vault, newMandatory); err != nil {
					m.err = err
					m.phase = phaseDone
					return m, nil
				}
				m.mandatory = newMandatory
				m.editing = false
				// Re-run scan with updated mandatory
				m.items = nil
				m.results = nil
				m.syncIdx = 0
				m.master = nil
				m.phase = phaseLoading
				return m, tea.Batch(m.spinner.Tick, pushScanCmd(m.vault, m.root, m.mandatory))
			}
			return m, nil
		}

		switch {
		case key.Matches(msg, pushKeys.Quit):
			return m, tea.Quit
		case key.Matches(msg, pushKeys.Help):
			m.showHelp = !m.showHelp
		case key.Matches(msg, pushKeys.Up):
			if m.cursor > 0 {
				m.cursor--
			} else if !m.paginator.OnFirstPage() {
				m.paginator.PrevPage()
				m.cursor = pageSize - 1
			}
		case key.Matches(msg, pushKeys.Down):
			start, end := m.paginator.GetSliceBounds(len(m.items))
			if m.cursor < end-start-1 {
				m.cursor++
			} else if !m.paginator.OnLastPage() {
				m.paginator.NextPage()
				m.cursor = 0
			}
		case key.Matches(msg, pushKeys.Toggle):
			start, _ := m.paginator.GetSliceBounds(len(m.items))
			m.items[start+m.cursor].selected = !m.items[start+m.cursor].selected
		case key.Matches(msg, pushKeys.All):
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
		case key.Matches(msg, pushKeys.Edit):
			if m.phase == phaseSelect {
				mandatorySet := make(map[string]bool)
				for _, name := range m.mandatory {
					mandatorySet[name] = true
				}
				m.editItems = make([]editItem, len(m.allSkills))
				for i, name := range m.allSkills {
					m.editItems[i] = editItem{skillName: name, mandatory: mandatorySet[name]}
				}
				m.editCursor = 0
				m.editing = true
			}
		case key.Matches(msg, pushKeys.Confirm):
			if m.phase == phaseSelect {
				selected := []skillItem{}
				for _, item := range m.items {
					if item.selected {
						selected = append(selected, item)
					}
				}
				if len(selected) == 0 {
					return m, nil
				}
				m.items = selected
				m.syncIdx = 0
				m.phase = phaseSyncing
				return m, tea.Batch(pushTickCmd(), m.progress.SetPercent(0))
			}
		}
	}
	return m, nil
}

func (m pushModel) View() string {
	header := styles.AppHeader("push", m.linkHovered, m.backHovered) + "\n\n"
	if m.editing {
		return "\n" + header + m.editView()
	}
	switch m.phase {
	case phaseLoading:
		return "\n" + header + "  " + m.spinner.View() + "  " + styles.Muted.Render("Scanning projects…") + "\n"
	case phaseSyncing:
		label := fmt.Sprintf("  Pushing  %d / %d", m.syncIdx+1, len(m.items))
		return "\n" + header + styles.Header.Render(label) + "\n\n  " + m.progress.View() + "\n"
	case phaseDone:
		return "\n" + header + m.resultsView()
	default:
		return "\n" + header + m.selectView()
	}
}

func (m pushModel) selectView() string {
	selected := 0
	for _, item := range m.items {
		if item.selected {
			selected++
		}
	}

	divider := styles.Muted.Render(strings.Repeat("─", 52))
	colHdr := "  " + styles.Muted.Render("[·]") + " " +
		styles.Muted.Render(fmt.Sprintf("%-22s  %s", "skill", "projects"))

	s := styles.Warning.Render("Select skills to push") + "   " +
		styles.Muted.Render(fmt.Sprintf("%d / %d selected", selected, len(m.items))) + "\n"
	s += colHdr + "\n"
	s += "  " + divider + "\n"

	start, end := m.paginator.GetSliceBounds(len(m.items))
	for i, item := range m.items[start:end] {
		cursor := "  "
		if i == m.cursor {
			cursor = styles.Warning.Render("> ")
		}
		checkbox := "[ ]"
		if item.selected {
			checkbox = styles.Warning.Render("[✓]")
		}
		projectCount := fmt.Sprintf("%d project", len(item.targets))
		if len(item.targets) != 1 {
			projectCount += "s"
		}
		s += fmt.Sprintf("%s%s %-22s  %s\n",
			cursor, checkbox,
			styles.SkillName.Render(item.skillName),
			styles.Muted.Render(projectCount),
		)
	}

	s += "  " + divider + "\n"
	if m.paginator.TotalPages > 1 {
		s += "\n" + styles.Muted.Render(m.paginator.View()) + "\n"
	}
	s += "\n" + m.helpView()
	return s
}

func (m pushModel) editView() string {
	divider := styles.Muted.Render(strings.Repeat("─", 52))
	s := styles.Warning.Render("Edit mandatory skills") + "\n"
	s += "  " + divider + "\n"
	for i, item := range m.editItems {
		cursor := "  "
		if i == m.editCursor {
			cursor = styles.Warning.Render("> ")
		}
		checkbox := "[ ]"
		if item.mandatory {
			checkbox = styles.Warning.Render("[✓]")
		}
		s += fmt.Sprintf("%s%s %s\n", cursor, checkbox, styles.SkillName.Render(item.skillName))
	}
	s += "  " + divider + "\n"
	s += "\n" + styles.Muted.Render("  ↑/↓ navigate  space toggle  enter save  esc/q cancel") + "\n"
	return s
}

func (m pushModel) resultsView() string {
	if m.err != nil {
		return styles.Error.Render("✗ "+m.err.Error()) + "\n\n" +
			m.help.ShortHelpView([]key.Binding{pushKeys.Quit}) + "\n"
	}
	if len(m.results) == 0 {
		msg := "No mandatory skills configured in vault config."
		if len(m.mandatory) > 0 {
			msg = "No opted-in projects found."
		}
		return styles.Muted.Render(msg) + "\n\n" +
			m.help.ShortHelpView([]key.Binding{pushKeys.Quit}) + "\n"
	}
	s := styles.Header.Render("Push results") + "\n\n"
	bySkill := make(map[string][]internal.SyncResult)
	order := []string{}
	for _, result := range m.results {
		name := result.Target.SkillName
		if _, seen := bySkill[name]; !seen {
			order = append(order, name)
		}
		bySkill[name] = append(bySkill[name], result)
	}
	for _, skillName := range order {
		results := bySkill[skillName]
		errCount, fileCount := 0, 0
		for _, r := range results {
			if r.Err != nil {
				errCount++
			} else {
				fileCount += len(r.Files)
			}
		}
		pushed := len(results) - errCount
		icon := styles.Success.Render("✓")
		if errCount > 0 && pushed == 0 {
			icon = styles.Error.Render("✗")
		}
		projectCount := fmt.Sprintf("%d project", pushed)
		if pushed != 1 {
			projectCount += "s"
		}
		summary := fmt.Sprintf("%s (%d files)", projectCount, fileCount)
		if errCount > 0 {
			summary += styles.Error.Render(fmt.Sprintf("  %d error(s)", errCount))
		}
		s += fmt.Sprintf("%s %-22s %s\n",
			icon,
			styles.SkillName.Render(skillName),
			styles.Muted.Render("pushed to "+summary),
		)
	}
	s += "\n" + m.help.ShortHelpView([]key.Binding{pushKeys.Quit}) + "\n"
	return s
}

func (m pushModel) helpView() string {
	if m.showHelp {
		return m.help.FullHelpView(pushKeys.FullHelp())
	}
	return m.help.ShortHelpView(pushKeys.ShortHelp())
}

// RunPush opens the interactive push TUI.
func RunPush(vault, root string, mandatory []string) error {
	sp := spinner.New()
	sp.Spinner = spinner.Points
	sp.Style = styles.SkillName

	prog := progress.New(
		progress.WithGradient("#626262", "#FFDB58"),
		progress.WithoutPercentage(),
		progress.WithWidth(40),
	)

	pg := paginator.New()
	pg.Type = paginator.Dots
	pg.PerPage = pageSize
	pg.ActiveDot = styles.Warning.Render("•")
	pg.InactiveDot = styles.Muted.Render("·")

	m := pushModel{
		vault:     vault,
		root:      root,
		mandatory: mandatory,
		phase:     phaseLoading,
		spinner:   sp,
		progress:  prog,
		paginator: pg,
		help:      help.New(),
	}
	_, err := tea.NewProgram(m, tea.WithAltScreen(), tea.WithMouseAllMotion()).Run()
	return err
}
