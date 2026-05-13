package translate

import "github.com/potato4d/translate-cli/internal/locale"

type TranslationMode string

const (
	ModeAutoPair TranslationMode = "auto_pair"
	ModeTarget   TranslationMode = "target"
)

type TranslationRequest struct {
	Text       string
	TargetLang *locale.Language
	LocalLang  locale.Language
	Mode       TranslationMode
	Tool       string
}
