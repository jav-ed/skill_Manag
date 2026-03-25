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

// runMenu is the root cobra handler. It shows the main menu and dispatches
// to the appropriate flow. --dry-run bypasses the menu for scripting use.
func runMenu(cmd *cobra.Command, args []string) error {
	vault := viper.GetString("vault")
	root := viper.GetString("root")

	// First-time setup if config is missing
	if vault == "" || root == "" {
		var err error
		vault, root, err = runSetup(vault, root)
		if err != nil {
			return err
		}
		if vault == "" || root == "" {
			return fmt.Errorf("vault and root are required")
		}
	}

	// --dry-run skips the menu and goes straight to a sync preview
	if dryRun {
		return doSync(vault, root, true)
	}

	// Main menu loop — Setup returns here with updated paths
	for {
		choice, err := showMenu()
		if err != nil || choice == -1 {
			return err
		}

		switch choice {
		case 0: // Sync
			if err := doSync(vault, root, false); err != nil {
				return err
			}
		case 1: // List
			if err := doList(vault, root); err != nil {
				return err
			}
		case 2: // Delete
			if err := doDeleteInteractive(root); err != nil {
				return err
			}
		case 3: // Setup
			vault, root, err = runSetup(vault, root)
			if err != nil {
				return err
			}
		}
	}
}

// doSync reads the vault, finds targets, and either runs the TUI or dry-run output
func doSync(vault, root string, dryRun bool) error {
	if !dryRun {
		return runInteractive(vault, root, false)
	}

	// --dry-run: scan synchronously and print results without a TUI
	masterSkills, err := internal.ReadMasterSkills(vault)
	if err != nil {
		return fmt.Errorf("reading vault: %w", err)
	}
	if len(masterSkills) == 0 {
		fmt.Println(styles.Warning.Render("No skills found in vault."))
		return nil
	}
	targets, err := internal.FindTargets(root, masterSkills)
	if err != nil {
		return fmt.Errorf("scanning projects: %w", err)
	}
	if len(targets) == 0 {
		fmt.Println(styles.Muted.Render("No matching skills found in any project."))
		return nil
	}
	return syncAll(targets, masterSkills, true)
}

// syncAll syncs every target and prints a per-project summary (used by --dry-run)
func syncAll(targets []internal.Target, masterSkills map[string]string, dryRun bool) error {
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
