//! The window and the terminal speak the player's language. Every string lives in an XML table
//! under `lang/`, one per language, and the tables are the same for both front ends.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};

use crate::portable::Portable;

/// The languages the PiBOH projects ship, as `(code, native name)`. No language is left out: a new
/// one is a new `lang/<code>.xml` and a line here.
pub const LANGUAGES: &[(&str, &str)] = &[
    ("en", "English"),
    ("it", "Italiano"),
    ("de", "Deutsch"),
    ("fr", "Français"),
    ("es", "Español"),
    ("pt", "Português"),
    ("nl", "Nederlands"),
    ("pl", "Polski"),
    ("cs", "Čeština"),
    ("sv", "Svenska"),
    ("ru", "Русский"),
    ("uk", "Українська"),
    ("tr", "Türkçe"),
    ("ar", "العربية"),
    ("hi", "हिन्दी"),
    ("zh", "中文"),
    ("ja", "日本語"),
    ("ko", "한국어"),
];

const TABLES: &[(&str, &str)] = &[
    ("en", include_str!("../lang/en.xml")),
    ("it", include_str!("../lang/it.xml")),
    ("de", include_str!("../lang/de.xml")),
    ("fr", include_str!("../lang/fr.xml")),
    ("es", include_str!("../lang/es.xml")),
    ("pt", include_str!("../lang/pt.xml")),
    ("nl", include_str!("../lang/nl.xml")),
    ("pl", include_str!("../lang/pl.xml")),
    ("cs", include_str!("../lang/cs.xml")),
    ("sv", include_str!("../lang/sv.xml")),
    ("ru", include_str!("../lang/ru.xml")),
    ("uk", include_str!("../lang/uk.xml")),
    ("tr", include_str!("../lang/tr.xml")),
    ("ar", include_str!("../lang/ar.xml")),
    ("hi", include_str!("../lang/hi.xml")),
    ("zh", include_str!("../lang/zh.xml")),
    ("ja", include_str!("../lang/ja.xml")),
    ("ko", include_str!("../lang/ko.xml")),
];

static CURRENT: Mutex<Option<HashMap<String, String>>> = Mutex::new(None);
static ENGLISH: OnceLock<HashMap<String, String>> = OnceLock::new();

/// The table of one language, read out of its XML file.
pub fn parse(xml: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();
    let mut rest = xml;
    while let Some(start) = rest.find("<string key=") {
        rest = &rest[start + "<string key=".len()..];
        let Some(quote) = rest.find('"') else { break };
        rest = &rest[quote + 1..];
        let Some(end_key) = rest.find('"') else { break };
        let key = rest[..end_key].to_string();
        rest = &rest[end_key + 1..];
        let Some(open) = rest.find('>') else { break };
        rest = &rest[open + 1..];
        let Some(close) = rest.find("</string>") else { break };
        let value = unescape(rest[..close].trim());
        rest = &rest[close + "</string>".len()..];
        if !key.is_empty() {
            map.insert(key, value);
        }
    }
    map
}

fn unescape(text: &str) -> String {
    text.replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
        .replace("&amp;", "&")
}

fn english() -> &'static HashMap<String, String> {
    ENGLISH.get_or_init(|| {
        TABLES
            .iter()
            .find(|(code, _)| *code == "en")
            .map(|(_, xml)| parse(xml))
            .unwrap_or_default()
    })
}

/// Picks the table in use. An unknown code falls back to English.
pub fn init(code: &str) {
    let table = TABLES
        .iter()
        .find(|(known, _)| known.eq_ignore_ascii_case(code))
        .map(|(_, xml)| parse(xml))
        .unwrap_or_else(|| english().clone());
    if let Ok(mut current) = CURRENT.lock() {
        *current = Some(table);
    }
}

/// A string, in the language in use, falling back to English and then to the key itself.
pub fn t(key: &str) -> String {
    if let Ok(current) = CURRENT.lock() {
        if let Some(map) = current.as_ref() {
            if let Some(text) = map.get(key) {
                return text.clone();
            }
        }
    }
    english()
        .get(key)
        .cloned()
        .unwrap_or_else(|| key.to_string())
}

/// The same, with `{name}` placeholders filled in.
pub fn tf(key: &str, args: &[(&str, String)]) -> String {
    let mut text = t(key);
    for (name, value) in args {
        text = text.replace(&format!("{{{name}}}"), value);
    }
    text
}

/// The language the player chose, `OFFHORIZON_LANG`, the system's, or English.
pub fn resolve(portable: &Portable) -> String {
    if let Some(chosen) = std::env::var_os("OFFHORIZON_LANG").filter(|v| !v.is_empty()) {
        return normalise(&chosen.to_string_lossy());
    }
    if let Some(chosen) = saved(portable) {
        return chosen;
    }
    for name in ["LC_ALL", "LC_MESSAGES", "LANG"] {
        if let Some(value) = std::env::var_os(name).filter(|v| !v.is_empty()) {
            let value = value.to_string_lossy().to_string();
            let code = value.split(['.', '@']).next().unwrap_or("en");
            return normalise(code);
        }
    }
    "en".into()
}

/// `it_IT.UTF-8`, `IT`, `pt_BR` → `it`, `it`, `pt`.
fn normalise(raw: &str) -> String {
    let lower = raw.trim().to_lowercase();
    let short: String = lower
        .split(['_', '-', '.'])
        .next()
        .unwrap_or("en")
        .chars()
        .take(2)
        .collect();
    if LANGUAGES.iter().any(|(code, _)| *code == short) {
        short
    } else {
        "en".into()
    }
}

/// The language this portable copy last ran in.
pub fn saved(portable: &Portable) -> Option<String> {
    portable.get("lang").map(|code| normalise(&code))
}

/// Keeps the language with the stick. The tool's own settings file is the one file written
/// without a copy: it holds nothing a player could lose.
pub fn save(portable: &Portable, code: &str) -> Result<PathBuf, String> {
    portable.set("lang", code)
}

/// Reads a table, for the CLI's `lang --table` and the tests.
pub fn table(code: &str) -> HashMap<String, String> {
    TABLES
        .iter()
        .find(|(known, _)| known.eq_ignore_ascii_case(code))
        .map(|(_, xml)| parse(xml))
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_language_carries_every_key_the_english_table_has() {
        let english = table("en");
        assert!(!english.is_empty());
        for (code, _) in LANGUAGES {
            let other = table(code);
            let missing: Vec<&String> = english
                .keys()
                .filter(|key| !other.contains_key(*key))
                .collect();
            assert!(missing.is_empty(), "{code} misses {missing:?}");
        }
    }

    #[test]
    fn a_system_locale_name_becomes_a_language_code() {
        assert_eq!(normalise("it_IT.UTF-8"), "it");
        assert_eq!(normalise("pt-BR"), "pt");
        assert_eq!(normalise("xx_XX"), "en");
    }

    #[test]
    fn the_chosen_language_is_the_one_the_window_speaks() {
        init("it");
        assert_eq!(t("nav.worlds"), table("it")["nav.worlds"]);
        init("en");
        assert_eq!(t("nav.worlds"), "Worlds");
    }

    #[test]
    fn a_placeholder_takes_the_value_it_is_given() {
        init("en");
        assert!(tf("status.world_loaded", &[("name", "Test".into())]).contains("Test"));
    }
}
