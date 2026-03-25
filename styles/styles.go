package styles

import "github.com/charmbracelet/lipgloss"

var (
	Success   = lipgloss.NewStyle().Foreground(lipgloss.Color("#00FF87")).Bold(true)
	Warning   = lipgloss.NewStyle().Foreground(lipgloss.Color("#FFDB58"))
	Error     = lipgloss.NewStyle().Foreground(lipgloss.Color("#FF5F5F")).Bold(true)
	Muted     = lipgloss.NewStyle().Foreground(lipgloss.Color("#626262"))
	Header    = lipgloss.NewStyle().Bold(true)
	SkillName = lipgloss.NewStyle().Foreground(lipgloss.Color("#87CEEB"))
)
