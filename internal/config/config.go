package config

import (
	"bufio"
	"fmt"
	"os"
	"path/filepath"
	"strconv"
	"strings"

	apperrors "github.com/potato4d/translate-cli/internal/errors"
	"github.com/potato4d/translate-cli/internal/locale"
)

const DefaultTimeoutMS = 60000

type ToolConfig struct {
	Enabled bool
}

type Config struct {
	Version     int
	DefaultTool string
	LocalLang   string
	TimeoutMS   int
	Tools       map[string]ToolConfig
}

func Default() Config {
	return Config{
		Version:   1,
		LocalLang: locale.DetectLocalLanguage().Code,
		TimeoutMS: DefaultTimeoutMS,
		Tools: map[string]ToolConfig{
			"codex":  {Enabled: true},
			"claude": {Enabled: true},
		},
	}
}

func Load(path string) (Config, bool, error) {
	cfg := Default()
	data, err := os.ReadFile(path)
	if os.IsNotExist(err) {
		return cfg, false, nil
	}
	if err != nil {
		return cfg, false, err
	}

	if err := parseTOML(&cfg, string(data)); err != nil {
		return cfg, true, apperrors.WithHint(
			apperrors.New(apperrors.CodeConfig, "failed to parse config: %s", err),
			path,
		)
	}
	if cfg.TimeoutMS <= 0 {
		cfg.TimeoutMS = DefaultTimeoutMS
	}
	if cfg.Tools == nil {
		cfg.Tools = Default().Tools
	}
	return cfg, true, nil
}

func Save(path string, cfg Config) error {
	if cfg.Version == 0 {
		cfg.Version = 1
	}
	if cfg.TimeoutMS <= 0 {
		cfg.TimeoutMS = DefaultTimeoutMS
	}
	if cfg.Tools == nil {
		cfg.Tools = Default().Tools
	}

	if err := os.MkdirAll(filepath.Dir(path), 0700); err != nil {
		return err
	}

	var b strings.Builder
	fmt.Fprintf(&b, "version = %d\n", cfg.Version)
	fmt.Fprintf(&b, "default_tool = %q\n", cfg.DefaultTool)
	fmt.Fprintf(&b, "local_lang = %q\n", cfg.LocalLang)
	fmt.Fprintf(&b, "timeout_ms = %d\n\n", cfg.TimeoutMS)

	for _, id := range []string{"codex", "claude"} {
		tool := cfg.Tools[id]
		fmt.Fprintf(&b, "[tools.%s]\n", id)
		fmt.Fprintf(&b, "enabled = %t\n\n", tool.Enabled)
	}

	return os.WriteFile(path, []byte(b.String()), 0600)
}

func LocalLanguage(cfg Config) locale.Language {
	return locale.MustLanguage(cfg.LocalLang)
}

func parseTOML(cfg *Config, data string) error {
	section := ""
	scanner := bufio.NewScanner(strings.NewReader(data))
	for scanner.Scan() {
		line := scanner.Text()
		if i := strings.Index(line, "#"); i >= 0 {
			line = line[:i]
		}
		line = strings.TrimSpace(line)
		if line == "" {
			continue
		}
		if strings.HasPrefix(line, "[") && strings.HasSuffix(line, "]") {
			section = strings.TrimSpace(strings.TrimSuffix(strings.TrimPrefix(line, "["), "]"))
			continue
		}

		key, value, ok := strings.Cut(line, "=")
		if !ok {
			return fmt.Errorf("invalid line %q", line)
		}
		key = strings.TrimSpace(key)
		value = strings.TrimSpace(value)

		switch section {
		case "":
			if err := parseRootValue(cfg, key, value); err != nil {
				return err
			}
		case "tools.codex", "tools.claude":
			id := strings.TrimPrefix(section, "tools.")
			if cfg.Tools == nil {
				cfg.Tools = map[string]ToolConfig{}
			}
			tool := cfg.Tools[id]
			if key == "enabled" {
				enabled, err := strconv.ParseBool(value)
				if err != nil {
					return fmt.Errorf("%s.enabled: %w", id, err)
				}
				tool.Enabled = enabled
				cfg.Tools[id] = tool
			}
		default:
			continue
		}
	}
	return scanner.Err()
}

func parseRootValue(cfg *Config, key string, value string) error {
	switch key {
	case "version":
		v, err := strconv.Atoi(value)
		if err != nil {
			return fmt.Errorf("version: %w", err)
		}
		cfg.Version = v
	case "default_tool":
		v, err := strconv.Unquote(value)
		if err != nil {
			return fmt.Errorf("default_tool: %w", err)
		}
		cfg.DefaultTool = v
	case "local_lang":
		v, err := strconv.Unquote(value)
		if err != nil {
			return fmt.Errorf("local_lang: %w", err)
		}
		cfg.LocalLang = v
	case "timeout_ms":
		v, err := strconv.Atoi(value)
		if err != nil {
			return fmt.Errorf("timeout_ms: %w", err)
		}
		cfg.TimeoutMS = v
	}
	return nil
}
