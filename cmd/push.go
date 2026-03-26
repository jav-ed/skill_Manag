package cmd

import (
	"github.com/spf13/viper"
	"skill_Manag/cmd/tui"
)

func doPush(vault, root string) error {
	viper.ReadInConfig() // ensure latest mandatory — may have been edited from push screen or setup
	mandatory := viper.GetStringSlice("mandatory")
	return tui.RunPush(vault, root, mandatory)
}
