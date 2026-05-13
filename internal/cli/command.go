package cli

import (
	"context"
	"fmt"
	"io"
	"os"
	"strings"

	"github.com/potato4d/translate-cli/internal/agent"
	"github.com/potato4d/translate-cli/internal/config"
	apperrors "github.com/potato4d/translate-cli/internal/errors"
	"github.com/potato4d/translate-cli/internal/translate"
	"github.com/potato4d/translate-cli/internal/wizard"
)

const Version = "0.1.0"

func Main(args []string, stdin io.Reader, stdout io.Writer, stderr io.Writer) int {
	err := run(context.Background(), args, stdin, stdout, stderr)
	return apperrors.Render(err, stderr)
}

func run(ctx context.Context, args []string, stdin io.Reader, stdout io.Writer, stderr io.Writer) error {
	inv, err := Parse(args)
	if err != nil {
		return err
	}

	if inv.Help {
		fmt.Fprintln(stdout, Usage())
		return nil
	}
	if inv.Version {
		fmt.Fprintf(stdout, "translate-cli %s\n", Version)
		return nil
	}

	if inv.Text == "" && !inv.Setup {
		text, ok, err := readStdinIfAvailable(stdin)
		if err != nil {
			return err
		}
		if ok {
			inv.Text = text
		}
	}

	configPath, err := config.DefaultPath()
	if err != nil {
		return err
	}
	cfg, exists, err := config.Load(configPath)
	if err != nil {
		return err
	}

	if inv.Setup || needsWizard(cfg, exists) {
		if inv.NoWizard {
			return apperrors.New(apperrors.CodeConfig, "setup is required")
		}
		cfg, err = wizard.Run(ctx, stdin, stderr, wizard.Options{
			ConfigPath: configPath,
			Config:     cfg,
		})
		if err != nil {
			return err
		}
		if inv.Setup && inv.Text == "" && inv.TargetLang == nil {
			return nil
		}
	}

	if inv.Text == "" {
		text, ok, err := readStdinIfAvailable(stdin)
		if err != nil {
			return err
		}
		if ok {
			inv.Text = text
		}
	}
	if strings.TrimSpace(inv.Text) == "" {
		return apperrors.WithHint(apperrors.New(apperrors.CodeUsage, "missing text to translate"), Usage())
	}

	toolID := selectedTool(inv, cfg)
	adapter, ok := agent.ByID(toolID)
	if !ok {
		return apperrors.WithHint(
			apperrors.New(apperrors.CodeUsage, "unsupported tool: %s", toolID),
			"available tools: codex, claude",
		)
	}

	detection := adapter.Detect(ctx, agent.DetectionRuntime{ExistingDefault: cfg.DefaultTool, EnvTool: os.Getenv("TRANSLATE_CLI_TOOL"), SkipAuth: true})
	if !detection.Found {
		return apperrors.New(apperrors.CodeToolNotFound, "tool %q is not installed or not found in PATH", toolID)
	}
	if toolID == "codex" && !detection.Authenticated {
		return agent.CodexAuthenticationError()
	}

	req := translate.TranslationRequest{
		Text:       inv.Text,
		TargetLang: inv.TargetLang,
		LocalLang:  config.LocalLanguage(cfg),
		Mode:       inv.Mode,
		Tool:       toolID,
	}
	result, err := agent.Run(ctx, adapter, req, cfg.TimeoutMS)
	if err != nil {
		return err
	}
	fmt.Fprintln(stdout, result.Text)
	return nil
}

func needsWizard(cfg config.Config, exists bool) bool {
	if !exists {
		return true
	}
	if cfg.DefaultTool == "" || cfg.LocalLang == "" {
		return true
	}
	adapter, ok := agent.ByID(cfg.DefaultTool)
	if !ok {
		return true
	}
	detection := adapter.Detect(context.Background(), agent.DetectionRuntime{ExistingDefault: cfg.DefaultTool, EnvTool: os.Getenv("TRANSLATE_CLI_TOOL"), SkipAuth: true})
	return !detection.Found
}

func selectedTool(inv Invocation, cfg config.Config) string {
	if inv.Tool != "" {
		return inv.Tool
	}
	if envTool := os.Getenv("TRANSLATE_CLI_TOOL"); envTool != "" {
		return envTool
	}
	return cfg.DefaultTool
}

func readStdinIfAvailable(stdin io.Reader) (string, bool, error) {
	if file, ok := stdin.(*os.File); ok {
		stat, err := file.Stat()
		if err == nil && stat.Mode()&os.ModeCharDevice != 0 {
			return "", false, nil
		}
	}

	data, err := io.ReadAll(stdin)
	if err != nil {
		return "", false, err
	}
	text := string(data)
	if strings.TrimSpace(text) == "" {
		return "", false, nil
	}
	return text, true, nil
}
