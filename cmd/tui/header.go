package tui

import (
	tea "github.com/charmbracelet/bubbletea"
	"skill_Manag/styles"
)

// handleHeaderMouse processes a MouseMsg for the app header.
// hasBack must be true for child screens (those that show the "←" back arrow).
// Returns updated hover states and whether to open the browser or go back.
func handleHeaderMouse(msg tea.MouseMsg, linkHovered, backHovered bool, hasBack bool) (newLinkHovered, newBackHovered bool, openURL, goBack bool) {
	onLink := msg.Y == styles.HeaderLinkRow &&
		msg.X >= styles.HeaderLinkCol &&
		msg.X < styles.HeaderLinkCol+styles.HeaderLinkLen

	onBack := hasBack &&
		msg.Y == styles.HeaderBackRow &&
		msg.X >= styles.HeaderBackCol &&
		msg.X < styles.HeaderBackCol+styles.HeaderBackLen

	newLinkHovered = linkHovered
	newBackHovered = backHovered
	if msg.Action == tea.MouseActionMotion || msg.Action == tea.MouseActionPress {
		newLinkHovered = onLink
		if hasBack {
			newBackHovered = onBack
		}
	}
	openURL = msg.Action == tea.MouseActionPress && msg.Button == tea.MouseButtonLeft && onLink
	goBack = msg.Action == tea.MouseActionPress && msg.Button == tea.MouseButtonLeft && onBack
	return
}
