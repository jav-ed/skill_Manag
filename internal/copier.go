package internal

import (
	"io"
	"io/fs"
	"os"
	"os/exec"
	"path/filepath"
	"strings"
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

	// Prefer git-aware file list; fall back to filtered walk if vault isn't a git repo
	files, ok := gitListFiles(srcDir)
	if !ok {
		var err error
		files, err = walkVaultFiles(srcDir)
		if err != nil {
			result.Err = err
			return result
		}
	}
	result.Files = files

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

// gitListFiles returns all files tracked by git under dir, relative to dir.
// Only staged/committed files are included — untracked files are intentionally
// excluded so mid-transition vault state (partial renames, unstaged adds) cannot
// produce duplicate or ghost entries.
// Returns false if git is unavailable or dir is not inside a git repo.
func gitListFiles(dir string) ([]string, bool) {
	out, err := exec.Command("git", "-C", dir, "ls-files", "--cached", ".").Output()
	if err != nil {
		return nil, false
	}
	var files []string
	for _, line := range strings.Split(strings.TrimSpace(string(out)), "\n") {
		if line != "" {
			files = append(files, filepath.FromSlash(line))
		}
	}
	return files, true
}

// walkVaultFiles collects regular, non-symlink files under dir, skipping skipDirs.
// Used as a fallback when the vault is not a git repo.
func walkVaultFiles(dir string) ([]string, error) {
	var files []string
	err := filepath.WalkDir(dir, func(path string, d os.DirEntry, err error) error {
		if err != nil {
			return err
		}
		if d.IsDir() && skipDirs[d.Name()] {
			return filepath.SkipDir
		}
		if d.Type()&fs.ModeSymlink != 0 || d.IsDir() {
			return nil
		}
		rel, err := filepath.Rel(dir, path)
		if err != nil {
			return err
		}
		files = append(files, rel)
		return nil
	})
	return files, err
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
