package cli

import (
	"bytes"
	"os"
	"path/filepath"
	"strings"
	"testing"

	"github.com/potato4d/translate-cli/internal/config"
)

func TestMainVersion(t *testing.T) {
	var stdout bytes.Buffer
	var stderr bytes.Buffer
	code := Main([]string{"--version"}, strings.NewReader(""), &stdout, &stderr)
	if code != 0 {
		t.Fatalf("code = %d, stderr = %q", code, stderr.String())
	}
	if got, want := stdout.String(), "translate-cli 0.1.0\n"; got != want {
		t.Fatalf("stdout = %q, want %q", got, want)
	}
}

func TestMainWithFakeCodex(t *testing.T) {
	tmp := t.TempDir()
	configPath := filepath.Join(tmp, "config.toml")
	cfg := config.Default()
	cfg.DefaultTool = "codex"
	cfg.LocalLang = "ja"
	if err := config.Save(configPath, cfg); err != nil {
		t.Fatal(err)
	}

	t.Setenv("TRANSLATE_CLI_CONFIG", configPath)
	t.Setenv("PATH", fakePath(t, "fake-codex")+string(os.PathListSeparator)+os.Getenv("PATH"))

	var stdout bytes.Buffer
	var stderr bytes.Buffer
	code := Main([]string{"ja", "hello"}, strings.NewReader(""), &stdout, &stderr)
	if code != 0 {
		t.Fatalf("code = %d, stderr = %q", code, stderr.String())
	}
	if got, want := stdout.String(), "こんにちは\n"; got != want {
		t.Fatalf("stdout = %q, want %q", got, want)
	}
}

func TestMainWithFakeClaudeFromStdin(t *testing.T) {
	tmp := t.TempDir()
	configPath := filepath.Join(tmp, "config.toml")
	cfg := config.Default()
	cfg.DefaultTool = "claude"
	cfg.LocalLang = "ja"
	if err := config.Save(configPath, cfg); err != nil {
		t.Fatal(err)
	}

	t.Setenv("TRANSLATE_CLI_CONFIG", configPath)
	t.Setenv("PATH", fakePath(t, "fake-claude")+string(os.PathListSeparator)+os.Getenv("PATH"))

	var stdout bytes.Buffer
	var stderr bytes.Buffer
	code := Main([]string{"--tool", "claude", "ja"}, strings.NewReader("hello"), &stdout, &stderr)
	if code != 0 {
		t.Fatalf("code = %d, stderr = %q", code, stderr.String())
	}
	if got, want := stdout.String(), "こんにちは\n"; got != want {
		t.Fatalf("stdout = %q, want %q", got, want)
	}
}

func fakePath(t *testing.T, name string) string {
	t.Helper()
	wd, err := os.Getwd()
	if err != nil {
		t.Fatal(err)
	}
	return filepath.Clean(filepath.Join(wd, "..", "..", "testdata", name))
}
