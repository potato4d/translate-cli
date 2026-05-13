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

const maxCodexPromptArgBytes = 16 * 1024

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
	if runtime.SkipAuth {
		result.Authenticated = true
		result.Status = "found"
		result.Score = score(a.ID(), result.Found, result.Authenticated, result.NonInteractive, runtime)
		return result
	}

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
	prompt := translate.BuildPlainTextPrompt(req)
	args := []string{
		"--ask-for-approval", "never",
		"--model", "gpt-5.3-codex-spark",
		"-c", `model_reasoning_effort="low"`,
		"-c", "include_permissions_instructions=false",
		"-c", "include_apps_instructions=false",
		"-c", "include_environment_context=false",
		"-c", "include_apply_patch_tool=false",
		"exec",
		"--skip-git-repo-check",
		"--ignore-user-config",
		"--ignore-rules",
		"--ephemeral",
		"--sandbox", "read-only",
		"--color", "never",
		"--json",
	}

	stdin := ""
	if len(prompt) <= maxCodexPromptArgBytes {
		args = append(args, prompt)
	} else {
		args = append(args, "-")
		stdin = prompt
	}

	return ExecSpec{
		Command:    "codex",
		Args:       args,
		Stdin:      stdin,
		WorkDir:    runtime.WorkDir,
		AllowRaw:   true,
		StreamJSON: true,
	}
}

func (a CodexAdapter) ExtractResult(raw ExecResult) (translate.TranslationResult, error) {
	source := raw.LastMessageText
	if strings.TrimSpace(source) == "" {
		source = raw.Stdout
	}
	result, err := output.NormalizeAllowRaw(source)
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
