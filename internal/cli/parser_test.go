package cli

import (
	"testing"

	"github.com/potato4d/translate-cli/internal/translate"
)

func TestParseText(t *testing.T) {
	inv, err := Parse([]string{"hello", "world"})
	if err != nil {
		t.Fatal(err)
	}
	if inv.Mode != translate.ModeAutoPair {
		t.Fatalf("Mode = %q, want %q", inv.Mode, translate.ModeAutoPair)
	}
	if inv.Text != "hello world" {
		t.Fatalf("Text = %q", inv.Text)
	}
	if inv.TargetLang != nil {
		t.Fatalf("TargetLang = %#v, want nil", inv.TargetLang)
	}
}

func TestParseTargetLanguage(t *testing.T) {
	inv, err := Parse([]string{"ja", "Good", "morning"})
	if err != nil {
		t.Fatal(err)
	}
	if inv.Mode != translate.ModeTarget {
		t.Fatalf("Mode = %q, want %q", inv.Mode, translate.ModeTarget)
	}
	if inv.TargetLang == nil || inv.TargetLang.Code != "ja" {
		t.Fatalf("TargetLang = %#v, want ja", inv.TargetLang)
	}
	if inv.Text != "Good morning" {
		t.Fatalf("Text = %q", inv.Text)
	}
}

func TestParseToolOverride(t *testing.T) {
	inv, err := Parse([]string{"--tool", "claude", "ja", "Good morning"})
	if err != nil {
		t.Fatal(err)
	}
	if inv.Tool != "claude" {
		t.Fatalf("Tool = %q", inv.Tool)
	}
	if inv.TargetLang == nil || inv.TargetLang.Code != "ja" {
		t.Fatalf("TargetLang = %#v, want ja", inv.TargetLang)
	}
	if inv.Text != "Good morning" {
		t.Fatalf("Text = %q", inv.Text)
	}
}

func TestParseLanguageOnlyAllowsStdinText(t *testing.T) {
	inv, err := Parse([]string{"fr"})
	if err != nil {
		t.Fatal(err)
	}
	if inv.Mode != translate.ModeTarget {
		t.Fatalf("Mode = %q, want %q", inv.Mode, translate.ModeTarget)
	}
	if inv.Text != "" {
		t.Fatalf("Text = %q, want empty", inv.Text)
	}
}

func TestParseUnknownToolSyntax(t *testing.T) {
	_, err := Parse([]string{"--tool"})
	if err == nil {
		t.Fatal("expected error")
	}
}
