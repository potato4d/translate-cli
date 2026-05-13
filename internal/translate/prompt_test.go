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

func TestBuildPlainTextPrompt(t *testing.T) {
	target := locale.MustLanguage("ja")
	prompt := BuildPlainTextPrompt(TranslationRequest{
		Text:       "Good morning",
		TargetLang: &target,
		LocalLang:  locale.MustLanguage("en"),
		Mode:       ModeTarget,
	})
	for _, want := range []string{
		"Return only the translated text.",
		"Translate the text into Japanese.",
		"<text>\nGood morning\n</text>",
	} {
		if !strings.Contains(prompt, want) {
			t.Fatalf("prompt missing %q:\n%s", want, prompt)
		}
	}
	if strings.Contains(prompt, "Return only JSON") {
		t.Fatalf("plain text prompt should not request JSON:\n%s", prompt)
	}
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
