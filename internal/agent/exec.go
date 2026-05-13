package agent

import (
	"bytes"
	"context"
	"errors"
	"fmt"
	"os"
	"os/exec"
	"path/filepath"
	"strings"
	"time"

	apperrors "github.com/potato4d/translate-cli/internal/errors"
	"github.com/potato4d/translate-cli/internal/translate"
)

func Run(ctx context.Context, adapter Adapter, req translate.TranslationRequest, timeoutMS int) (translate.TranslationResult, error) {
	if timeoutMS <= 0 {
		timeoutMS = 60000
	}

	tempDir, err := os.MkdirTemp("", "translate-cli-*")
	if err != nil {
		return translate.TranslationResult{}, err
	}
	defer os.RemoveAll(tempDir)

	schemaPath := filepath.Join(tempDir, "schema.json")
	if err := os.WriteFile(schemaPath, []byte(translate.JSONSchema), 0600); err != nil {
		return translate.TranslationResult{}, err
	}

	runtime := RuntimeContext{
		Timeout:         time.Duration(timeoutMS) * time.Millisecond,
		WorkDir:         tempDir,
		SchemaPath:      schemaPath,
		LastMessagePath: filepath.Join(tempDir, "last-message.json"),
	}
	spec := adapter.BuildCommand(req, runtime)

	runCtx, cancel := context.WithTimeout(ctx, runtime.Timeout)
	defer cancel()

	cmd := exec.CommandContext(runCtx, spec.Command, spec.Args...)
	if spec.WorkDir != "" {
		cmd.Dir = spec.WorkDir
	}
	cmd.Stdin = bytes.NewBufferString(spec.Stdin)
	var stdout bytes.Buffer
	var stderr bytes.Buffer
	cmd.Stdout = &stdout
	cmd.Stderr = &stderr

	err = cmd.Run()
	if errors.Is(runCtx.Err(), context.DeadlineExceeded) {
		return translate.TranslationResult{}, apperrors.New(apperrors.CodeTimeout, "translation timed out after %ds", timeoutMS/1000)
	}
	if err != nil {
		return translate.TranslationResult{}, agentRunError(adapter.ID(), err, stderr.String())
	}

	lastMessage := ""
	if data, readErr := os.ReadFile(runtime.LastMessagePath); readErr == nil {
		lastMessage = string(data)
	}

	return adapter.ExtractResult(ExecResult{
		Stdout:          stdout.String(),
		Stderr:          stderr.String(),
		LastMessageText: lastMessage,
	})
}

func agentRunError(id string, runErr error, stderr string) error {
	switch id {
	case "codex":
		return apperrors.WithHint(
			apperrors.New(apperrors.CodeAgentExecution, "codex failed to run: %s", runErrorMessage(runErr, stderr)),
			"Run: codex",
		)
	case "claude":
		return apperrors.WithHint(
			apperrors.New(apperrors.CodeAgentExecution, "claude failed to run: %s", runErrorMessage(runErr, stderr)),
			"Run: claude",
		)
	default:
		if stderr != "" {
			return apperrors.New(apperrors.CodeAgentExecution, "%s failed to run: %s", id, runErrorMessage(runErr, stderr))
		}
		return apperrors.New(apperrors.CodeAgentExecution, "%s failed to run: %s", id, runErr)
	}
}

func runErrorMessage(runErr error, stderr string) string {
	stderr = strings.TrimSpace(stderr)
	if stderr == "" {
		return runErr.Error()
	}
	stderr = strings.Join(strings.Fields(stderr), " ")
	if len(stderr) > 500 {
		stderr = stderr[:500] + "..."
	}
	return fmt.Sprintf("%s: %s", runErr, stderr)
}

func parseError(id string, err error) error {
	return apperrors.WithHint(
		apperrors.New(apperrors.CodeAgentExecution, "failed to parse translation result from %s", id),
		fmt.Sprintf("%s: %s", "run with --debug to inspect raw output", err),
	)
}
