package cmd

import (
	"os"
	"path/filepath"
	"strings"

	"github.com/spf13/cobra"
	"github.com/spf13/viper"
)

var rootCmd = &cobra.Command{
	Use:   "skill_Manag",
	Short: "Sync Claude Code skills across all your projects",
	Long: `skill_Manag keeps Claude Code skill files in sync across all your projects.

Skills are stored in .agents/skills/<SkillName>/ inside each project.
Maintaining them manually across many projects is error-prone — change one
skill and every other project is instantly out of date.

skill_Manag solves this by reading from a single Vault (your skill collection)
and propagating changes to every project that has a matching skill installed.
Files are copied (not symlinked), so they stay git-tracked and work over SSH.

Config: vault path stored in ~/.config/skill_Manag/vault
        root and mandatory stored in <vault>/config.yaml

Running without arguments opens the interactive TUI.
Pass --dry-run for a non-interactive preview of all changes.

Flags --vault and --root override the config file for one-off runs.`,
	Example: `  skill_Manag
  skill_Manag --dry-run
  skill_Manag --vault /path/to/skills --root /path/to/projects`,
	RunE: runMenu,
}

func Execute() {
	if err := rootCmd.Execute(); err != nil {
		os.Exit(1)
	}
}

func init() {
	cobra.OnInitialize(initConfig)

	// Persistent flags available to all subcommands
	rootCmd.PersistentFlags().String("vault", "", "Your skill vault directory (overrides config)")
	rootCmd.PersistentFlags().String("root", "", "Root directory to scan for projects (overrides config)")

	// --dry-run lives on root since sync is the root action
	rootCmd.Flags().BoolVar(&dryRun, "dry-run", false, "Non-interactive — preview all changes without applying them")

	viper.BindPFlag("vault", rootCmd.PersistentFlags().Lookup("vault"))
	viper.BindPFlag("root", rootCmd.PersistentFlags().Lookup("root"))
}

// initConfig loads config before any command runs.
// Vault path comes from: --vault flag > SKILL_MANAG_VAULT env > ~/.config/skill_Manag/vault pointer file.
// Root and mandatory come from: --root flag > SKILL_MANAG_ROOT env > <vault>/config.yaml.
func initConfig() {
	// Env vars: SKILL_MANAG_VAULT, SKILL_MANAG_ROOT
	viper.SetEnvPrefix("SKILL_MANAG")
	viper.AutomaticEnv()

	// Resolve vault path — flag/env take precedence over pointer file
	vault := viper.GetString("vault")
	if vault == "" {
		home, err := os.UserHomeDir()
		if err == nil {
			data, err := os.ReadFile(filepath.Join(home, ".config", "skill_Manag", "vault"))
			if err == nil {
				vault = strings.TrimSpace(string(data))
				viper.Set("vault", vault)
			}
		}
	}

	// Read vault config (root, mandatory) if vault path is known
	if vault != "" {
		viper.SetConfigFile(filepath.Join(vault, "config.yaml"))
		viper.ReadInConfig() // silently ignore missing file
	}
}
