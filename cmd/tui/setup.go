package tui

import (
	"fmt"
	"os"
	"path/filepath"
	"sort"
	"strings"

	"github.com/charmbracelet/bubbles/filepicker"
	tea "github.com/charmbracelet/bubbletea"
	"github.com/spf13/viper"
	"skill_Manag/internal"
	"skill_Manag/styles"
)

type setupPhase int

const (
	setupVault setupPhase = iota
	setupRoot
	setupMandatory
	setupSave
	setupDone
)

type setupModel struct {
	fp                filepicker.Model
	phase             setupPhase
	vault             string
	root              string
	vaultSkills       []string
	mandatorySelected map[string]bool
	mandatoryCursor   int
	save              bool
	errMsg            string
	linkHovered       bool
	backHovered       bool
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
		// Mandatory checklist — mouse support for items.
		// setupMandatory layout: row 0=empty, row 1=header, row 2=divider, row 3=empty,
		// row 4=vault, row 5=root, row 6=empty, row 7=→Mandatory, row 8=desc, row 9=empty → items at row 10.
		if m.phase == setupMandatory {
			const itemsStart = 10
			if msg.Y >= itemsStart {
				idx := msg.Y - itemsStart
				if idx >= 0 && idx < len(m.vaultSkills) {
					switch msg.Action {
					case tea.MouseActionMotion:
						m.mandatoryCursor = idx
					case tea.MouseActionPress:
						if msg.Button == tea.MouseButtonLeft {
							m.mandatoryCursor = idx
							name := m.vaultSkills[idx]
							m.mandatorySelected[name] = !m.mandatorySelected[name]
						}
					}
				}
			}
			return m, nil
		}
		if m.phase == setupSave || m.phase == setupDone {
			return m, nil
		}
		var cmd tea.Cmd
		m.fp, cmd = m.fp.Update(msg)
		return m, cmd

	case tea.KeyMsg:
		switch msg.String() {
		case "ctrl+c", "alt+left", "q":
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
		// Mandatory step — handle checklist navigation, don't pass to filepicker
		if m.phase == setupMandatory {
			switch msg.String() {
			case "up", "k":
				if m.mandatoryCursor > 0 {
					m.mandatoryCursor--
				}
			case "down", "j":
				if m.mandatoryCursor < len(m.vaultSkills)-1 {
					m.mandatoryCursor++
				}
			case " ":
				if len(m.vaultSkills) > 0 {
					name := m.vaultSkills[m.mandatoryCursor]
					m.mandatorySelected[name] = !m.mandatorySelected[name]
				}
			case "enter":
				m.phase = setupSave
			}
			return m, nil
		}
	}

	if m.phase == setupSave || m.phase == setupDone || m.phase == setupMandatory {
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
			m.vaultSkills = readVaultSkillNames(m.vault)
			m.mandatorySelected = make(map[string]bool)
			for _, name := range viper.GetStringSlice("mandatory") {
				m.mandatorySelected[name] = true
			}
			m.phase = setupMandatory
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
	case setupMandatory:
		s += styles.Success.Render("  ✓ Vault  ") + styles.Muted.Render(m.vault) + "\n"
		s += styles.Success.Render("  ✓ Root   ") + styles.Muted.Render(m.root) + "\n\n"
		s += styles.SkillName.Render("  → Mandatory") + "\n"
		s += styles.Muted.Render("     Skills pushed to every opted-in project:") + "\n\n"
		if len(m.vaultSkills) == 0 {
			s += styles.Muted.Render("  (no skills found in vault)") + "\n"
		} else {
			for i, name := range m.vaultSkills {
				cursor := "  "
				if i == m.mandatoryCursor {
					cursor = styles.Warning.Render("> ")
				}
				checkbox := "[ ]"
				if m.mandatorySelected[name] {
					checkbox = styles.Warning.Render("[✓]")
				}
				s += fmt.Sprintf("  %s%s %s\n", cursor, checkbox, styles.SkillName.Render(name))
			}
		}
		s += "\n" + styles.Muted.Render("  ↑/↓ navigate  space toggle  enter confirm") + "\n"
	case setupSave:
		mandatory := mandatoryList(m.vaultSkills, m.mandatorySelected)
		mandatoryStr := "(none)"
		if len(mandatory) > 0 {
			mandatoryStr = strings.Join(mandatory, ", ")
		}
		s += styles.Success.Render("  ✓ Vault      ") + styles.Muted.Render(m.vault) + "\n"
		s += styles.Success.Render("  ✓ Root       ") + styles.Muted.Render(m.root) + "\n"
		s += styles.Success.Render("  ✓ Mandatory  ") + styles.Muted.Render(mandatoryStr) + "\n\n"
		s += divider + "\n"
		s += "  " + styles.Warning.Render("Save config?") + "  " +
			styles.Muted.Render("vault pointer → ~/.config/skill_Manag/vault   root → <vault>/config.yaml   (y / n)") + "\n"
	}

	if m.errMsg != "" {
		s += "\n  " + styles.Error.Render("✗ "+m.errMsg) + "\n"
	}
	s += "\n" + styles.Muted.Render("  q  ·  alt+←  ·  ctrl+c   back")
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
		mandatory := mandatoryList(result.vaultSkills, result.mandatorySelected)
		if saveErr := saveConfig(vault, root, mandatory); saveErr != nil {
			fmt.Fprintf(os.Stderr, "warning: could not save config: %v\n", saveErr)
		}
	}
	return vault, root, nil
}

func saveConfig(vault, root string, mandatory []string) error {
	home, err := os.UserHomeDir()
	if err != nil {
		return err
	}
	// Write vault path pointer
	dir := filepath.Join(home, ".config", "skill_Manag")
	if err := os.MkdirAll(dir, 0755); err != nil {
		return err
	}
	if err := os.WriteFile(filepath.Join(dir, "vault"), []byte(vault+"\n"), 0644); err != nil {
		return err
	}
	// Write vault config — preserve any existing keys, then set root and mandatory
	vaultConfigPath := filepath.Join(vault, "config.yaml")
	v := viper.New()
	v.SetConfigFile(vaultConfigPath)
	v.ReadInConfig() // ignore error — file may not exist yet
	v.Set("root", root)
	v.Set("mandatory", mandatory)
	return v.WriteConfigAs(vaultConfigPath)
}

// SaveVaultMandatory updates only the mandatory list in the vault config,
// preserving all other keys. Used by the Push screen's edit mode.
func SaveVaultMandatory(vault string, mandatory []string) error {
	vaultConfigPath := filepath.Join(vault, "config.yaml")
	v := viper.New()
	v.SetConfigFile(vaultConfigPath)
	v.ReadInConfig()
	v.Set("mandatory", mandatory)
	return v.WriteConfigAs(vaultConfigPath)
}

// readVaultSkillNames returns all skill names from the vault, sorted.
func readVaultSkillNames(vault string) []string {
	master, err := internal.ReadMasterSkills(vault)
	if err != nil {
		return nil
	}
	names := make([]string, 0, len(master))
	for name := range master {
		names = append(names, name)
	}
	sort.Strings(names)
	return names
}

// mandatoryList returns the selected mandatory skills in vault order.
func mandatoryList(vaultSkills []string, selected map[string]bool) []string {
	result := make([]string, 0)
	for _, name := range vaultSkills {
		if selected[name] {
			result = append(result, name)
		}
	}
	return result
}
