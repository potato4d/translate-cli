package agent

import (
	"context"

	"github.com/potato4d/translate-cli/internal/output"
	"github.com/potato4d/translate-cli/internal/translate"
)

type ClaudeAdapter struct{}

func (ClaudeAdapter) ID() string {
	return "claude"
}

func (ClaudeAdapter) DisplayName() string {
	return "Claude"
}

func (a ClaudeAdapter) Detect(ctx context.Context, runtime DetectionRuntime) DetectionResult {
	path, found := lookPath(a.ID())
	result := DetectionResult{
		ID:          a.ID(),
		DisplayName: a.DisplayName(),
		Path:        path,
		Found:       found,
		Status:      "not found",
	}
	if found {
		result.NonInteractive = true
		result.Status = "found"
	}
	result.Score = score(a.ID(), result.Found, result.Authenticated, result.NonInteractive, runtime)
	return result
}

func (ClaudeAdapter) BuildCommand(req translate.TranslationRequest, runtime RuntimeContext) ExecSpec {
	return ExecSpec{
		Command: "claude",
		Args: []string{
			"--bare",
			"-p", translate.BuildPrompt(req),
			"--output-format", "json",
			"--json-schema", translate.JSONSchema,
			"--no-session-persistence",
			"--max-turns", "1",
			"--tools", "",
		},
		AllowRaw: true,
	}
}

func (a ClaudeAdapter) ExtractResult(raw ExecResult) (translate.TranslationResult, error) {
	result, err := output.NormalizeAllowRaw(raw.Stdout)
	if err != nil {
		return translate.TranslationResult{}, parseError(a.ID(), err)
	}
	return result, nil
}
