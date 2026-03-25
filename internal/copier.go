package internal

import (
	"io"
	"os"
	"path/filepath"
)

// SyncResult holds the outcome of syncing one skill to one project
type SyncResult struct {
	Target Target
	Files  []string // relative file paths that were (or would be) copied
	Err    error
}

// SyncSkill copies all files from the master skill dir into the target skill dir.
// When dryRun is true it collects what would change but writes nothing.
func SyncSkill(masterSkills map[string]string, target Target, dryRun bool) SyncResult {
	srcDir := masterSkills[target.SkillName]
	result := SyncResult{Target: target}

	err := filepath.WalkDir(srcDir, func(path string, d os.DirEntry, err error) error {
		if err != nil {
			return err
		}

		rel, err := filepath.Rel(srcDir, path)
		if err != nil {
			return err
		}

		dst := filepath.Join(target.SkillPath, rel)

		if d.IsDir() {
			if !dryRun {
				return os.MkdirAll(dst, 0755)
			}
			return nil
		}

		// Track every file that is being synced
		result.Files = append(result.Files, rel)

		if !dryRun {
			return copyFile(path, dst)
		}

		return nil
	})

	result.Err = err
	return result
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
