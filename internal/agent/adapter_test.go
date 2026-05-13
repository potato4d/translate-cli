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
		"include_permissions_instructions=false",
		"include_apps_instructions=false",
		"include_environment_context=false",
		"include_apply_patch_tool=false",
		"exec",
		"--skip-git-repo-check",
		"--ignore-user-config",
		"--ignore-rules",
		"--ephemeral",
		"--sandbox read-only",
		"--ask-for-approval never",
		"--color never",
		"--json",
	} {
		if !strings.Contains(args, want) {
			t.Fatalf("args = %q, missing %q", args, want)
		}
	}
	if strings.Contains(args, "--output-schema") {
		t.Fatalf("args = %q, must not use output schema in fast Codex path", args)
	}
	if !spec.AllowRaw {
		t.Fatal("AllowRaw = false, want true")
	}
	if !spec.StreamJSON {
		t.Fatal("StreamJSON = false, want true")
	}
	if spec.WorkDir != "" {
		t.Fatalf("WorkDir = %q, want empty when runtime does not provide one", spec.WorkDir)
	}
	if spec.Stdin != "" {
		t.Fatalf("Stdin = %q, want empty for prompt-arg Codex fast path", spec.Stdin)
	}
	if !strings.Contains(args, "Return only the translated text.") {
		t.Fatalf("args = %q, missing prompt", args)
	}
}

func TestCodexBuildCommandFallsBackToStdinForLargePrompt(t *testing.T) {
	req := request()
	req.Text = strings.Repeat("x", maxCodexPromptArgBytes+1)

	spec := CodexAdapter{}.BuildCommand(req, RuntimeContext{})
	args := strings.Join(spec.Args, " ")

	if !strings.HasSuffix(args, " -") {
		t.Fatalf("args = %q, want stdin marker for large prompt", args)
	}
	if !strings.Contains(spec.Stdin, "Return only the translated text.") {
		t.Fatalf("stdin = %q", spec.Stdin)
	}
}

func TestCodexBuildCommandUsesRuntimeWorkDir(t *testing.T) {
	spec := CodexAdapter{}.BuildCommand(request(), RuntimeContext{WorkDir: "/tmp"})
	if spec.WorkDir != "/tmp" {
		t.Fatalf("WorkDir = %q", spec.WorkDir)
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
