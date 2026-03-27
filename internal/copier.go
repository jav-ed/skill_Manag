package internal

import (
	"io"
	"os"
	"path/filepath"
)

// SyncResult holds the outcome of syncing one skill to one project
type SyncResult struct {
	Target  Target
	Files   []string // relative file paths that were (or would be) copied
	Removed []string // relative file paths that were (or would be) removed as stale
	Err     error
}

// SyncSkill mirrors the master skill dir into the target skill dir.
// It deletes the destination skill directory first, then copies fresh from
// the vault — guaranteeing no stale files remain.
// When dryRun is true it collects what would change but writes nothing.
func SyncSkill(masterSkills map[string]string, target Target, dryRun bool) SyncResult {
	srcDir := masterSkills[target.SkillName]
	result := SyncResult{Target: target}

	// Collect vault files
	err := filepath.WalkDir(srcDir, func(path string, d os.DirEntry, err error) error {
		if err != nil || d.IsDir() {
			return err
		}
		rel, err := filepath.Rel(srcDir, path)
		if err != nil {
			return err
		}
		result.Files = append(result.Files, rel)
		return nil
	})
	if err != nil {
		result.Err = err
		return result
	}

	// Always record what is/would be removed before touching the destination
	result.Removed = staleFiles(target.SkillPath, result.Files)

	if dryRun {
		return result
	}

	// Delete the existing skill dir so no stale files survive
	if err := os.RemoveAll(target.SkillPath); err != nil {
		result.Err = err
		return result
	}

	// Copy vault → destination fresh
	for _, rel := range result.Files {
		src := filepath.Join(srcDir, rel)
		dst := filepath.Join(target.SkillPath, rel)
		if err := copyFile(src, dst); err != nil {
			result.Err = err
			return result
		}
	}

	return result
}

// staleFiles returns relative paths present in dstDir that are not in vaultFiles.
func staleFiles(dstDir string, vaultFiles []string) []string {
	if _, err := os.Stat(dstDir); os.IsNotExist(err) {
		return nil
	}

	keep := make(map[string]bool, len(vaultFiles))
	for _, f := range vaultFiles {
		keep[f] = true
	}

	var stale []string
	_ = filepath.WalkDir(dstDir, func(path string, d os.DirEntry, err error) error {
		if err != nil || d.IsDir() {
			return nil
		}
		rel, err := filepath.Rel(dstDir, path)
		if err != nil {
			return nil
		}
		if !keep[rel] {
			stale = append(stale, rel)
		}
		return nil
	})

	return stale
}

// copyFile copies src to dst, creating any missing parent directories
func copyFile(src, dst string) error {
	if err := os.MkdirAll(filepath.Dir(dst), 0755); err != nil {
		return err
	}

	in, err := os.Open(src)
	if err != nil {
		return err
	}
	defer in.Close()

	out, err := os.Create(dst)
	if err != nil {
		return err
	}
	defer out.Close()

	_, err = io.Copy(out, in)
	return err
}
