package tui

import (
	"fmt"
	"sort"
	"strings"

	"github.com/charmbracelet/bubbles/help"
	"github.com/charmbracelet/bubbles/key"
	"github.com/charmbracelet/bubbles/paginator"
	"github.com/charmbracelet/bubbles/spinner"
	tea "github.com/charmbracelet/bubbletea"
	"skill_Manag/internal"
	"skill_Manag/styles"
)

type deleteSkillItem struct {
	skillName string
	targets   []internal.Target
	selected  bool
}

type deleteScanDoneMsg struct {
	items []deleteSkillItem
	err   error
}

type deleteModel struct {
	root        string
	items       []deleteSkillItem
	cursor      int
	dryRun      bool
	phase       phase
	spinner     spinner.Model
	paginator   paginator.Model
	results     []internal.DeleteResult
	help        help.Model
	showHelp    bool
	err         error
	linkHovered bool
	backHovered bool
}

func (m deleteModel) Init() tea.Cmd {
	return tea.Batch(m.spinner.Tick, deleteScanCmd(m.root))
}

func deleteScanCmd(root string) tea.Cmd {
	return func() tea.Msg {
		targets, err := internal.FindAllSkillTargets(root)
		if err != nil {
			return deleteScanDoneMsg{err: err}
		}
		return deleteScanDoneMsg{items: groupDeleteTargets(targets)}
	}
}

func groupDeleteTargets(targets []internal.Target) []deleteSkillItem {
	grouped := make(map[string]*deleteSkillItem)
	for _, t := range targets {
		if _, ok := grouped[t.SkillName]; !ok {
			grouped[t.SkillName] = &deleteSkillItem{skillName: t.SkillName}
		}
		grouped[t.SkillName].targets = append(grouped[t.SkillName].targets, t)
	}
	names := make([]string, 0, len(grouped))
	for name := range grouped {
		names = append(names, name)
	}
	sort.Strings(names)
	items := make([]deleteSkillItem, len(names))
	for i, name := range names {
		items[i] = *grouped[name]
	}
	return items
}

func (m deleteModel) Update(msg tea.Msg) (tea.Model, tea.Cmd) {
	switch msg := msg.(type) {
	case deleteScanDoneMsg:
		if msg.err != nil {
			m.err = msg.err
			m.phase = phaseDone
			return m, nil
		}
		if len(msg.items) == 0 {
			m.phase = phaseDone
			return m, nil
		}
		m.items = msg.items
		m.paginator.SetTotalPages(len(msg.items))
		m.phase = phaseSelect
		return m, nil

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
			return m, tea.Quit
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
		if m.phase == phaseConfirm {
			switch msg.String() {
			case "y", "Y", "enter":
				m.phase = phaseDone
				m.deleteSelected()
			case "n", "N", "esc":
				m.phase = phaseSelect
			case "ctrl+c":
				return m, tea.Quit
			}
			return m, nil
		}
		switch {
		case key.Matches(msg, deleteKeys.Quit):
			return m, tea.Quit
		case key.Matches(msg, deleteKeys.Help):
			m.showHelp = !m.showHelp
		case key.Matches(msg, deleteKeys.Up):
			if m.cursor > 0 {
				m.cursor--
			} else if !m.paginator.OnFirstPage() {
				m.paginator.PrevPage()
				m.cursor = pageSize - 1
			}
		case key.Matches(msg, deleteKeys.Down):
			start, end := m.paginator.GetSliceBounds(len(m.items))
			if m.cursor < end-start-1 {
				m.cursor++
			} else if !m.paginator.OnLastPage() {
				m.paginator.NextPage()
				m.cursor = 0
			}
		case key.Matches(msg, deleteKeys.Toggle):
			start, _ := m.paginator.GetSliceBounds(len(m.items))
			m.items[start+m.cursor].selected = !m.items[start+m.cursor].selected
		case key.Matches(msg, deleteKeys.All):
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
		case key.Matches(msg, deleteKeys.Confirm):
			for _, item := range m.items {
				if item.selected {
					m.phase = phaseConfirm
					break
				}
			}
		}
	}
	return m, nil
}

func (m *deleteModel) deleteSelected() {
	for _, item := range m.items {
		if !item.selected {
			continue
		}
		for _, target := range item.targets {
			result := internal.DeleteSkill(target, m.dryRun)
			m.results = append(m.results, result)
		}
	}
}

