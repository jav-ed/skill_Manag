package tui

import (
	"fmt"
	"os"
	"path/filepath"
	"strings"

	"github.com/charmbracelet/bubbles/filepicker"
	tea "github.com/charmbracelet/bubbletea"
	"skill_Manag/styles"
)

type setupPhase int

const (
	setupVault setupPhase = iota
	setupRoot
	setupSave
	setupDone
)

type setupModel struct {
	fp          filepicker.Model
	phase       setupPhase
	vault       string
	root        string
	save        bool
	errMsg      string
	linkHovered bool
	backHovered bool
}

func newSetupModel(existingVault, existingRoot string) setupModel {
	fp := filepicker.New()
	fp.DirAllowed = true
	fp.FileAllowed = false
	fp.ShowHidden = false

	startPath := existingVault
	if startPath == "" {
		startPath, _ = os.UserHomeDir()
	}
	fp.CurrentDirectory = startPath

	return setupModel{
		fp:    fp,
		phase: setupVault,
		vault: existingVault,
		root:  existingRoot,
	}
}

func (m setupModel) Init() tea.Cmd {
	return m.fp.Init()
}

func (m setupModel) Update(msg tea.Msg) (tea.Model, tea.Cmd) {
	switch msg := msg.(type) {
	case tea.MouseMsg:
		var openURL, goBack bool
		m.linkHovered, m.backHovered, openURL, goBack = handleHeaderMouse(msg, m.linkHovered, m.backHovered, true)
		if openURL {
			openBrowser("https://javedab.com")
		}
		if goBack {
			return m, tea.Quit
		}
		var cmd tea.Cmd
		m.fp, cmd = m.fp.Update(msg)
		return m, cmd

	case tea.KeyMsg:
		switch msg.String() {
		case "ctrl+c", "alt+left":
			return m, tea.Quit
		case "y", "Y":
			if m.phase == setupSave {
				m.save = true
				m.phase = setupDone
				return m, tea.Quit
			}
		case "n", "N":
			if m.phase == setupSave {
				m.save = false
				m.phase = setupDone
				return m, tea.Quit
			}
		}
	}

	if m.phase == setupSave || m.phase == setupDone {
		return m, nil
	}

	var cmd tea.Cmd
	m.fp, cmd = m.fp.Update(msg)

	if didSelect, path := m.fp.DidSelectFile(msg); didSelect {
		switch m.phase {
		case setupVault:
			m.vault = path
			m.phase = setupRoot
			root := m.root
			if root == "" {
				root, _ = os.UserHomeDir()
			}
			m.fp.CurrentDirectory = root
			return m, m.fp.Init()
		case setupRoot:
			m.root = path
			m.phase = setupSave
		}
	}

	return m, cmd
}

func (m setupModel) View() string {
	divider := styles.Muted.Render(strings.Repeat("─", 56))
	s := "\n"
	s += styles.AppHeader("setup", m.linkHovered, m.backHovered) + "\n"
	s += divider + "\n\n"

	switch m.phase {
	case setupVault:
		s += styles.SkillName.Render("  → Vault") + "\n"
		s += styles.Muted.Render("     Your skill collection — the one folder where you") + "\n"
		s += styles.Muted.Render("     write and maintain skills. Select a directory:") + "\n\n"
		s += m.fp.View() + "\n"
	case setupRoot:
		s += styles.Success.Render("  ✓ Vault  ") + styles.Muted.Render(m.vault) + "\n\n"
		s += styles.SkillName.Render("  → Root") + "\n"
		s += styles.Muted.Render("     The folder that contains all your projects.") + "\n"
		s += styles.Muted.Render("     skill_Manag walks it to find installed skills:") + "\n\n"
		s += m.fp.View() + "\n"
	case setupSave:
		s += styles.Success.Render("  ✓ Vault  ") + styles.Muted.Render(m.vault) + "\n"
		s += styles.Success.Render("  ✓ Root   ") + styles.Muted.Render(m.root) + "\n\n"
		s += divider + "\n"
		s += "  " + styles.Warning.Render("Save to ~/.config/skill_Manag/config.yaml?") +
			"  " + styles.Muted.Render("(y / n)") + "\n"
	}

	if m.errMsg != "" {
		s += "\n  " + styles.Error.Render("✗ "+m.errMsg) + "\n"
	}
	s += "\n" + styles.Muted.Render("  alt+←  ·  ctrl+c   back")
	return s
}

// RunSetup opens the interactive setup TUI and returns the configured vault and root.
func RunSetup(existingVault, existingRoot string) (vault, root string, err error) {
	m := newSetupModel(existingVault, existingRoot)
	final, err := tea.NewProgram(m, tea.WithOutput(os.Stderr), tea.WithAltScreen(), tea.WithMouseAllMotion()).Run()
	if err != nil {
		return "", "", err
	}
	result := final.(setupModel)
	vault = result.vault
	root = result.root
	if result.save {
		if saveErr := saveConfig(vault, root); saveErr != nil {
			fmt.Fprintf(os.Stderr, "warning: could not save config: %v\n", saveErr)
		}
	}
	return vault, root, nil
}

func saveConfig(vault, root string) error {
	home, err := os.UserHomeDir()
	if err != nil {
		return err
	}
	dir := filepath.Join(home, ".config", "skill_Manag")
	if err := os.MkdirAll(dir, 0755); err != nil {
		return err
	}
	content := fmt.Sprintf("vault: %s\nroot: %s\n", vault, root)
	return os.WriteFile(filepath.Join(dir, "config.yaml"), []byte(content), 0644)
}
