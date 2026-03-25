package cmd

import (
	"fmt"
	"os"

	"github.com/spf13/cobra"
	"github.com/spf13/viper"
	"skill_Manag/internal"
	"skill_Manag/styles"
)

var dryRun bool

func runSync(cmd *cobra.Command, args []string) error {
	source := viper.GetString("source")
	root := viper.GetString("root")

	if source == "" {
		return fmt.Errorf("master skills path is required: use --source or set 'source' in ~/.config/skill_Manag/config.yaml")
	}
	if root == "" {
		return fmt.Errorf("scan root is required: use --root or set 'root' in ~/.config/skill_Manag/config.yaml")
	}

	// Read available skills from master collection
	masterSkills, err := internal.ReadMasterSkills(source)
	if err != nil {
		return fmt.Errorf("reading master skills: %w", err)
	}
	if len(masterSkills) == 0 {
		fmt.Println(styles.Warning.Render("No skills found in master directory."))
		return nil
	}

	fmt.Printf("\n%s\n", styles.Muted.Render("Scanning "+root+"..."))

	// Walk all projects and collect matching skill dirs
	targets, err := internal.FindTargets(root, masterSkills)
	if err != nil {
		return fmt.Errorf("scanning projects: %w", err)
	}
	if len(targets) == 0 {
		fmt.Println(styles.Muted.Render("No matching skills found in any project."))
		return nil
	}

	// --dry-run bypasses the TUI for scripting use
	if dryRun {
		return syncAll(targets, masterSkills, true)
	}

	// Default: interactive TUI
	return runInteractive(targets, masterSkills, false)
}

// syncAll syncs every target and prints a per-project summary
func syncAll(targets []internal.Target, masterSkills map[string]string, dryRun bool) error {
	// Group targets by project for a cleaner output
	byProject := make(map[string][]internal.Target)
	for _, t := range targets {
		byProject[t.ProjectPath] = append(byProject[t.ProjectPath], t)
	}

	totalSynced := 0
	totalErrors := 0

	for project, projectTargets := range byProject {
		fmt.Printf("\n%s\n", styles.Header.Render("● "+project))

		for _, target := range projectTargets {
			result := internal.SyncSkill(masterSkills, target, dryRun)

			if result.Err != nil {
				fmt.Fprintf(os.Stderr, "  %s %s  %s\n",
					styles.Error.Render("✗"),
					styles.SkillName.Render(target.SkillName),
					styles.Error.Render(result.Err.Error()),
				)
				totalErrors++
				continue
			}

			icon := styles.Success.Render("✓")
			verb := "synced"
			if dryRun {
				icon = styles.Warning.Render("~")
				verb = "would sync"
			}

			fmt.Printf("  %s %-30s %s\n",
				icon,
				styles.SkillName.Render(target.SkillName),
				styles.Muted.Render(fmt.Sprintf("%s (%d files)", verb, len(result.Files))),
			)
			totalSynced++
		}
	}

	fmt.Println()
	summary := fmt.Sprintf("%d skills synced across %d projects.", totalSynced, len(byProject))
	if dryRun {
		summary = fmt.Sprintf("%d skills would be synced across %d projects. (dry run)", totalSynced, len(byProject))
	}
	if totalErrors > 0 {
		summary += fmt.Sprintf(" %d error(s).", totalErrors)
		fmt.Println(styles.Warning.Render(summary))
	} else {
		fmt.Println(styles.Success.Render(summary))
	}

	return nil
}
