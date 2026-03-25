package cmd

import (
	"fmt"

	"github.com/spf13/cobra"
	"github.com/spf13/viper"
	"skill_Manag/internal"
	"skill_Manag/styles"
)

var listCmd = &cobra.Command{
	Use:   "list",
	Short: "Browse all installed skills across projects",
	Long: `Opens an interactive TUI showing every skill installed across all projects.

Filter by skill name, select items, then sync or delete directly from the view.

  /       enter filter mode
  space   toggle selection
  a       select all visible
  s       sync selected from master
  d       delete selected
  esc     clear filter
  q       quit`,
	Example: `  skill_Manag list
  skill_Manag list --root /path/to/projects`,
	RunE: runList,
}

func init() {
	rootCmd.AddCommand(listCmd)
}

func runList(cmd *cobra.Command, args []string) error {
	root := viper.GetString("root")
	if root == "" {
		return fmt.Errorf("scan root is required: use --root or set 'root' in ~/.config/skill_Manag/config.yaml")
	}

	fmt.Printf("\n%s\n", styles.Muted.Render("Scanning "+root+"..."))

	// Master skills are optional — only needed if user triggers sync from the TUI
	var masterSkills map[string]string
	if source := viper.GetString("source"); source != "" {
		var err error
		masterSkills, err = internal.ReadMasterSkills(source)
		if err != nil {
			return fmt.Errorf("reading master skills: %w", err)
		}
	}

	// Collect every installed skill across all projects
	targets, err := internal.FindAllSkillTargets(root)
	if err != nil {
		return fmt.Errorf("scanning projects: %w", err)
	}
	if len(targets) == 0 {
		fmt.Println(styles.Muted.Render("No skills found in any project."))
		return nil
	}

	return runInteractiveList(targets, masterSkills)
}
