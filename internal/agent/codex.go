package agent

import (
	"context"
	"os/exec"
	"strings"

	apperrors "github.com/potato4d/translate-cli/internal/errors"
	"github.com/potato4d/translate-cli/internal/output"
	"github.com/potato4d/translate-cli/internal/translate"
)

type CodexAdapter struct{}

func (CodexAdapter) ID() string {
	return "codex"
}

func (CodexAdapter) DisplayName() string {
	return "Codex"
}

func (a CodexAdapter) Detect(ctx context.Context, runtime DetectionRuntime) DetectionResult {
	path, found := lookPath(a.ID())
	result := DetectionResult{
		ID:          a.ID(),
		DisplayName: a.DisplayName(),
		Path:        path,
		Found:       found,
		Status:      "not found",
	}
	if !found {
		result.Score = score(a.ID(), false, false, false, runtime)
		return result
	}

	result.NonInteractive = true
	loginCtx, cancel := detectContext(ctx)
	defer cancel()
	login := exec.CommandContext(loginCtx, "codex", "login", "status")
	if err := login.Run(); err == nil {
		result.Authenticated = true
		result.Status = "found, authenticated"
	} else {
		result.Status = "found, authentication unknown"
	}
	result.Score = score(a.ID(), result.Found, result.Authenticated, result.NonInteractive, runtime)
	return result
}

func (CodexAdapter) BuildCommand(req translate.TranslationRequest, runtime RuntimeContext) ExecSpec {
	return ExecSpec{
		Command: "codex",
		Args: []string{
			"--ask-for-approval", "never",
			"--model", "gpt-5.3-codex-spark",
			"-c", `model_reasoning_effort="low"`,
			"exec",
			"--skip-git-repo-check",
			"--ignore-user-config",
			"--ignore-rules",
			"--ephemeral",
			"--sandbox", "read-only",
			"--color", "never",
			"--output-schema", runtime.SchemaPath,
			"--output-last-message", runtime.LastMessagePath,
			"-",
		},
		Stdin: translate.BuildPrompt(req),
	}
}

func (a CodexAdapter) ExtractResult(raw ExecResult) (translate.TranslationResult, error) {
	source := raw.LastMessageText
	if strings.TrimSpace(source) == "" {
		source = raw.Stdout
	}
	result, err := output.Normalize(source)
	if err != nil {
		return translate.TranslationResult{}, parseError(a.ID(), err)
	}
	return result, nil
}

func CodexAuthenticationError() error {
	return apperrors.WithHint(
		apperrors.New(apperrors.CodeAgentExecution, "codex is not authenticated"),
		"Run: codex",
	)
}
