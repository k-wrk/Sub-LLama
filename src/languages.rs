#[derive(Debug, Clone)]
pub struct LanguageDefinition {
    pub code: &'static str,
    pub name: &'static str,
    pub family: &'static str,
    pub hf_repo: &'static str,
    pub ct2_repo: Option<&'static str>,
    pub target_tag: Option<&'static str>,
    pub vlc_suffix: &'static str,
    pub track_title: &'static str,
    pub match_codes: &'static [&'static str],
}

pub const SUPPORTED_LANGUAGES: &[LanguageDefinition] = &[
    // 1. Portuguese (Brazil)
    LanguageDefinition {
        code: "pt",
        name: "Portuguese (Brazil)",
        family: "Romance",
        hf_repo: "Helsinki-NLP/opus-mt-tc-big-en-pt",
        ct2_repo: Some("playto-mt/opus-mt-tc-big-en-pt-ct2"),
        target_tag: None,
        vlc_suffix: "pt-BR",
        track_title: "Português (Brasil)",
        match_codes: &["pt", "por", "pt-br", "pt_br", "pob", "portuguese"],
    },

    // 2. Spanish
    LanguageDefinition {
        code: "es",
        name: "Spanish",
        family: "Romance",
        hf_repo: "Helsinki-NLP/opus-mt-tc-big-en-es",
        ct2_repo: Some("CristianRojaas/opus-mt-en-es-ct2-int8"),
        target_tag: None,
        vlc_suffix: "es",
        track_title: "Español",
        match_codes: &["es", "spa", "es-es", "es-419", "spanish"],
    },

    // 3. French
    LanguageDefinition {
        code: "fr",
        name: "French",
        family: "Romance",
        hf_repo: "Helsinki-NLP/opus-mt-tc-big-en-fr",
        ct2_repo: Some("michaelfeil/ct2fast-opus-mt-en-fr"),
        target_tag: None,
        vlc_suffix: "fr",
        track_title: "Français",
        match_codes: &["fr", "fra", "fre", "french"],
    },

    // 4. German
    LanguageDefinition {
        code: "de",
        name: "German",
        family: "Germanic",
        hf_repo: "Helsinki-NLP/opus-mt-en-de",
        ct2_repo: Some("michaelfeil/ct2fast-opus-mt-en-de"),
        target_tag: None,
        vlc_suffix: "de",
        track_title: "Deutsch",
        match_codes: &["de", "deu", "ger", "german"],
    },

    // 5. Italian
    LanguageDefinition {
        code: "it",
        name: "Italian",
        family: "Romance",
        hf_repo: "Helsinki-NLP/opus-mt-tc-big-en-it",
        ct2_repo: Some("ooeoeo/opus-mt-tc-big-en-it-ct2-float16"),
        target_tag: None,
        vlc_suffix: "it",
        track_title: "Italiano",
        match_codes: &["it", "ita", "italian"],
    },

    // 6. Russian
    LanguageDefinition {
        code: "ru",
        name: "Russian",
        family: "Slavic",
        hf_repo: "Helsinki-NLP/opus-mt-en-ru",
        ct2_repo: Some("ooeoeo/opus-mt-en-ru-ct2-float16"),
        target_tag: None,
        vlc_suffix: "ru",
        track_title: "Русский (Russian)",
        match_codes: &["ru", "rus", "russian"],
    },

    // 7. Chinese (Simplified)
    LanguageDefinition {
        code: "zh",
        name: "Chinese (Simplified)",
        family: "Sino-Tibetan",
        hf_repo: "Helsinki-NLP/opus-mt-en-zh",
        ct2_repo: Some("ooeoeo/opus-mt-en-zh-ct2-float16"),
        target_tag: None,
        vlc_suffix: "zh",
        track_title: "中文 (Chinese)",
        match_codes: &["zh", "zho", "chi", "chinese", "zh-hans", "zh-cn"],
    },

    // 8. Japanese
    LanguageDefinition {
        code: "ja",
        name: "Japanese",
        family: "Japonic",
        hf_repo: "Helsinki-NLP/opus-mt-en-ja",
        ct2_repo: Some("craftwise/ct2-opus-mt-en-ja-int8"),
        target_tag: None,
        vlc_suffix: "ja",
        track_title: "日本語 (Japanese)",
        match_codes: &["ja", "jpn", "japanese"],
    },

    // 9. Korean
    LanguageDefinition {
        code: "ko",
        name: "Korean",
        family: "Koreanic",
        hf_repo: "Helsinki-NLP/opus-mt-tc-big-en-ko",
        ct2_repo: Some("ooeoeo/opus-mt-tc-big-en-ko-ct2-float16"),
        target_tag: None,
        vlc_suffix: "ko",
        track_title: "한국어 (Korean)",
        match_codes: &["ko", "kor", "korean"],
    },

    // 10. Arabic
    LanguageDefinition {
        code: "ar",
        name: "Arabic",
        family: "Semitic",
        hf_repo: "Helsinki-NLP/opus-mt-tc-big-en-ar",
        ct2_repo: Some("ai-path/opus-mt-tc-big-en-ar-ct2"),
        target_tag: None,
        vlc_suffix: "ar",
        track_title: "العربية (Arabic)",
        match_codes: &["ar", "ara", "arabic"],
    },

    // 11. Turkish
    LanguageDefinition {
        code: "tr",
        name: "Turkish",
        family: "Turkic",
        hf_repo: "Helsinki-NLP/opus-mt-tc-big-en-tr",
        ct2_repo: Some("ooeoeo/opus-mt-tc-big-en-tr-ct2-float16"),
        target_tag: None,
        vlc_suffix: "tr",
        track_title: "Türkçe (Turkish)",
        match_codes: &["tr", "tur", "turkish"],
    },

    // 12. Vietnamese
    LanguageDefinition {
        code: "vi",
        name: "Vietnamese",
        family: "Austroasiatic",
        hf_repo: "Helsinki-NLP/opus-mt-en-vi",
        ct2_repo: Some("dekthedev/opus-mt-en-vi-ct2-int8"),
        target_tag: None,
        vlc_suffix: "vi",
        track_title: "Tiếng Việt (Vietnamese)",
        match_codes: &["vi", "vie", "vietnamese"],
    },

    // 13. Indonesian
    LanguageDefinition {
        code: "id",
        name: "Indonesian",
        family: "Austronesian",
        hf_repo: "Helsinki-NLP/opus-mt-en-id",
        ct2_repo: Some("ooeoeo/opus-mt-en-id-ct2-float16"),
        target_tag: None,
        vlc_suffix: "id",
        track_title: "Bahasa Indonesia",
        match_codes: &["id", "ind", "indonesian"],
    },
];

pub fn find_language(query: &str) -> Option<&'static LanguageDefinition> {
    let q = query.trim().to_lowercase();
    SUPPORTED_LANGUAGES.iter().find(|l| {
        l.code.eq_ignore_ascii_case(&q)
            || l.name.to_lowercase() == q
            || l.match_codes.iter().any(|&c| c == q)
    })
}
