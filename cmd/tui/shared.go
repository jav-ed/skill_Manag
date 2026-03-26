package tui

import (
	"path/filepath"
	"strings"
)

type phase int

const (
	phaseLoading phase = iota
	phaseSelect
	phaseSyncing
	phaseConfirm
	phaseDone
)

const pageSize = 10

// shortPath returns the last 2 path components for compact display.
func shortPath(p string) string {
	p = filepath.ToSlash(p)
	parts := strings.Split(strings.TrimRight(p, "/"), "/")
	if len(parts) >= 2 {
		return parts[len(parts)-2] + "/" + parts[len(parts)-1]
	}
	return filepath.Base(p)
}
