package internal

import (
	"os"
	"path/filepath"
	"testing"
)

func TestSyncSkill_CopiesVaultFiles(t *testing.T) {
	vault := copyFixtures(t, fixtureVault)
	projects := copyFixtures(t, fixtureProjects)

	masterSkills, err := ReadMasterSkills(vault)
	if err != nil {
		t.Fatalf("ReadMasterSkills: %v", err)
	}

	skillPath := filepath.Join(projects, "org/team/project-a/.agents/skills/coding")
	target := Target{
		ProjectPath: filepath.Join(projects, "org/team/project-a"),
		SkillName:   "coding",
		SkillPath:   skillPath,
	}

	result := SyncSkill(masterSkills, target, false)
	if result.Err != nil {
		t.Fatalf("SyncSkill: %v", result.Err)
	}

	// Every file from the vault skill must exist in the destination
	for _, rel := range result.Files {
		dst := filepath.Join(skillPath, rel)
		if _, err := os.Stat(dst); err != nil {
			t.Errorf("expected copied file missing: %s", rel)
		}
	}
}

func TestSyncSkill_RemovesStaleFiles(t *testing.T) {
	vault := copyFixtures(t, fixtureVault)
	projects := copyFixtures(t, fixtureProjects)

	masterSkills, err := ReadMasterSkills(vault)
	if err != nil {
		t.Fatalf("ReadMasterSkills: %v", err)
	}

	skillPath := filepath.Join(projects, "org/team/project-a/.agents/skills/coding")
	target := Target{
		ProjectPath: filepath.Join(projects, "org/team/project-a"),
		SkillName:   "coding",
		SkillPath:   skillPath,
	}

	stale := filepath.Join(skillPath, "STALE_FILE.md")
	if _, err := os.Stat(stale); err != nil {
		t.Fatalf("fixture missing STALE_FILE.md: %v", err)
	}

	result := SyncSkill(masterSkills, target, false)
	if result.Err != nil {
		t.Fatalf("SyncSkill: %v", result.Err)
	}

	// Stale file must be gone from disk
	if _, err := os.Stat(stale); !os.IsNotExist(err) {
		t.Error("STALE_FILE.md should have been removed but still exists")
	}

	// And reported in Removed
	found := false
	for _, r := range result.Removed {
		if r == "STALE_FILE.md" {
			found = true
			break
		}
	}
	if !found {
		t.Errorf("STALE_FILE.md not in result.Removed: %v", result.Removed)
	}
}

func TestSyncSkill_DryRunTouchesNothing(t *testing.T) {
	vault := copyFixtures(t, fixtureVault)
	projects := copyFixtures(t, fixtureProjects)

	masterSkills, err := ReadMasterSkills(vault)
	if err != nil {
		t.Fatalf("ReadMasterSkills: %v", err)
	}

	skillPath := filepath.Join(projects, "org/team/project-a/.agents/skills/coding")
	target := Target{
		ProjectPath: filepath.Join(projects, "org/team/project-a"),
		SkillName:   "coding",
		SkillPath:   skillPath,
	}

	result := SyncSkill(masterSkills, target, true)
	if result.Err != nil {
		t.Fatalf("SyncSkill dry-run: %v", result.Err)
	}

	// Stale file must still be on disk
	stale := filepath.Join(skillPath, "STALE_FILE.md")
	if _, err := os.Stat(stale); err != nil {
		t.Error("dry-run must not delete STALE_FILE.md")
	}

	// Vault files must NOT have been written (SKILL.md content unchanged)
	content, _ := os.ReadFile(filepath.Join(skillPath, "SKILL.md"))
	if string(content) != "# coding (outdated — will be overwritten by sync)\n" {
		t.Error("dry-run must not overwrite existing files")
	}

	// But Removed must still be populated
	if len(result.Removed) == 0 {
		t.Error("dry-run should still report what would be removed")
	}
}
