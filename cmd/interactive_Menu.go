package cmd

import (
	"strings"

	"github.com/charmbracelet/bubbles/list"
	tea "github.com/charmbracelet/bubbletea"
	"github.com/charmbracelet/lipgloss"
	"skill_Manag/styles"
)

type menuItem struct {
	title  string
	desc   string
	detail string
}

func (i menuItem) Title() string       { return i.title }
func (i menuItem) Description() string { return i.desc }
func (i menuItem) FilterValue() string { return i.title }

type menuModel struct {
	list   list.Model
	width  int
	chosen int
}

func newMenuModel() menuModel {
	items := []list.Item{
		menuItem{
			title: "Sync",
			desc:  "Updates only skills already installed per project — vault never force-pushes",
			detail: "Walks every project under your root and updates skills they already " +
				"have installed — pulling the latest from your vault. The opt-in rule: " +
				"if a project doesn't have a skill, it will inshallah never be added. Only what's " +
				"already there gets refreshed.",
		},
		menuItem{
			title: "List",
			desc:  "Full table view — filter by name, sync or delete selected rows in place",
			detail: "A searchable table of every skill installed across all your projects. " +
				"Filter by name with /, select rows with space, then sync or delete the " +
				"selection directly without leaving the screen.",
		},
		menuItem{
			title: "Delete",
			desc:  "Nothing pre-selected — pick explicitly and confirm before anything is removed",
			detail: "Remove skills from projects. Nothing is pre-selected — you pick " +
				"explicitly, then confirm before anything is removed. Supports removing " +
				"from all matching projects at once.",
		},
		menuItem{
			title: "Setup",
			desc:  "Filesystem picker for vault and root — auto-runs on first launch",
			detail: "Configure your vault (the folder holding your master skill files) and " +
				"root (the folder containing all your projects) using a filesystem picker. " +
				"Settings are saved to ~/.config/skill_Manag/config.yaml.",
		},
	}

	delegate := list.NewDefaultDelegate()
	delegate.Styles.SelectedTitle = delegate.Styles.SelectedTitle.
		Foreground(lipgloss.Color("#87CEEB")).
		BorderLeftForeground(lipgloss.Color("#87CEEB"))
	delegate.Styles.SelectedDesc = delegate.Styles.SelectedDesc.
		Foreground(lipgloss.Color("#626262")).
		BorderLeftForeground(lipgloss.Color("#87CEEB"))
	delegate.Styles.NormalTitle = delegate.Styles.NormalTitle.
		Foreground(lipgloss.Color("#FFFFFF"))
	delegate.Styles.NormalDesc = delegate.Styles.NormalDesc.
		Foreground(lipgloss.Color("#626262"))

	l := list.New(items, delegate, 52, 12)
	l.SetShowTitle(false)
	l.SetShowStatusBar(false)
	l.SetFilteringEnabled(false)

	return menuModel{list: l, chosen: -1}
}

func (m menuModel) Init() tea.Cmd { return nil }

func (m menuModel) Update(msg tea.Msg) (tea.Model, tea.Cmd) {
	switch msg := msg.(type) {
	case tea.KeyMsg:
		switch msg.String() {
		case "ctrl+c", "q":
			return m, tea.Quit
		case "enter":
			m.chosen = m.list.Index()
			return m, tea.Quit
		}
	case tea.WindowSizeMsg:
		m.width = msg.Width
		m.list.SetWidth(msg.Width)
		m.list.SetHeight(msg.Height - 11) // header (4) + detail panel (5) + breathing room (2)
		return m, nil
	}

	var cmd tea.Cmd
	m.list, cmd = m.list.Update(msg)
	return m, cmd
}

func (m menuModel) View() string {
	pad := lipgloss.NewStyle().PaddingLeft(2)

	// Element 1 — tool name + website
	name := lipgloss.NewStyle().Bold(true).Render("skill_Manag")
	website := styles.Muted.Render("javedab.com")

	// Element 2 — tagline
	tagline := styles.Muted.Render("Sync Claude Code skills across all your projects from a single vault.")

	// Element 3 — menu list
	menu := m.list.View()

	// Element 4 — detail panel for the currently hovered item
	detail := m.detailView()

	header := pad.Render(name+"  ·  "+website) + "\n" +
		pad.Render(tagline)

	return "\n" + header + "\n\n" + menu + "\n" + detail
}

func (m menuModel) detailView() string {
	item, ok := m.list.SelectedItem().(menuItem)
	if !ok {
		return ""
	}

	w := m.width
	if w < 10 {
		w = 72 // fallback before first WindowSizeMsg
	}

	pad := lipgloss.NewStyle().PaddingLeft(2)
	divider := styles.Muted.Render(strings.Repeat("─", w-4))
	text := styles.Muted.Render(
		lipgloss.NewStyle().Width(w - 4).Render(item.detail),
	)

	return pad.Render(divider) + "\n" + pad.Render(text)
}

// showMenu runs the menu TUI and returns the chosen index (-1 = quit)
func showMenu() (int, error) {
	final, err := tea.NewProgram(newMenuModel(), tea.WithAltScreen()).Run()
	if err != nil {
		return -1, err
	}
	return final.(menuModel).chosen, nil
}
