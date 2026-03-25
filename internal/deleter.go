package internal

import "os"

// DeleteResult holds the outcome of deleting one skill from one project
type DeleteResult struct {
	Target Target
	Err    error
}

// DeleteSkill removes the skill directory from the project.
// When dryRun is true it reports what would be deleted without touching anything.
func DeleteSkill(target Target, dryRun bool) DeleteResult {
	result := DeleteResult{Target: target}

	if !dryRun {
		result.Err = os.RemoveAll(target.SkillPath)
	}

	return result
}
