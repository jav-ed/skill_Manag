package cmd

import (
	"fmt"
	"os"
	"path/filepath"

	"github.com/spf13/cobra"
	"github.com/spf13/viper"
	"skill_Manag/internal"
	"skill_Manag/styles"
)

var (
	deleteProject string
	deleteDryRun  bool
)

var deleteCmd = &cobra.Command{
	Use:   "delete [skill-name]",
	Short: "Remove a skill from one or all projects",
	Long: `Removes a skill directory from projects. Three modes:

  Interactive (no args)
    skill_Manag delete
    Opens a TUI listing every installed skill across all projects.
    Nothing is pre-selected — opt in explicitly, then confirm.

  By name — all projects
    skill_Manag delete <skill-name>
    Finds every project that has this skill and removes it from all of them.

  By name + project — targeted
    skill_Manag delete <skill-name> --project /path/to/project
    Removes the skill from one specific project only.

Add --dry-run to any mode to preview what would be removed without deleting.

Flags:
  --dry-run    Preview what would be deleted without removing anything.
  --project    Limit deletion to one specific project path.
  --root       Override the scan root for this run.`,
	Example: `  skill_Manag delete
  skill_Manag delete coding
  skill_Manag delete coding --project /path/to/project
  skill_Manag delete coding --dry-run`,
	Args: cobra.MaximumNArgs(1),
	RunE: runDelete,
}

func init() {
	rootCmd.AddCommand(deleteCmd)
	deleteCmd.Flags().StringVar(&deleteProject, "project", "", "Limit deletion to this project path")
	deleteCmd.Flags().BoolVar(&deleteDryRun, "dry-run", false, "Preview what would be deleted without removing anything")
}

func doDeleteInteractive(root string) error {
	if root == "" {
		return fmt.Errorf("scan root is required: use --root or set 'root' in ~/.config/skill_Manag/config.yaml")
	}
	return runInteractiveDelete(root, deleteDryRun)
}

func runDelete(cmd *cobra.Command, args []string) error {
	if len(args) == 0 {
		return doDeleteInteractive(viper.GetString("root"))
	}

	skillName := args[0]

	// --project: delete from one specific project
	if deleteProject != "" {
		target := internal.Target{
			ProjectPath: deleteProject,
			SkillName:   skillName,
			SkillPath:   filepath.Join(deleteProject, ".agents", "skills", skillName),
		}
		result := internal.DeleteSkill(target, deleteDryRun)
		printDeleteResult(result, deleteDryRun)
		return result.Err
	}

	// Skill name only: delete from all projects that have it
	root := viper.GetString("root")
	if root == "" {
		return fmt.Errorf("scan root is required: use --root or set 'root' in ~/.config/skill_Manag/config.yaml")
	}

	targets, err := internal.FindTargetsByName(root, skillName)
	if err != nil {
		return fmt.Errorf("scanning projects: %w", err)
	}
	if len(targets) == 0 {
		fmt.Printf("\n%s\n", styles.Muted.Render(skillName+" not found in any project."))
		return nil
	}

	for _, target := range targets {
		result := internal.DeleteSkill(target, deleteDryRun)
		printDeleteResult(result, deleteDryRun)
	}

	fmt.Println()
	summary := fmt.Sprintf("%d project(s) updated.", len(targets))
	if deleteDryRun {
		summary = fmt.Sprintf("%d project(s) would be updated. (dry run)", len(targets))
	}
	fmt.Println(styles.Success.Render(summary))

	return nil
}

// printDeleteResult prints a single delete outcome line
func printDeleteResult(result internal.DeleteResult, dryRun bool) {
	if result.Err != nil {
		fmt.Fprintf(os.Stderr, "  %s %-30s %s\n",
			styles.Error.Render("✗"),
			styles.SkillName.Render(result.Target.SkillName),
			styles.Error.Render(result.Err.Error()),
		)
		return
	}

	icon := styles.Error.Render("✗")
	verb := "deleted from"
	if dryRun {
		icon = styles.Warning.Render("~")
		verb = "would delete from"
	}

	fmt.Printf("  %s %-30s %s\n",
		icon,
		styles.SkillName.Render(result.Target.SkillName),
		styles.Muted.Render(verb+" "+result.Target.ProjectPath),
	)
}
