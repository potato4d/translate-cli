package output

import "testing"

func TestNormalizeTranslatedText(t *testing.T) {
	result, err := Normalize(`{"translated_text":"こんにちは"}`)
	if err != nil {
		t.Fatal(err)
	}
	if result.Text != "こんにちは" {
		t.Fatalf("Text = %q", result.Text)
	}
}

func TestNormalizeStructuredOutput(t *testing.T) {
	result, err := Normalize(`{"structured_output":{"translated_text":"こんにちは"}}`)
	if err != nil {
		t.Fatal(err)
	}
	if result.Text != "こんにちは" {
		t.Fatalf("Text = %q", result.Text)
	}
}

func TestNormalizeEmbeddedJSON(t *testing.T) {
	result, err := Normalize("log\n{\"translated_text\":\"こんにちは\"}\n")
	if err != nil {
		t.Fatal(err)
	}
	if result.Text != "こんにちは" {
		t.Fatalf("Text = %q", result.Text)
	}
}

func TestNormalizeBrokenJSON(t *testing.T) {
	if _, err := Normalize(`{"translated_text":`); err == nil {
		t.Fatal("expected error")
	}
}

func TestNormalizeAllowRaw(t *testing.T) {
	result, err := NormalizeAllowRaw("こんにちは\n")
	if err != nil {
		t.Fatal(err)
	}
	if result.Text != "こんにちは" {
		t.Fatalf("Text = %q", result.Text)
	}
}
