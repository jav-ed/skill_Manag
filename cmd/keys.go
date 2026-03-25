package cmd

import "github.com/charmbracelet/bubbles/key"

// syncKeyMap defines keybindings for the sync selection screen
type syncKeyMap struct {
	Up      key.Binding
	Down    key.Binding
	Toggle  key.Binding
	All     key.Binding
	Confirm key.Binding
	Help    key.Binding
	Quit    key.Binding
}

func (k syncKeyMap) ShortHelp() []key.Binding {
	return []key.Binding{k.Toggle, k.All, k.Confirm, k.Help, k.Quit}
}
func (k syncKeyMap) FullHelp() [][]key.Binding {
	return [][]key.Binding{
		{k.Up, k.Down},
		{k.Toggle, k.All},
		{k.Confirm, k.Quit},
	}
}

var syncKeys = syncKeyMap{
	Up:      key.NewBinding(key.WithKeys("up", "k"), key.WithHelp("↑/k", "up")),
	Down:    key.NewBinding(key.WithKeys("down", "j"), key.WithHelp("↓/j", "down")),
	Toggle:  key.NewBinding(key.WithKeys(" "), key.WithHelp("space", "toggle")),
	All:     key.NewBinding(key.WithKeys("a"), key.WithHelp("a", "all")),
	Confirm: key.NewBinding(key.WithKeys("enter"), key.WithHelp("enter", "confirm")),
	Help:    key.NewBinding(key.WithKeys("?"), key.WithHelp("?", "help")),
	Quit:    key.NewBinding(key.WithKeys("q", "ctrl+c"), key.WithHelp("q", "quit")),
}

// deleteKeyMap defines keybindings for the delete selection screen
type deleteKeyMap struct {
	Up      key.Binding
	Down    key.Binding
	Toggle  key.Binding
	All     key.Binding
	Confirm key.Binding
	Help    key.Binding
	Quit    key.Binding
}

func (k deleteKeyMap) ShortHelp() []key.Binding {
	return []key.Binding{k.Toggle, k.All, k.Confirm, k.Help, k.Quit}
}
func (k deleteKeyMap) FullHelp() [][]key.Binding {
	return [][]key.Binding{
		{k.Up, k.Down},
		{k.Toggle, k.All},
		{k.Confirm, k.Quit},
	}
}

var deleteKeys = deleteKeyMap{
	Up:      key.NewBinding(key.WithKeys("up", "k"), key.WithHelp("↑/k", "up")),
	Down:    key.NewBinding(key.WithKeys("down", "j"), key.WithHelp("↓/j", "down")),
	Toggle:  key.NewBinding(key.WithKeys(" "), key.WithHelp("space", "toggle")),
	All:     key.NewBinding(key.WithKeys("a"), key.WithHelp("a", "all")),
	Confirm: key.NewBinding(key.WithKeys("enter"), key.WithHelp("enter", "confirm")),
	Help:    key.NewBinding(key.WithKeys("?"), key.WithHelp("?", "help")),
	Quit:    key.NewBinding(key.WithKeys("q", "ctrl+c"), key.WithHelp("q", "quit")),
}

// listKeyMap defines keybindings for the list browser screen
type listKeyMap struct {
	Up     key.Binding
	Down   key.Binding
	Toggle key.Binding
	All    key.Binding
	Filter key.Binding
	Sync   key.Binding
	Delete key.Binding
	Help   key.Binding
	Quit   key.Binding
}

func (k listKeyMap) ShortHelp() []key.Binding {
	return []key.Binding{k.Filter, k.Toggle, k.Sync, k.Delete, k.Help, k.Quit}
}
func (k listKeyMap) FullHelp() [][]key.Binding {
	return [][]key.Binding{
		{k.Up, k.Down},
		{k.Toggle, k.All},
		{k.Filter, k.Sync, k.Delete},
		{k.Help, k.Quit},
	}
}

var listKeys = listKeyMap{
	Up:     key.NewBinding(key.WithKeys("up", "k"), key.WithHelp("↑/k", "up")),
	Down:   key.NewBinding(key.WithKeys("down", "j"), key.WithHelp("↓/j", "down")),
	Toggle: key.NewBinding(key.WithKeys(" "), key.WithHelp("space", "toggle")),
	All:    key.NewBinding(key.WithKeys("a"), key.WithHelp("a", "all")),
	Filter: key.NewBinding(key.WithKeys("/"), key.WithHelp("/", "filter")),
	Sync:   key.NewBinding(key.WithKeys("s"), key.WithHelp("s", "sync")),
	Delete: key.NewBinding(key.WithKeys("d"), key.WithHelp("d", "delete")),
	Help:   key.NewBinding(key.WithKeys("?"), key.WithHelp("?", "help")),
	Quit:   key.NewBinding(key.WithKeys("q", "ctrl+c"), key.WithHelp("q", "quit")),
}
