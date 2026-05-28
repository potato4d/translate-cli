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

const ISO_639_1_LANGUAGES: &[(&str, &str)] = &[
    ("aa", "Afar"),
    ("ab", "Abkhazian"),
    ("ae", "Avestan"),
    ("af", "Afrikaans"),
    ("ak", "Akan"),
    ("am", "Amharic"),
    ("an", "Aragonese"),
    ("ar", "Arabic"),
    ("as", "Assamese"),
    ("av", "Avaric"),
    ("ay", "Aymara"),
    ("az", "Azerbaijani"),
    ("ba", "Bashkir"),
    ("be", "Belarusian"),
    ("bg", "Bulgarian"),
    ("bi", "Bislama"),
    ("bm", "Bambara"),
    ("bn", "Bengali"),
    ("bo", "Tibetan"),
    ("br", "Breton"),
    ("bs", "Bosnian"),
    ("ca", "Catalan"),
    ("ce", "Chechen"),
    ("ch", "Chamorro"),
    ("co", "Corsican"),
    ("cr", "Cree"),
    ("cs", "Czech"),
    ("cu", "Church Slavic"),
    ("cv", "Chuvash"),
    ("cy", "Welsh"),
    ("da", "Danish"),
    ("de", "German"),
    ("dv", "Divehi"),
    ("dz", "Dzongkha"),
    ("ee", "Ewe"),
    ("el", "Modern Greek (1453-)"),
    ("en", "English"),
    ("eo", "Esperanto"),
    ("es", "Spanish"),
    ("et", "Estonian"),
    ("eu", "Basque"),
    ("fa", "Persian"),
    ("ff", "Fulah"),
    ("fi", "Finnish"),
    ("fj", "Fijian"),
    ("fo", "Faroese"),
    ("fr", "French"),
    ("fy", "Western Frisian"),
    ("ga", "Irish"),
    ("gd", "Gaelic"),
    ("gl", "Galician"),
    ("gn", "Guarani"),
    ("gu", "Gujarati"),
    ("gv", "Manx"),
    ("ha", "Hausa"),
    ("he", "Hebrew"),
    ("hi", "Hindi"),
    ("ho", "Hiri Motu"),
    ("hr", "Croatian"),
    ("ht", "Haitian"),
    ("hu", "Hungarian"),
    ("hy", "Armenian"),
    ("hz", "Herero"),
    (
        "ia",
        "Interlingua (International Auxiliary Language Association)",
    ),
    ("id", "Indonesian"),
    ("ie", "Interlingue"),
    ("ig", "Igbo"),
    ("ii", "Sichuan Yi"),
    ("ik", "Inupiaq"),
    ("io", "Ido"),
    ("is", "Icelandic"),
    ("it", "Italian"),
    ("iu", "Inuktitut"),
    ("ja", "Japanese"),
    ("jv", "Javanese"),
    ("ka", "Georgian"),
    ("kg", "Kongo"),
    ("ki", "Kikuyu"),
    ("kj", "Kuanyama"),
    ("kk", "Kazakh"),
    ("kl", "Kalaallisut"),
    ("km", "Central Khmer"),
    ("kn", "Kannada"),
    ("ko", "Korean"),
    ("kr", "Kanuri"),
    ("ks", "Kashmiri"),
    ("ku", "Kurdish"),
    ("kv", "Komi"),
    ("kw", "Cornish"),
    ("ky", "Kirghiz"),
    ("la", "Latin"),
    ("lb", "Luxembourgish"),
    ("lg", "Ganda"),
    ("li", "Limburgan"),
    ("ln", "Lingala"),
    ("lo", "Lao"),
    ("lt", "Lithuanian"),
    ("lu", "Luba-Katanga"),
    ("lv", "Latvian"),
    ("mg", "Malagasy"),
    ("mh", "Marshallese"),
    ("mi", "Maori"),
    ("mk", "Macedonian"),
    ("ml", "Malayalam"),
    ("mn", "Mongolian"),
    ("mr", "Marathi"),
    ("ms", "Malay (macrolanguage)"),
    ("mt", "Maltese"),
    ("my", "Burmese"),
    ("na", "Nauru"),
    ("nb", "Norwegian Bokmal"),
    ("nd", "North Ndebele"),
    ("ne", "Nepali (macrolanguage)"),
    ("ng", "Ndonga"),
    ("nl", "Dutch"),
    ("nn", "Norwegian Nynorsk"),
    ("no", "Norwegian"),
    ("nr", "South Ndebele"),
    ("nv", "Navajo"),
    ("ny", "Chichewa"),
    ("oc", "Occitan (post 1500)"),
    ("oj", "Ojibwa"),
    ("om", "Oromo"),
    ("or", "Oriya (macrolanguage)"),
    ("os", "Ossetian"),
    ("pa", "Panjabi"),
    ("pi", "Pali"),
    ("pl", "Polish"),
    ("ps", "Pushto"),
    ("pt", "Portuguese"),
    ("qu", "Quechua"),
    ("rm", "Romansh"),
    ("rn", "Rundi"),
    ("ro", "Romanian"),
    ("ru", "Russian"),
    ("rw", "Kinyarwanda"),
    ("sa", "Sanskrit"),
    ("sc", "Sardinian"),
    ("sd", "Sindhi"),
    ("se", "Northern Sami"),
    ("sg", "Sango"),
    ("si", "Sinhala"),
    ("sk", "Slovak"),
    ("sl", "Slovenian"),
    ("sm", "Samoan"),
    ("sn", "Shona"),
    ("so", "Somali"),
    ("sq", "Albanian"),
    ("sr", "Serbian"),
    ("ss", "Swati"),
    ("st", "Sotho, Southern"),
    ("su", "Sundanese"),
    ("sv", "Swedish"),
    ("sw", "Swahili (macrolanguage)"),
    ("ta", "Tamil"),
    ("te", "Telugu"),
    ("tg", "Tajik"),
    ("th", "Thai"),
    ("ti", "Tigrinya"),
    ("tk", "Turkmen"),
    ("tl", "Tagalog"),
    ("tn", "Tswana"),
    ("to", "Tonga (Tonga Islands)"),
    ("tr", "Turkish"),
    ("ts", "Tsonga"),
    ("tt", "Tatar"),
    ("tw", "Twi"),
    ("ty", "Tahitian"),
    ("ug", "Uighur"),
    ("uk", "Ukrainian"),
    ("ur", "Urdu"),
    ("uz", "Uzbek"),
    ("ve", "Venda"),
    ("vi", "Vietnamese"),
    ("vo", "Volapuk"),
    ("wa", "Walloon"),
    ("wo", "Wolof"),
    ("xh", "Xhosa"),
    ("yi", "Yiddish"),
    ("yo", "Yoruba"),
    ("za", "Zhuang"),
    ("zh", "Chinese"),
    ("zu", "Zulu"),
];

