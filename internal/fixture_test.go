package internal

import (
	"io"
	"os"
	"path/filepath"
	"testing"
)

const (
	fixtureVault    = "testdata/vault"
	fixtureProjects = "testdata/projects"
)

// copyFixtures copies a testdata directory into t.TempDir() and returns the
// path to the copy. Tests that mutate the filesystem must call this — the
// originals in testdata/ are never touched.
//
// It also creates .git directories inside every direct child of the copy that
// contains a .agents/ directory, simulating real project repos. These cannot
// be committed to git, so they are added at runtime.
func copyFixtures(t *testing.T, src string) string {
	t.Helper()

	dst := t.TempDir()
	if err := copyDir(src, dst); err != nil {
		t.Fatalf("copyFixtures: %v", err)
	}

	// Add .git dirs to simulate real projects (only for the projects fixture)
	if filepath.Base(src) == "projects" {
		addGitDirs(t, dst)
	}

	return dst
}

// copyDir recursively copies src into dst.
func copyDir(src, dst string) error {
	return filepath.WalkDir(src, func(path string, d os.DirEntry, err error) error {
		if err != nil {
			return err
		}

		rel, err := filepath.Rel(src, path)
		if err != nil {
			return err
		}

		target := filepath.Join(dst, rel)

		if d.IsDir() {
			return os.MkdirAll(target, 0755)
		}

		return copyFile2(path, target)
	})
}

func copyFile2(src, dst string) error {
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

// addGitDirs walks root and creates an empty .git directory next to every
// .agents directory it finds — matching the layout of a real project.
func addGitDirs(t *testing.T, root string) {
	t.Helper()

	err := filepath.WalkDir(root, func(path string, d os.DirEntry, err error) error {
		if err != nil {
			return nil
		}
		if d.IsDir() && d.Name() == ".agents" {
			gitDir := filepath.Join(filepath.Dir(path), ".git")
			if err := os.MkdirAll(gitDir, 0755); err != nil {
				return err
			}
		}
		return nil
	})

	if err != nil {
		t.Fatalf("addGitDirs: %v", err)
	}
}
