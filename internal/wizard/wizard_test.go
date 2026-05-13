package wizard

import (
	"context"
	"os"
	"path/filepath"
	"strings"
	"testing"

	"github.com/potato4d/translate-cli/internal/config"
	apperrors "github.com/potato4d/translate-cli/internal/errors"
)

func TestRunSavesRecommendedToolConfig(t *testing.T) {
	t.Setenv("PATH", fakeToolPath(t, "fake-codex")+string(os.PathListSeparator)+os.Getenv("PATH"))

	path := filepath.Join(t.TempDir(), "config.toml")
	cfg := config.Default()
	cfg.DefaultTool = ""
	cfg.LocalLang = "ja"

	var output strings.Builder
	saved, err := Run(context.Background(), strings.NewReader("\n\n"), &output, Options{
		ConfigPath: path,
		Config:     cfg,
	})
	if err != nil {
		t.Fatal(err)
	}
	if saved.DefaultTool != "codex" {
		t.Fatalf("DefaultTool = %q, want codex", saved.DefaultTool)
	}
	if saved.LocalLang != "ja" {
		t.Fatalf("LocalLang = %q, want ja", saved.LocalLang)
	}
	if !strings.Contains(output.String(), "translate CLI does not store API keys") {
		t.Fatalf("output missing privacy notice: %q", output.String())
	}

	loaded, exists, err := config.Load(path)
	if err != nil {
		t.Fatal(err)
	}
	if !exists {
		t.Fatal("saved config does not exist")
	}
	if loaded.DefaultTool != "codex" || loaded.LocalLang != "ja" {
		t.Fatalf("loaded = %#v", loaded)
	}
}

func TestRunErrorsWhenNoSupportedToolsAreFound(t *testing.T) {
	t.Setenv("PATH", t.TempDir())

	_, err := Run(context.Background(), strings.NewReader(""), &strings.Builder{}, Options{
		ConfigPath: filepath.Join(t.TempDir(), "config.toml"),
		Config:     config.Default(),
	})
	if err == nil {
		t.Fatal("expected error")
	}
	appErr, ok := err.(*apperrors.Error)
	if !ok {
		t.Fatalf("err = %T, want *apperrors.Error", err)
	}
	if appErr.Code != apperrors.CodeToolNotFound {
		t.Fatalf("Code = %d, want %d", appErr.Code, apperrors.CodeToolNotFound)
	}
}

func fakeToolPath(t *testing.T, name string) string {
	t.Helper()
	wd, err := os.Getwd()
	if err != nil {
		t.Fatal(err)
	}
	return filepath.Clean(filepath.Join(wd, "..", "..", "testdata", name))
}
