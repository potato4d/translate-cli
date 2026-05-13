package wizard

import (
	"bufio"
	"context"
	"fmt"
	"io"
	"os"
	"strconv"
	"strings"

	"github.com/potato4d/translate-cli/internal/agent"
	"github.com/potato4d/translate-cli/internal/config"
	apperrors "github.com/potato4d/translate-cli/internal/errors"
	"github.com/potato4d/translate-cli/internal/locale"
)

type Options struct {
	ConfigPath string
	Config     config.Config
}

func Run(ctx context.Context, in io.Reader, out io.Writer, opts Options) (config.Config, error) {
	cfg := opts.Config
	if cfg.Version == 0 {
		cfg = config.Default()
	}

	results := agent.DetectAll(ctx, agent.DetectionRuntime{
		ExistingDefault: cfg.DefaultTool,
		EnvTool:         os.Getenv("TRANSLATE_CLI_TOOL"),
	})
	tools := foundTools(results)
	if len(tools) == 0 {
		return cfg, apperrors.WithHint(
			apperrors.New(apperrors.CodeToolNotFound, "no supported Agent CLI found"),
			"Install one of: codex, claude. Then run: t --setup",
		)
	}

	local := locale.MustLanguage(cfg.LocalLang)
	if cfg.LocalLang == "" {
		local = locale.DetectLocalLanguage()
	}
	recommended, _ := agent.Recommended(results)

	reader := bufio.NewReader(in)
	fmt.Fprintln(out, "translate CLI setup")
	fmt.Fprintln(out)
	fmt.Fprintln(out, "translate CLI sends text to the selected Agent CLI.")
	fmt.Fprintln(out, "The Agent CLI may send it to its configured model provider.")
	fmt.Fprintln(out, "translate CLI does not store API keys.")
	fmt.Fprintln(out)
	fmt.Fprintf(out, "Detected your local language: %s (%s)\n", local.Name, local.Code)
	fmt.Fprintln(out)
	fmt.Fprintln(out, "Available tools:")
	for i, tool := range tools {
		fmt.Fprintf(out, "  %d. %-6s %s\n", i+1, tool.ID, tool.Status)
	}
	fmt.Fprintln(out)
	fmt.Fprintf(out, "Recommended tool: %s\n", recommended.ID)
	fmt.Fprintf(out, "Use %s as the default tool? [Y/n] ", recommended.ID)

	answer := readAnswer(reader)
	selected := recommended.ID
	if strings.EqualFold(answer, "n") || strings.EqualFold(answer, "no") {
		selected = selectTool(reader, out, tools)
	}

	fmt.Fprintf(out, "Use %s <-> English as default translation pair? [Y/n] ", local.Name)
	answer = readAnswer(reader)
	if strings.EqualFold(answer, "n") || strings.EqualFold(answer, "no") {
		fmt.Fprint(out, "Local language code [", local.Code, "]: ")
		if custom := readAnswer(reader); custom != "" {
			local = locale.MustLanguage(custom)
		}
	}

	cfg.DefaultTool = selected
	cfg.LocalLang = local.Code
	if cfg.TimeoutMS <= 0 {
		cfg.TimeoutMS = config.DefaultTimeoutMS
	}
	if cfg.Tools == nil {
		cfg.Tools = config.Default().Tools
	}

	if err := config.Save(opts.ConfigPath, cfg); err != nil {
		return cfg, err
	}
	fmt.Fprintln(out)
	fmt.Fprintf(out, "Saved config: %s\n", opts.ConfigPath)
	return cfg, nil
}

func selectTool(reader *bufio.Reader, out io.Writer, tools []agent.DetectionResult) string {
	fmt.Fprintln(out, "Select default tool:")
	for i, tool := range tools {
		fmt.Fprintf(out, "  %d. %s\n", i+1, tool.ID)
	}
	fmt.Fprint(out, "Tool [1]: ")
	answer := readAnswer(reader)
	if answer == "" {
		return tools[0].ID
	}
	n, err := strconv.Atoi(answer)
	if err != nil || n < 1 || n > len(tools) {
		return tools[0].ID
	}
	return tools[n-1].ID
}

func readAnswer(reader *bufio.Reader) string {
	line, err := reader.ReadString('\n')
	if err != nil && line == "" {
		return ""
	}
	return strings.TrimSpace(line)
}
