package cmd

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
	fp      filepicker.Model
	phase   setupPhase
	vault   string
	root    string
	save    bool
	errMsg  string
}

func newSetupModel(existingVault, existingRoot string) setupModel {
	fp := filepicker.New()
	fp.DirAllowed = true
	fp.FileAllowed = false
	fp.ShowHidden = false

	// Start from existing path or home
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
	case tea.KeyMsg:
		switch msg.String() {
		case "ctrl+c":
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
			// Reset filepicker for root selection
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
	s += styles.Header.Render("  skill_Manag") + styles.Muted.Render("  ·  javedab.com") + "  " + styles.Muted.Render("setup") + "\n"
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

	return s
}

func runSetup(existingVault, existingRoot string) (vault, root string, err error) {
	m := newSetupModel(existingVault, existingRoot)
	final, err := tea.NewProgram(m, tea.WithOutput(os.Stderr), tea.WithAltScreen()).Run()
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
