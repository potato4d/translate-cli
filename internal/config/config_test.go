package config

import (
	"os"
	"path/filepath"
	"testing"
)

func TestSaveLoad(t *testing.T) {
	path := filepath.Join(t.TempDir(), "config.toml")
	cfg := Default()
	cfg.DefaultTool = "codex"
	cfg.LocalLang = "ja"

	if err := Save(path, cfg); err != nil {
		t.Fatal(err)
	}
	loaded, exists, err := Load(path)
	if err != nil {
		t.Fatal(err)
	}
	if !exists {
		t.Fatal("exists = false")
	}
	if loaded.DefaultTool != "codex" || loaded.LocalLang != "ja" {
		t.Fatalf("loaded = %#v", loaded)
	}
	if loaded.Tools["codex"].Enabled != true || loaded.Tools["claude"].Enabled != true {
		t.Fatalf("tools = %#v", loaded.Tools)
	}
}

func TestLoadMissing(t *testing.T) {
	loaded, exists, err := Load(filepath.Join(t.TempDir(), "missing.toml"))
	if err != nil {
		t.Fatal(err)
	}
	if exists {
		t.Fatal("exists = true")
	}
	if loaded.TimeoutMS != DefaultTimeoutMS {
		t.Fatalf("TimeoutMS = %d", loaded.TimeoutMS)
	}
}

func TestLoadBroken(t *testing.T) {
	path := filepath.Join(t.TempDir(), "config.toml")
	if err := os.WriteFile(path, []byte("version = nope\n"), 0600); err != nil {
		t.Fatal(err)
	}
	if _, _, err := Load(path); err == nil {
		t.Fatal("expected parse error")
	}
}
