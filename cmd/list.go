package cmd

import (
	"fmt"

	"github.com/spf13/cobra"
	"github.com/spf13/viper"
	"skill_Manag/cmd/tui"
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
	return doList(viper.GetString("vault"), viper.GetString("root"))
}

func doList(vault, root string) error {
	if root == "" {
		return fmt.Errorf("scan root is required: use --root or configure via Setup")
	}
	return tui.RunList(vault, root)
}
