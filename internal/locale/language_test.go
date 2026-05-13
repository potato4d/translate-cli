package locale

import "testing"

func TestResolveLanguage(t *testing.T) {
	tests := map[string]string{
		"ja":      "ja",
		"日本語":     "ja",
		"english": "en",
		"zh-TW":   "zh-TW",
		"zh_Hans": "zh-CN",
		"french":  "fr",
	}

	for input, want := range tests {
		lang, ok := ResolveLanguage(input)
		if !ok {
			t.Fatalf("ResolveLanguage(%q) not ok", input)
		}
		if lang.Code != want {
			t.Fatalf("ResolveLanguage(%q).Code = %q, want %q", input, lang.Code, want)
		}
	}
}

func TestResolveLanguageUnknown(t *testing.T) {
	if _, ok := ResolveLanguage("hello"); ok {
		t.Fatal("expected unknown language")
	}
}
