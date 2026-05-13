package cli

import (
	"strings"

	apperrors "github.com/potato4d/translate-cli/internal/errors"
	"github.com/potato4d/translate-cli/internal/locale"
	"github.com/potato4d/translate-cli/internal/translate"
)

type Invocation struct {
	Tool       string
	Text       string
	TargetLang *locale.Language
	Mode       translate.TranslationMode
	Version    bool
	Help       bool
	Setup      bool
	NoWizard   bool
}

func Parse(args []string) (Invocation, error) {
	var inv Invocation
	var positional []string

	for i := 0; i < len(args); i++ {
		arg := args[i]
		switch {
		case arg == "--":
			positional = append(positional, args[i+1:]...)
			i = len(args)
		case arg == "--version" || arg == "-v":
			inv.Version = true
		case arg == "--help" || arg == "-h":
			inv.Help = true
		case arg == "--setup":
			inv.Setup = true
		case arg == "--no-wizard":
			inv.NoWizard = true
		case arg == "--tool":
			if i+1 >= len(args) {
				return inv, apperrors.New(apperrors.CodeUsage, "--tool requires a value")
			}
			inv.Tool = args[i+1]
			i++
		case strings.HasPrefix(arg, "--tool="):
			inv.Tool = strings.TrimPrefix(arg, "--tool=")
			if inv.Tool == "" {
				return inv, apperrors.New(apperrors.CodeUsage, "--tool requires a value")
			}
		case strings.HasPrefix(arg, "-"):
			return inv, apperrors.New(apperrors.CodeUsage, "unknown option: %s", arg)
		default:
			positional = append(positional, arg)
		}
	}

	applyPositional(&inv, positional)
	return inv, nil
}

func applyPositional(inv *Invocation, positional []string) {
	if len(positional) == 0 {
		inv.Mode = translate.ModeAutoPair
		return
	}

	if lang, ok := locale.ResolveLanguage(positional[0]); ok {
		inv.TargetLang = &lang
		inv.Mode = translate.ModeTarget
		if len(positional) > 1 {
			inv.Text = strings.Join(positional[1:], " ")
		}
		return
	}

	inv.Mode = translate.ModeAutoPair
	inv.Text = strings.Join(positional, " ")
}

func Usage() string {
	return strings.TrimSpace(`Usage:
  t <text>
  t <lang> <text>
  t --tool <tool> <text>
  t --tool <tool> <lang> <text>

Options:
  --tool <codex|claude>  Use a specific Agent CLI
  --setup               Run first-run setup
  --no-wizard           Fail instead of running setup automatically
  --version             Print version
  --help                Show help`)
}
