package locale

import "strings"

type Language struct {
	Code string
	Name string
}

var languages = map[string]Language{
	"ar":                  {Code: "ar", Name: "Arabic"},
	"arabic":              {Code: "ar", Name: "Arabic"},
	"de":                  {Code: "de", Name: "German"},
	"german":              {Code: "de", Name: "German"},
	"en":                  {Code: "en", Name: "English"},
	"eng":                 {Code: "en", Name: "English"},
	"english":             {Code: "en", Name: "English"},
	"英語":                  {Code: "en", Name: "English"},
	"es":                  {Code: "es", Name: "Spanish"},
	"spanish":             {Code: "es", Name: "Spanish"},
	"fr":                  {Code: "fr", Name: "French"},
	"fre":                 {Code: "fr", Name: "French"},
	"french":              {Code: "fr", Name: "French"},
	"日本語":                 {Code: "ja", Name: "Japanese"},
	"ja":                  {Code: "ja", Name: "Japanese"},
	"japanese":            {Code: "ja", Name: "Japanese"},
	"jp":                  {Code: "ja", Name: "Japanese"},
	"it":                  {Code: "it", Name: "Italian"},
	"italian":             {Code: "it", Name: "Italian"},
	"ko":                  {Code: "ko", Name: "Korean"},
	"korean":              {Code: "ko", Name: "Korean"},
	"kr":                  {Code: "ko", Name: "Korean"},
	"pt":                  {Code: "pt", Name: "Portuguese"},
	"portuguese":          {Code: "pt", Name: "Portuguese"},
	"ru":                  {Code: "ru", Name: "Russian"},
	"russian":             {Code: "ru", Name: "Russian"},
	"zh":                  {Code: "zh", Name: "Chinese"},
	"chinese":             {Code: "zh", Name: "Chinese"},
	"zh-cn":               {Code: "zh-CN", Name: "Simplified Chinese"},
	"zh-hans":             {Code: "zh-CN", Name: "Simplified Chinese"},
	"simplified chinese":  {Code: "zh-CN", Name: "Simplified Chinese"},
	"zh-tw":               {Code: "zh-TW", Name: "Traditional Chinese"},
	"zh-hant":             {Code: "zh-TW", Name: "Traditional Chinese"},
	"traditional chinese": {Code: "zh-TW", Name: "Traditional Chinese"},
}

func ResolveLanguage(input string) (Language, bool) {
	key := normalizeLanguageKey(input)
	lang, ok := languages[key]
	return lang, ok
}

func MustLanguage(code string) Language {
	if lang, ok := ResolveLanguage(code); ok {
		return lang
	}
	return Language{Code: code, Name: code}
}

func normalizeLanguageKey(input string) string {
	key := strings.TrimSpace(input)
	key = strings.ReplaceAll(key, "_", "-")
	key = strings.ToLower(key)
	return key
}

func DefaultPairLanguage(local Language) Language {
	if local.Code == "en" {
		return Language{Code: "ja", Name: "Japanese"}
	}
	return Language{Code: "en", Name: "English"}
}
