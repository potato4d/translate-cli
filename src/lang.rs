#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Language {
    pub code: String,
    pub name: String,
}

impl Language {
    fn new(code: &str, name: &str) -> Self {
        Self {
            code: code.to_string(),
            name: name.to_string(),
        }
    }
}

pub fn resolve_language(input: &str) -> Option<Language> {
    let key = normalize_language_key(input);
    let (code, name) = match key.as_str() {
        "ar" | "arabic" => ("ar", "Arabic"),
        "de" | "german" => ("de", "German"),
        "en" | "eng" | "english" | "英語" => ("en", "English"),
        "es" | "spanish" => ("es", "Spanish"),
        "fr" | "fre" | "french" => ("fr", "French"),
        "ja" | "japanese" | "jp" | "日本語" => ("ja", "Japanese"),
        "it" | "italian" => ("it", "Italian"),
        "ko" | "korean" | "kr" => ("ko", "Korean"),
        "pt" | "portuguese" => ("pt", "Portuguese"),
        "ru" | "russian" => ("ru", "Russian"),
        "zh" | "chinese" => ("zh", "Chinese"),
        "zh-cn" | "zh-hans" | "simplified chinese" => ("zh-CN", "Simplified Chinese"),
        "zh-tw" | "zh-hant" | "traditional chinese" => ("zh-TW", "Traditional Chinese"),
        _ => return None,
    };
    Some(Language::new(code, name))
}

pub fn must_language(code: &str) -> Language {
    resolve_language(code).unwrap_or_else(|| Language::new(code, code))
}

pub fn detect_local_language() -> Language {
    if let Some(lang) = std::env::var("TRANSLATE_CLI_LOCAL_LANG")
        .ok()
        .and_then(|value| resolve_language(&value))
    {
        return lang;
    }

    for key in ["LC_ALL", "LC_MESSAGES", "LANG"] {
        if let Some(lang) = std::env::var(key)
            .ok()
            .and_then(|value| resolve_language(&locale_prefix(&value)))
        {
            return lang;
        }
    }

    Language::new("en", "English")
}

pub fn default_pair_language(local: &Language) -> Language {
    if local.code == "en" {
        Language::new("ja", "Japanese")
    } else {
        Language::new("en", "English")
    }
}

fn normalize_language_key(input: &str) -> String {
    input.trim().replace('_', "-").to_lowercase()
}

fn locale_prefix(value: &str) -> String {
    let mut prefix = value.trim();
    if let Some(index) = prefix.find(['.', '@']) {
        prefix = &prefix[..index];
    }
    if let Some(index) = prefix.find('_') {
        prefix = &prefix[..index];
    }
    prefix.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_common_aliases() {
        assert_eq!(resolve_language("ja").unwrap().name, "Japanese");
        assert_eq!(resolve_language("日本語").unwrap().code, "ja");
        assert_eq!(
            resolve_language("zh_TW").unwrap().name,
            "Traditional Chinese"
        );
        assert_eq!(resolve_language("english").unwrap().code, "en");
    }
}
