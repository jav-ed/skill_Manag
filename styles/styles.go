package styles

import (
	lipgloss "github.com/charmbracelet/lipgloss"
	v2 "charm.land/lipgloss/v2"
)

var (
	Success   = lipgloss.NewStyle().Foreground(lipgloss.Color("#00FF87")).Bold(true)
	Warning   = lipgloss.NewStyle().Foreground(lipgloss.Color("#FFDB58"))
	Error     = lipgloss.NewStyle().Foreground(lipgloss.Color("#FF5F5F")).Bold(true)
	Muted     = lipgloss.NewStyle().Foreground(lipgloss.Color("#626262"))
	Header    = lipgloss.NewStyle().Bold(true)
	SkillName = lipgloss.NewStyle().Foreground(lipgloss.Color("#87CEEB"))
)

// HeaderLinkRow is the terminal row (0-indexed) where AppHeader renders.
// All views prefix output with "\n", so the header is always on row 1.
const HeaderLinkRow = 1

// HeaderLinkCol is the visual start column (0-indexed) of "javedab.com" in AppHeader.
// prefix(3) + "skill_Manag"(11) + sep(5) = 19. Holds for both menu and child screens
// because " ← " and "   " are both 3 visual chars.
const HeaderLinkCol = 19

// HeaderLinkLen is the visual width of the link text "javedab.com".
const HeaderLinkLen = 11

// HeaderBackRow is the terminal row of the back arrow "←" in child-screen headers.
const HeaderBackRow = 1

// HeaderBackCol is the column of "←" — padded with one space on each side, so the
// arrow itself sits at col 1.
const HeaderBackCol = 1

// HeaderBackLen is the visual width of the back arrow.
const HeaderBackLen = 1

// AppHeader returns the standard top bar.
// When screen == "", renders the main-menu header with no back button ("   skill_Manag …").
// When screen != "", shows " ← " as the prefix (padded on both sides).
// Pass backHovered=true to highlight the arrow.
func AppHeader(screen string, linkHovered bool, backHovered bool) string {
	muted := v2.NewStyle().Foreground(v2.Color("#626262"))
	sep := muted.Render("  ·  ")

	var prefix string
	if screen != "" {
		arrowStyle := muted
		if backHovered {
			arrowStyle = v2.NewStyle().Foreground(v2.Color("#87CEEB"))
		}
		prefix = " " + arrowStyle.Render("←") + " "
	} else {
		prefix = "   "
	}
	name := prefix + v2.NewStyle().Bold(true).Render("skill_Manag")

	linkStyle := muted.Underline(true).Hyperlink("https://javedab.com")
	if linkHovered {
		linkStyle = v2.NewStyle().Foreground(v2.Color("#87CEEB")).Underline(true).Hyperlink("https://javedab.com")
	}
	link := linkStyle.Render("javedab.com")

	s := name + sep + link
	if screen != "" {
		s += sep + muted.Render(screen)
	}
	return s
}
