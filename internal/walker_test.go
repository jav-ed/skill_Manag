package internal

import (
	"path/filepath"
	"testing"
)

func TestFindTargets_RespectsOptIn(t *testing.T) {
	vault := copyFixtures(t, fixtureVault)
	projects := copyFixtures(t, fixtureProjects)

	masterSkills, err := ReadMasterSkills(vault)
	if err != nil {
		t.Fatalf("ReadMasterSkills: %v", err)
	}

	targets, err := FindTargets(projects, masterSkills)
	if err != nil {
		t.Fatalf("FindTargets: %v", err)
	}

	// project-a has coding + doc-start, NOT tmux or refac-cli
	projectA := filepath.Join(projects, "org/team/project-a")
	var projectASkills []string
	for _, tgt := range targets {
		if tgt.ProjectPath == projectA {
			projectASkills = append(projectASkills, tgt.SkillName)
		}
	}

	if len(projectASkills) != 2 {
		t.Errorf("project-a: expected 2 skills, got %d: %v", len(projectASkills), projectASkills)
	}
	for _, want := range []string{"coding", "doc-start"} {
		found := false
		for _, s := range projectASkills {
			if s == want {
				found = true
				break
			}
		}
		if !found {
			t.Errorf("project-a: expected skill %q not found in %v", want, projectASkills)
		}
	}
	for _, unwanted := range []string{"tmux", "refac-cli"} {
		for _, s := range projectASkills {
			if s == unwanted {
				t.Errorf("project-a: skill %q should not be a target (not opted in)", unwanted)
			}
		}
	}
}

func TestFindTargets_SkipsNoiseDirs(t *testing.T) {
	vault := copyFixtures(t, fixtureVault)
	projects := copyFixtures(t, fixtureProjects)

	masterSkills, err := ReadMasterSkills(vault)
	if err != nil {
		t.Fatalf("ReadMasterSkills: %v", err)
	}

	targets, err := FindTargets(projects, masterSkills)
	if err != nil {
		t.Fatalf("FindTargets: %v", err)
	}

	noiseSegments := []string{"node_modules", "vendor", ".git", ".venv", "__pycache__", ".cache"}
	for _, tgt := range targets {
		for _, seg := range noiseSegments {
			if filepath.IsAbs(tgt.SkillPath) {
				rel, _ := filepath.Rel(projects, tgt.SkillPath)
				for _, part := range filepath.SplitList(rel) {
					if part == seg {
						t.Errorf("target inside noise dir %q: %s", seg, tgt.SkillPath)
					}
				}
			}
			// Check each path component
			path := tgt.SkillPath
			for path != filepath.Dir(path) {
				if filepath.Base(path) == seg {
					t.Errorf("target path passes through noise dir %q: %s", seg, tgt.SkillPath)
				}
				path = filepath.Dir(path)
			}
		}
	}
}

func TestFindTargets_FindsDeepProject(t *testing.T) {
	vault := copyFixtures(t, fixtureVault)
	projects := copyFixtures(t, fixtureProjects)

	masterSkills, err := ReadMasterSkills(vault)
	if err != nil {
		t.Fatalf("ReadMasterSkills: %v", err)
	}

	targets, err := FindTargets(projects, masterSkills)
	if err != nil {
		t.Fatalf("FindTargets: %v", err)
	}

	// project-d is at standalone/deep/nested/project-d — 5 levels deep
	projectD := filepath.Join(projects, "standalone/deep/nested/project-d")
	var found []string
	for _, tgt := range targets {
		if tgt.ProjectPath == projectD {
			found = append(found, tgt.SkillName)
		}
	}

	if len(found) == 0 {
		t.Fatal("project-d (5 levels deep) was not found by FindTargets")
	}
	for _, want := range []string{"doc-start", "astro"} {
		ok := false
		for _, s := range found {
			if s == want {
				ok = true
				break
			}
		}
		if !ok {
			t.Errorf("project-d: expected skill %q, got %v", want, found)
		}
	}
}

func TestFindTargets_IgnoresNoSkillsProject(t *testing.T) {
	vault := copyFixtures(t, fixtureVault)
	projects := copyFixtures(t, fixtureProjects)

	masterSkills, err := ReadMasterSkills(vault)
	if err != nil {
		t.Fatalf("ReadMasterSkills: %v", err)
	}

	targets, err := FindTargets(projects, masterSkills)
	if err != nil {
		t.Fatalf("FindTargets: %v", err)
	}

	noSkills := filepath.Join(projects, "no-skills")
	for _, tgt := range targets {
		if tgt.ProjectPath == noSkills {
			t.Errorf("no-skills project should produce no targets, got skill %q", tgt.SkillName)
		}
	}
}
