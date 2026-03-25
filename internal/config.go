package internal

import (
	"os"
	"path/filepath"

	"github.com/spf13/viper"
)

type Config struct {
	Source string
	Root   string
}

// LoadConfig reads config from ~/.config/skill_Manag/config.yaml.
// Flags always override config file values — viper handles the precedence.
func LoadConfig() Config {
	home, err := os.UserHomeDir()
	if err != nil {
		return Config{}
	}

	// Tell viper where to look for the config file
	viper.SetConfigName("config")
	viper.SetConfigType("yaml")
	viper.AddConfigPath(filepath.Join(home, ".config", "skill_Manag"))

	// Silently ignore missing config — it is optional
	viper.ReadInConfig()

	return Config{
		Source: viper.GetString("source"),
		Root:   viper.GetString("root"),
	}
}
