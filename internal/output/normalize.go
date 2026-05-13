package output

import (
	"encoding/json"
	"errors"
	"strings"

	"github.com/potato4d/translate-cli/internal/translate"
)

var ErrNoTranslation = errors.New("no translated_text in output")

func Normalize(raw string) (translate.TranslationResult, error) {
	return normalize(raw, false)
}

func NormalizeAllowRaw(raw string) (translate.TranslationResult, error) {
	return normalize(raw, true)
}

func normalize(raw string, allowRaw bool) (translate.TranslationResult, error) {
	trimmed := strings.TrimSpace(raw)
	if trimmed == "" {
		return translate.TranslationResult{}, ErrNoTranslation
	}

	if result, ok := parseJSONTranslation(trimmed); ok {
		return result, nil
	}
	if result, ok := parseEmbeddedJSON(trimmed); ok {
		return result, nil
	}
	if allowRaw {
		return translate.TranslationResult{Text: trimmed}, nil
	}
	return translate.TranslationResult{}, ErrNoTranslation
}

func parseEmbeddedJSON(raw string) (translate.TranslationResult, bool) {
	for start := strings.LastIndex(raw, "{"); start >= 0; start = strings.LastIndex(raw[:start], "{") {
		for end := strings.LastIndex(raw, "}"); end > start; end = strings.LastIndex(raw[:end], "}") {
			if result, ok := parseJSONTranslation(raw[start : end+1]); ok {
				return result, true
			}
		}
	}
	return translate.TranslationResult{}, false
}

func parseJSONTranslation(raw string) (translate.TranslationResult, bool) {
	var body map[string]json.RawMessage
	if err := json.Unmarshal([]byte(raw), &body); err != nil {
		return translate.TranslationResult{}, false
	}

	if text, ok := stringField(body, "translated_text"); ok {
		return translate.TranslationResult{Text: text}, true
	}

	if nested, ok := body["structured_output"]; ok {
		var structured map[string]json.RawMessage
		if json.Unmarshal(nested, &structured) == nil {
			if text, ok := stringField(structured, "translated_text"); ok {
				return translate.TranslationResult{Text: text}, true
			}
		}
	}

	if result, ok := stringField(body, "result"); ok {
		if nested, nestedOK := parseJSONTranslation(result); nestedOK {
			return nested, true
		}
		if strings.TrimSpace(result) != "" {
			return translate.TranslationResult{Text: strings.TrimSpace(result)}, true
		}
	}

	return translate.TranslationResult{}, false
}

func stringField(body map[string]json.RawMessage, key string) (string, bool) {
	raw, ok := body[key]
	if !ok {
		return "", false
	}
	var value string
	if err := json.Unmarshal(raw, &value); err != nil {
		return "", false
	}
	return value, true
}
