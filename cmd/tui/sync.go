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

type syncTickMsg struct{}

func syncTickCmd() tea.Cmd {
	return func() tea.Msg { return syncTickMsg{} }
}

type skillItem struct {
	skillName string
	targets   []internal.Target
	selected  bool
}

type syncScanDoneMsg struct {
	items  []skillItem
	master map[string]string
	err    error
}

type syncModel struct {
	vault       string
	root        string
	items       []skillItem
	master      map[string]string
	cursor      int
	dryRun      bool
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

func (m syncModel) Init() tea.Cmd {
	return tea.Batch(m.spinner.Tick, syncScanCmd(m.vault, m.root))
}

func syncScanCmd(vault, root string) tea.Cmd {
	return func() tea.Msg {
		master, err := internal.ReadMasterSkills(vault)
		if err != nil {
			return syncScanDoneMsg{err: err}
		}
		targets, err := internal.FindTargets(root, master)
		if err != nil {
			return syncScanDoneMsg{err: err}
		}
		return syncScanDoneMsg{items: groupSyncTargets(targets), master: master}
	}
}

func groupSyncTargets(targets []internal.Target) []skillItem {
	grouped := make(map[string]*skillItem)
	for _, t := range targets {
		if _, ok := grouped[t.SkillName]; !ok {
			grouped[t.SkillName] = &skillItem{skillName: t.SkillName, selected: true}
		}
		grouped[t.SkillName].targets = append(grouped[t.SkillName].targets, t)
	}
	names := make([]string, 0, len(grouped))
	for name := range grouped {
		names = append(names, name)
	}
	sort.Strings(names)
	items := make([]skillItem, len(names))
	for i, name := range names {
		items[i] = *grouped[name]
	}
	return items
}

func (m syncModel) Update(msg tea.Msg) (tea.Model, tea.Cmd) {
	switch msg := msg.(type) {
	case syncScanDoneMsg:
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
		m.master = msg.master
		m.paginator.SetTotalPages(len(msg.items))
		m.phase = phaseSelect
		return m, nil

	case syncTickMsg:
		if m.phase != phaseSyncing {
			return m, nil
		}
		if m.syncIdx >= len(m.items) {
			m.phase = phaseDone
			return m, nil
		}
		item := m.items[m.syncIdx]
		for _, target := range item.targets {
			result := internal.SyncSkill(m.master, target, m.dryRun)
			m.results = append(m.results, result)
		}
		m.syncIdx++
		pct := float64(m.syncIdx) / float64(len(m.items))
		var cmds []tea.Cmd
		cmds = append(cmds, m.progress.SetPercent(pct))
		if m.syncIdx < len(m.items) {
			cmds = append(cmds, syncTickCmd())
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
		switch {
		case key.Matches(msg, syncKeys.Quit):
			return m, tea.Quit
		case key.Matches(msg, syncKeys.Help):
			m.showHelp = !m.showHelp
		case key.Matches(msg, syncKeys.Up):
			if m.cursor > 0 {
				m.cursor--
			} else if !m.paginator.OnFirstPage() {
				m.paginator.PrevPage()
				m.cursor = pageSize - 1
			}
		case key.Matches(msg, syncKeys.Down):
			start, end := m.paginator.GetSliceBounds(len(m.items))
			if m.cursor < end-start-1 {
				m.cursor++
			} else if !m.paginator.OnLastPage() {
				m.paginator.NextPage()
				m.cursor = 0
			}
		case key.Matches(msg, syncKeys.Toggle):
			start, _ := m.paginator.GetSliceBounds(len(m.items))
			m.items[start+m.cursor].selected = !m.items[start+m.cursor].selected
		case key.Matches(msg, syncKeys.All):
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
		case key.Matches(msg, syncKeys.Confirm):
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
			return m, tea.Batch(syncTickCmd(), m.progress.SetPercent(0))
		}
	}
	return m, nil
}

func (m syncModel) View() string {
	header := styles.AppHeader("sync", m.linkHovered, m.backHovered) + "\n\n"
	switch m.phase {
	case phaseLoading:
		return "\n" + header + "  " + m.spinner.View() + "  " + styles.Muted.Render("Scanning projects…") + "\n"
	case phaseSyncing:
		label := fmt.Sprintf("  Syncing  %d / %d", m.syncIdx+1, len(m.items))
		return "\n" + header + styles.Header.Render(label) + "\n\n  " + m.progress.View() + "\n"
	case phaseDone:
		return "\n" + header + m.resultsView()
	default:
		return "\n" + header + m.selectView()
	}
}

func (m syncModel) selectView() string {
	selected := 0
	for _, item := range m.items {
		if item.selected {
			selected++
		}
	}

	divider := styles.Muted.Render(strings.Repeat("─", 52))
	colHdr := "  " + styles.Muted.Render("[·]") + " " +
		styles.Muted.Render(fmt.Sprintf("%-22s  %s", "skill", "projects"))

	s := styles.Header.Render("Select skills to sync") + "   " +
		styles.Muted.Render(fmt.Sprintf("%d / %d selected", selected, len(m.items))) + "\n"
	s += colHdr + "\n"
	s += "  " + divider + "\n"

	start, end := m.paginator.GetSliceBounds(len(m.items))
	for i, item := range m.items[start:end] {
		cursor := "  "
		if i == m.cursor {
			cursor = styles.Success.Render("> ")
		}
		checkbox := "[ ]"
		if item.selected {
			checkbox = styles.Success.Render("[✓]")
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

func (m syncModel) resultsView() string {
	if m.err != nil {
		return styles.Error.Render("✗ "+m.err.Error()) + "\n\n" +
			m.help.ShortHelpView([]key.Binding{syncKeys.Quit}) + "\n"
	}
	if len(m.results) == 0 {
		return styles.Muted.Render("No matching skills found in any project.") + "\n\n" +
			m.help.ShortHelpView([]key.Binding{syncKeys.Quit}) + "\n"
	}
	s := styles.Header.Render("Results") + "\n\n"
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
		synced := len(results) - errCount
		icon := styles.Success.Render("✓")
		verb := "synced to"
		if m.dryRun {
			icon = styles.Warning.Render("~")
			verb = "would sync to"
		}
		if errCount > 0 && synced == 0 {
			icon = styles.Error.Render("✗")
		}
		projectCount := fmt.Sprintf("%d project", synced)
		if synced != 1 {
			projectCount += "s"
		}
		summary := fmt.Sprintf("%s (%d files)", projectCount, fileCount)
		if errCount > 0 {
			summary += styles.Error.Render(fmt.Sprintf("  %d error(s)", errCount))
		}
		s += fmt.Sprintf("%s %-22s %s\n",
			icon,
			styles.SkillName.Render(skillName),
			styles.Muted.Render(verb+" "+summary),
		)
	}
	s += "\n" + m.help.ShortHelpView([]key.Binding{syncKeys.Quit}) + "\n"
	return s
}

func (m syncModel) helpView() string {
	if m.showHelp {
		return m.help.FullHelpView(syncKeys.FullHelp())
	}
	return m.help.ShortHelpView(syncKeys.ShortHelp())
}

// RunSync opens the interactive sync TUI.
func RunSync(vault, root string, dryRun bool) error {
	sp := spinner.New()
	sp.Spinner = spinner.Points
	sp.Style = styles.SkillName

	prog := progress.New(
		progress.WithGradient("#626262", "#87CEEB"),
		progress.WithoutPercentage(),
		progress.WithWidth(40),
	)

	pg := paginator.New()
	pg.Type = paginator.Dots
	pg.PerPage = pageSize
	pg.ActiveDot = styles.SkillName.Render("•")
	pg.InactiveDot = styles.Muted.Render("·")

	m := syncModel{
		vault:     vault,
		root:      root,
		dryRun:    dryRun,
		phase:     phaseLoading,
		spinner:   sp,
		progress:  prog,
		paginator: pg,
		help:      help.New(),
	}
	_, err := tea.NewProgram(m, tea.WithAltScreen(), tea.WithMouseAllMotion()).Run()
	return err
}
