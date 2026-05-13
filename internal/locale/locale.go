package locale

import (
	"os"
	"strings"
)

func DetectLocalLanguage() Language {
	if lang, ok := ResolveLanguage(os.Getenv("TRANSLATE_CLI_LOCAL_LANG")); ok {
		return lang
	}

	for _, key := range []string{"LC_ALL", "LC_MESSAGES", "LANG"} {
		value := os.Getenv(key)
		if value == "" {
			continue
		}
		if lang, ok := ResolveLanguage(localePrefix(value)); ok {
			return lang
		}
	}

	return Language{Code: "en", Name: "English"}
}

func localePrefix(value string) string {
	value = strings.TrimSpace(value)
	if i := strings.IndexAny(value, ".@"); i >= 0 {
		value = value[:i]
	}
	if i := strings.Index(value, "_"); i >= 0 {
		value = value[:i]
	}
	return value
}
