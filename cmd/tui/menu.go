package tui

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
	list        list.Model
	width       int
	chosen      int
	linkHovered bool
	backHovered bool
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
			title: "Push",
			desc:  "Force-installs mandatory skills to every opted-in project",
			detail: "Reads the mandatory list from your vault config and pushes those skills " +
				"to every project that already has .agents/skills/ — bypassing the opt-in rule. " +
				"Configure mandatory skills by adding 'mandatory: [skill-name]' to <vault>/config.yaml.",
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
		m.list.SetHeight(msg.Height - 11)
		return m, nil
	case tea.MouseMsg:
		var openURL bool
		m.linkHovered, m.backHovered, openURL, _ = handleHeaderMouse(msg, m.linkHovered, m.backHovered, false)
		if openURL {
			openBrowser("https://javedab.com")
		}
		var cmd tea.Cmd
		m.list, cmd = m.list.Update(msg)
		// The bubbles list has no mouse handling — we drive it ourselves.
		// View layout: "\n" + AppHeader + "\n" + tagline + "\n\n" → list starts at row 4.
		// DefaultDelegate: Height=2 (title+desc) + Spacing=1 → 3 terminal rows per item.
		// On every motion: call Select() so the visual cursor follows the mouse.
		// On press: confirm whatever Select() pointed at.
		const listTop = 4
		const rowsPerItem = 3
		if msg.Y >= listTop {
			idx := (msg.Y - listTop) / rowsPerItem
			if idx >= 0 && idx < len(m.list.Items()) {
				m.list.Select(idx)
				if msg.Action == tea.MouseActionPress && msg.Button == tea.MouseButtonLeft {
					m.chosen = idx
					return m, tea.Quit
				}
			}
		}
		return m, cmd
	}

	var cmd tea.Cmd
	m.list, cmd = m.list.Update(msg)
	return m, cmd
}

func (m menuModel) View() string {
	pad := lipgloss.NewStyle().PaddingLeft(2)
	tagline := styles.Muted.Render("Sync Claude Code skills across all your projects from a single vault.")
	header := styles.AppHeader("", m.linkHovered, false) + "\n" + pad.Render(tagline)
	return "\n" + header + "\n\n" + m.list.View() + "\n" + m.detailView()
}

func (m menuModel) detailView() string {
	item, ok := m.list.SelectedItem().(menuItem)
	if !ok {
		return ""
	}
	w := m.width
	if w < 10 {
		w = 72
	}
	pad := lipgloss.NewStyle().PaddingLeft(2)
	divider := styles.Muted.Render(strings.Repeat("─", w-4))
	text := styles.Muted.Render(lipgloss.NewStyle().Width(w - 4).Render(item.detail))
	return pad.Render(divider) + "\n" + pad.Render(text)
}

// ShowMenu runs the menu TUI and returns the chosen index (-1 = quit).
func ShowMenu() (int, error) {
	final, err := tea.NewProgram(newMenuModel(), tea.WithAltScreen(), tea.WithMouseAllMotion()).Run()
	if err != nil {
		return -1, err
	}
	return final.(menuModel).chosen, nil
}
