package tui

import (
	"fmt"
	"strings"

	"github.com/charmbracelet/bubbles/key"
	"skill_Manag/internal"
	"skill_Manag/styles"
)

func (m syncModel) selectView() string {
	selected := 0
	for _, item := range m.items {
		if item.selected {
			selected++
		}
	}

	divider := styles.Muted.Render(strings.Repeat("─", 52))
	colHdr := "  " + styles.Muted.Render("[·]") + " " +
		styles.Muted.Render(fmt.Sprintf("%-22s  %s", "skill", "projects"))

	s := styles.Header.Render("Select skills to sync") + "   " +
		styles.Muted.Render(fmt.Sprintf("%d / %d selected", selected, len(m.items))) + "\n"
	s += colHdr + "\n"
	s += "  " + divider + "\n"

	start, end := m.paginator.GetSliceBounds(len(m.items))
	for i, item := range m.items[start:end] {
		cursor := "  "
		if i == m.cursor {
			cursor = styles.Success.Render("> ")
		}
		checkbox := "[ ]"
		if item.selected {
			checkbox = styles.Success.Render("[✓]")
		}
		projectCount := fmt.Sprintf("%d project", len(item.targets))
		if len(item.targets) != 1 {
			projectCount += "s"
		}
		s += fmt.Sprintf("%s%s %-22s  %s\n",
			cursor, checkbox,
			styles.SkillName.Render(item.skillName),
			styles.Muted.Render(projectCount),
		)
	}

	s += "  " + divider + "\n"
	if m.paginator.TotalPages > 1 {
		s += "\n" + styles.Muted.Render(m.paginator.View()) + "\n"
	}
	s += "\n" + m.helpView()
	return s
}

func (m syncModel) resultsView() string {
	if m.err != nil {
		return styles.Error.Render("✗ "+m.err.Error()) + "\n\n" +
			m.help.ShortHelpView([]key.Binding{syncKeys.Quit}) + "\n"
	}
	if len(m.results) == 0 {
		return styles.Muted.Render("No matching skills found in any project.") + "\n\n" +
			m.help.ShortHelpView([]key.Binding{syncKeys.Quit}) + "\n"
	}

	s := styles.Header.Render("Results") + "\n\n"
	for _, skillName := range syncResultSkillOrder(m.results) {
		s += m.skillResultView(skillName, syncResultsForSkill(m.results, skillName))
	}
	s += "\n" + m.help.ShortHelpView([]key.Binding{syncKeys.Quit}) + "\n"
	return s
}

func (m syncModel) skillResultView(skillName string, results []internal.SyncResult) string {
	errCount, fileCount := 0, 0
	for _, r := range results {
		if r.Err != nil {
			errCount++
		} else {
			fileCount += len(r.Files)
		}
	}

	synced := len(results) - errCount
	icon := styles.Success.Render("✓")
	verb := "synced to"
	if m.dryRun {
		icon = styles.Warning.Render("~")
		verb = "would sync to"
	}
	if errCount > 0 && synced == 0 {
		icon = styles.Error.Render("✗")
	}

	projectCount := fmt.Sprintf("%d project", synced)
	if synced != 1 {
		projectCount += "s"
	}
	summary := fmt.Sprintf("%s (%d files)", projectCount, fileCount)
	if errCount > 0 {
		summary += styles.Error.Render(fmt.Sprintf("  %d error(s)", errCount))
	}

	s := fmt.Sprintf("%s %-22s %s\n",
		icon,
		styles.SkillName.Render(skillName),
		styles.Muted.Render(verb+" "+summary),
	)
	for _, result := range results {
		s += syncResultProjectLine(result, m.dryRun)
	}
	return s
}

func syncResultSkillOrder(results []internal.SyncResult) []string {
	seen := make(map[string]bool)
	order := []string{}
	for _, result := range results {
		name := result.Target.SkillName
		if seen[name] {
			continue
		}
		seen[name] = true
		order = append(order, name)
	}
	return order
}

func syncResultsForSkill(results []internal.SyncResult, skillName string) []internal.SyncResult {
	filtered := []internal.SyncResult{}
	for _, result := range results {
		if result.Target.SkillName == skillName {
			filtered = append(filtered, result)
		}
	}
	return filtered
}

func syncResultProjectLine(result internal.SyncResult, dryRun bool) string {
	if result.Err != nil {
		return fmt.Sprintf("  %s %s  %s\n",
			styles.Error.Render("✗"),
			styles.Muted.Render(result.Target.ProjectPath),
			styles.Error.Render(result.Err.Error()),
		)
	}

	marker := styles.Success.Render("✓")
	action := "updated"
	if dryRun {
		marker = styles.Warning.Render("~")
		action = "would update"
	}
	return fmt.Sprintf("  %s %s  %s\n",
		marker,
		styles.Muted.Render(action),
		styles.Muted.Render(result.Target.ProjectPath),
	)
}

func (m syncModel) helpView() string {
	if m.showHelp {
		return m.help.FullHelpView(syncKeys.FullHelp())
	}
	return m.help.ShortHelpView(syncKeys.ShortHelp())
}
