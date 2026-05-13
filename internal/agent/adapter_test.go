package agent

import (
	"strings"
	"testing"

	"github.com/potato4d/translate-cli/internal/locale"
	"github.com/potato4d/translate-cli/internal/translate"
)

func TestCodexBuildCommand(t *testing.T) {
	req := request()
	spec := CodexAdapter{}.BuildCommand(req, RuntimeContext{
		SchemaPath:      "/tmp/schema.json",
		LastMessagePath: "/tmp/out.json",
	})
	args := strings.Join(spec.Args, " ")
	for _, want := range []string{
		"gpt-5.3-codex-spark",
		`model_reasoning_effort="low"`,
		"exec",
		"--skip-git-repo-check",
		"--ignore-user-config",
		"--ignore-rules",
		"--ephemeral",
		"--sandbox read-only",
		"--ask-for-approval never",
		"--color never",
		"--output-schema /tmp/schema.json",
		"--output-last-message /tmp/out.json",
	} {
		if !strings.Contains(args, want) {
			t.Fatalf("args = %q, missing %q", args, want)
		}
	}
	if !strings.Contains(spec.Stdin, "You are a translation engine.") {
		t.Fatalf("stdin = %q", spec.Stdin)
	}
}

func TestClaudeBuildCommand(t *testing.T) {
	spec := ClaudeAdapter{}.BuildCommand(request(), RuntimeContext{})
	args := strings.Join(spec.Args, " ")
	for _, want := range []string{
		"--bare",
		"-p",
		"--output-format json",
		"--json-schema",
		"--no-session-persistence",
		"--max-turns 1",
		"--tools",
	} {
		if !strings.Contains(args, want) {
			t.Fatalf("args = %q, missing %q", args, want)
		}
	}
}

func TestExtractResult(t *testing.T) {
	result, err := CodexAdapter{}.ExtractResult(ExecResult{LastMessageText: `{"translated_text":"こんにちは"}`})
	if err != nil {
		t.Fatal(err)
	}
	if result.Text != "こんにちは" {
		t.Fatalf("Text = %q", result.Text)
	}
}

func request() translate.TranslationRequest {
	target := locale.MustLanguage("ja")
	return translate.TranslationRequest{
		Text:       "hello",
		TargetLang: &target,
		LocalLang:  locale.MustLanguage("en"),
		Mode:       translate.ModeTarget,
	}
}
