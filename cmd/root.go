package cmd

import (
	"os"
	"path/filepath"

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

Config file (optional): ~/.config/skill_Manag/config.yaml
  vault: /path/to/your/skill/vault
  root:  /path/to/projects

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

// initConfig loads the optional config file before any command runs
func initConfig() {
	home, err := os.UserHomeDir()
	if err != nil {
		return
	}

	viper.SetConfigName("config")
	viper.SetConfigType("yaml")
	viper.AddConfigPath(filepath.Join(home, ".config", "skill_Manag"))

	// Silently ignore a missing config file — flags are the fallback
	viper.ReadInConfig()

	// Env vars: SKILL_MANAG_VAULT, SKILL_MANAG_ROOT
	viper.SetEnvPrefix("SKILL_MANAG")
	viper.AutomaticEnv()
}
