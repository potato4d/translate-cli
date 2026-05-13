package translate

import (
	"os"
	"path/filepath"
	"strings"
	"testing"

	"github.com/potato4d/translate-cli/internal/locale"
)

func TestBuildPromptAutoPairSnapshot(t *testing.T) {
	prompt := BuildPrompt(TranslationRequest{
		Text:      "こんにちは",
		LocalLang: locale.MustLanguage("ja"),
		Mode:      ModeAutoPair,
	})
	assertSnapshot(t, "prompt_auto_pair.snap", prompt)
}

func TestBuildPromptTargetSnapshot(t *testing.T) {
	target := locale.MustLanguage("fr")
	prompt := BuildPrompt(TranslationRequest{
		Text:       "Good morning",
		TargetLang: &target,
		LocalLang:  locale.MustLanguage("ja"),
		Mode:       ModeTarget,
	})
	assertSnapshot(t, "prompt_target.snap", prompt)
}

func assertSnapshot(t *testing.T, name string, got string) {
	t.Helper()
	path := filepath.Join("__snapshots__", name)
	data, err := os.ReadFile(path)
	if err != nil {
		t.Fatal(err)
	}
	want := strings.TrimSuffix(string(data), "\n")
	if got != want {
		t.Fatalf("snapshot %s mismatch\n--- got ---\n%s\n--- want ---\n%s", name, got, want)
	}
}