pub fn resolve_language(input: &str) -> Option<Language> {
    let key = normalize_language_key(input);
    match key.as_str() {
        "arabic" => language_from_iso_639_1("ar"),
        "german" => language_from_iso_639_1("de"),
        "eng" | "english" | "英語" => language_from_iso_639_1("en"),
        "spanish" => language_from_iso_639_1("es"),
        "fre" | "french" => language_from_iso_639_1("fr"),
        "japanese" | "jp" | "日本語" => language_from_iso_639_1("ja"),
        "italian" => language_from_iso_639_1("it"),
        "korean" | "kr" => language_from_iso_639_1("ko"),
        "kanuri" => language_from_iso_639_1("kr"),
        "portuguese" => language_from_iso_639_1("pt"),
        "russian" => language_from_iso_639_1("ru"),
        "chinese" => language_from_iso_639_1("zh"),
        "zh-cn" | "zh-hans" | "simplified chinese" => {
            Some(Language::new("zh-CN", "Simplified Chinese"))
        }
        "zh-tw" | "zh-hant" | "traditional chinese" => {
            Some(Language::new("zh-TW", "Traditional Chinese"))
        }
        _ => language_from_iso_639_1(&key),
    }
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

fn language_from_iso_639_1(code: &str) -> Option<Language> {
    ISO_639_1_LANGUAGES
        .iter()
        .find(|(candidate, _)| *candidate == code)
        .map(|(code, name)| Language::new(code, name))
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
        assert_eq!(resolve_language("kr").unwrap().code, "ko");
        assert_eq!(resolve_language("kanuri").unwrap().code, "kr");
    }

    #[test]
    fn resolves_iso_639_1_codes() {
        assert_eq!(ISO_639_1_LANGUAGES.len(), 183);
        assert_eq!(resolve_language("AA").unwrap().name, "Afar");
        assert_eq!(resolve_language("tl").unwrap().name, "Tagalog");
        assert_eq!(resolve_language("vo").unwrap().name, "Volapuk");
    }

    #[test]
    fn rejects_unknown_and_deprecated_two_letter_codes() {
        assert!(resolve_language("go").is_none());
        assert!(resolve_language("zz").is_none());
        assert!(resolve_language("in").is_none());
    }
}