func (m deleteModel) View() string {
	header := styles.AppHeader("delete", m.linkHovered, m.backHovered) + "\n\n"
	switch m.phase {
	case phaseLoading:
		return "\n" + header + "  " + m.spinner.View() + "  " + styles.Muted.Render("Scanning projects…") + "\n"
	case phaseConfirm:
		return "\n" + header + m.confirmView()
	case phaseDone:
		return "\n" + header + m.resultsView()
	default:
		return "\n" + header + m.selectView()
	}
}

func (m deleteModel) confirmView() string {
	count := 0
	for _, item := range m.items {
		if item.selected {
			count++
		}
	}
	s := styles.Error.Render(fmt.Sprintf("Delete %d skill(s)?", count)) + "\n\n"
	s += styles.Muted.Render("  This will inshallah permanently remove the selected skills from all matching projects.") + "\n\n"
	s += "  " + styles.Warning.Render("y / enter") + styles.Muted.Render("  confirm   ") +
		styles.Warning.Render("n / esc") + styles.Muted.Render("  cancel") + "\n"
	return s
}

func (m deleteModel) selectView() string {
	selected := 0
	for _, item := range m.items {
		if item.selected {
			selected++
		}
	}

	divider := styles.Muted.Render(strings.Repeat("─", 52))
	colHdr := "  " + styles.Muted.Render("[·]") + " " +
		styles.Muted.Render(fmt.Sprintf("%-22s  %s", "skill", "projects"))

	s := styles.Error.Render("Select skills to delete") + "   " +
		styles.Muted.Render(fmt.Sprintf("%d / %d selected", selected, len(m.items))) + "\n"
	s += colHdr + "\n"
	s += "  " + divider + "\n"

	start, end := m.paginator.GetSliceBounds(len(m.items))
	for i, item := range m.items[start:end] {
		cursor := "  "
		if i == m.cursor {
			cursor = styles.Error.Render("> ")
		}
		checkbox := "[ ]"
		if item.selected {
			checkbox = styles.Error.Render("[✗]")
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

func (m deleteModel) resultsView() string {
	if m.err != nil {
		return styles.Error.Render("✗ "+m.err.Error()) + "\n\n" +
			m.help.ShortHelpView([]key.Binding{deleteKeys.Quit}) + "\n"
	}
	if len(m.results) == 0 {
		return styles.Muted.Render("No skills found in any project.") + "\n\n" +
			m.help.ShortHelpView([]key.Binding{deleteKeys.Quit}) + "\n"
	}
	s := styles.Header.Render("Delete results") + "\n\n"
	bySkill := make(map[string][]internal.DeleteResult)
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
		errCount, deleted := 0, 0
		for _, r := range results {
			if r.Err != nil {
				errCount++
			} else {
				deleted++
			}
		}
		icon := styles.Warning.Render("✗")
		verb := "deleted from"
		if m.dryRun {
			icon = styles.Warning.Render("~")
			verb = "would delete from"
		}
		if errCount > 0 && deleted == 0 {
			icon = styles.Error.Render("✗")
		}
		projectCount := fmt.Sprintf("%d project", deleted)
		if deleted != 1 {
			projectCount += "s"
		}
		summary := projectCount
		if errCount > 0 {
			summary += styles.Error.Render(fmt.Sprintf("  %d error(s)", errCount))
		}
		s += fmt.Sprintf("%s %-22s %s\n",
			icon,
			styles.SkillName.Render(skillName),
			styles.Muted.Render(verb+" "+summary),
		)
	}
	s += "\n" + m.help.ShortHelpView([]key.Binding{deleteKeys.Quit}) + "\n"
	return s
}

func (m deleteModel) helpView() string {
	if m.showHelp {
		return m.help.FullHelpView(deleteKeys.FullHelp())
	}
	return m.help.ShortHelpView(deleteKeys.ShortHelp())
}

// RunDelete opens the interactive delete TUI.
func RunDelete(root string, dryRun bool) error {
	sp := spinner.New()
	sp.Spinner = spinner.Points
	sp.Style = styles.SkillName

	pg := paginator.New()
	pg.Type = paginator.Dots
	pg.PerPage = pageSize
	pg.ActiveDot = styles.Error.Render("•")
	pg.InactiveDot = styles.Muted.Render("·")

	m := deleteModel{
		root:      root,
		dryRun:    dryRun,
		phase:     phaseLoading,
		spinner:   sp,
		paginator: pg,
		help:      help.New(),
	}
	_, err := tea.NewProgram(m, tea.WithAltScreen(), tea.WithMouseAllMotion()).Run()
	return err
}
