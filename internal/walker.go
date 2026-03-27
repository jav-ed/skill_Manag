package internal

import (
	"os"
	"path/filepath"
)

// Directories that are never worth descending into
var skipDirs = map[string]bool{
	// version control
	".git": true,

	// dependency trees
	"node_modules": true,
	"vendor":       true,

	// build output
	"dist":   true,
	"build":  true,
	"out":    true,
	"target": true,
	".next":  true,
	".nuxt":  true,

	// python
	".venv":         true,
	"__pycache__":   true,
	".tox":          true,
	".pytest_cache": true,

	// caches
	".cache":        true,
	".turbo":        true,
	".parcel-cache": true,
}

// Target is a skill directory inside a project that matches a master skill
type Target struct {
	ProjectPath string // path to the project (parent of .agents)
	SkillName   string // name of the skill folder
	SkillPath   string // full path to the skill dir inside the project
}

// ReadMasterSkills returns a map of skillName → full path for every dir in sourceDir
func ReadMasterSkills(sourceDir string) (map[string]string, error) {
	entries, err := os.ReadDir(sourceDir)
	if err != nil {
		return nil, err
	}

	skills := make(map[string]string)
	for _, e := range entries {
		if e.IsDir() {
			skills[e.Name()] = filepath.Join(sourceDir, e.Name())
		}
	}

	return skills, nil
}

// FindTargets walks root and returns every .agents/skills/<Name> dir
// whose name matches a key in masterSkills
func FindTargets(root string, masterSkills map[string]string) ([]Target, error) {
	var targets []Target

	err := filepath.WalkDir(root, func(path string, d os.DirEntry, err error) error {
		if err != nil {
			// Skip unreadable paths rather than aborting the whole walk
			return nil
		}

		// Skip noisy dirs early
		if d.IsDir() && skipDirs[d.Name()] {
			return filepath.SkipDir
		}

		// We only care about dirs named "skills" whose parent is ".agents"
		if !d.IsDir() || d.Name() != "skills" {
			return nil
		}
		if filepath.Base(filepath.Dir(path)) != ".agents" {
			return nil
		}

		// List children of this .agents/skills dir
		entries, err := os.ReadDir(path)
		if err != nil {
			return nil
		}

		for _, e := range entries {
			if !e.IsDir() {
				continue
			}

			// Only add if this skill exists in the master collection
			if _, ok := masterSkills[e.Name()]; ok {
				targets = append(targets, Target{
					ProjectPath: filepath.Dir(filepath.Dir(path)),
					SkillName:   e.Name(),
					SkillPath:   filepath.Join(path, e.Name()),
				})
			}
		}

		// No need to go deeper once we found a skills dir
		return filepath.SkipDir
	})

	return targets, err
}

// FindTargetsByName finds all projects that have a specific skill installed,
// regardless of whether it exists in the master collection
func FindTargetsByName(root, skillName string) ([]Target, error) {
	return FindTargets(root, map[string]string{skillName: ""})
}

// FindPushTargets finds all projects that have any skill installed,
// then creates a target for each push skill in those projects — bypassing the opt-in rule.
func FindPushTargets(root string, pushSkills map[string]string) ([]Target, error) {
	var targets []Target

	err := filepath.WalkDir(root, func(path string, d os.DirEntry, err error) error {
		if err != nil {
			return nil
		}

		if d.IsDir() && skipDirs[d.Name()] {
			return filepath.SkipDir
		}

		if !d.IsDir() || d.Name() != "skills" {
			return nil
		}
		if filepath.Base(filepath.Dir(path)) != ".agents" {
			return nil
		}

		// Project must have at least one skill already installed (opted in to skill_Manag)
		entries, err := os.ReadDir(path)
		if err != nil {
			return nil
		}
		hasSkill := false
		for _, e := range entries {
			if e.IsDir() {
				hasSkill = true
				break
			}
		}
		if !hasSkill {
			return filepath.SkipDir
		}

		projectPath := filepath.Dir(filepath.Dir(path))
		for skillName := range pushSkills {
			targets = append(targets, Target{
				ProjectPath: projectPath,
				SkillName:   skillName,
				SkillPath:   filepath.Join(path, skillName),
			})
		}

		return filepath.SkipDir
	})

	return targets, err
}

// FindAllSkillTargets returns every installed skill across all projects,
// with no filtering against a master collection — used by interactive delete
func FindAllSkillTargets(root string) ([]Target, error) {
	var targets []Target

	err := filepath.WalkDir(root, func(path string, d os.DirEntry, err error) error {
		if err != nil {
			return nil
		}

		if d.IsDir() && skipDirs[d.Name()] {
			return filepath.SkipDir
		}

		if !d.IsDir() || d.Name() != "skills" {
			return nil
		}
		if filepath.Base(filepath.Dir(path)) != ".agents" {
			return nil
		}

		entries, err := os.ReadDir(path)
		if err != nil {
			return nil
		}

		// Add every skill dir — no master filter
		for _, e := range entries {
			if e.IsDir() {
				targets = append(targets, Target{
					ProjectPath: filepath.Dir(filepath.Dir(path)),
					SkillName:   e.Name(),
					SkillPath:   filepath.Join(path, e.Name()),
				})
			}
		}

		return filepath.SkipDir
	})

	return targets, err
}
