pub mod engine;
pub mod personal;

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

pub use engine::SpellcheckEngine;

// ── Detected dictionary ──

#[derive(Debug, Clone)]
pub struct DictInfo {
    pub language: String,
    pub label: String,
    pub aff_path: PathBuf,
    pub dic_path: PathBuf,
}

// ── Dictionary search paths ──

fn system_dict_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    if let Ok(home) = std::env::var("HOME") {
        dirs.push(PathBuf::from(&home).join(".local").join("share").join("hunspell"));
        dirs.push(PathBuf::from(home).join(".local").join("share").join("lumen").join("dictionaries"));
    }
    dirs.push(PathBuf::from("/usr/share/hunspell"));
    dirs.push(PathBuf::from("/usr/share/myspell"));
    dirs.push(PathBuf::from("/usr/share/myspell/dicts"));
    dirs
}

// ── Locale detection ──

fn detect_locale() -> String {
    for var in &["LC_ALL", "LC_MESSAGES", "LANGUAGE", "LANG"] {
        if let Ok(val) = std::env::var(var) {
            if !val.is_empty() && val != "C" && val != "POSIX" {
                let base = val.split('.').next().unwrap_or(&val);
                return base.to_string();
            }
        }
    }
    "en_US".into()
}

fn normalize_lang(locale: &str) -> Vec<String> {
    let mut candidates = Vec::new();
    if !locale.is_empty() {
        candidates.push(locale.to_string());
        if let Some((lang, _region)) = locale.split_once('_') {
            if !candidates.contains(&lang.to_string()) {
                candidates.push(lang.to_string());
            }
        }
    }
    candidates
}

fn language_label(code: &str) -> String {
    match code {
        "es" | "es_ES" | "es_AR" | "es_MX" | "es_CO" | "es_CL" | "es_VE" => "Español".into(),
        "en" | "en_US" | "en_GB" | "en_AU" | "en_CA" => "English".into(),
        "pt" | "pt_BR" | "pt_PT" => "Português".into(),
        "fr" | "fr_FR" | "fr_CA" => "Français".into(),
        "de" | "de_DE" | "de_AT" | "de_CH" => "Deutsch".into(),
        "it" | "it_IT" => "Italiano".into(),
        "nl" | "nl_NL" => "Nederlands".into(),
        "ru" | "ru_RU" => "Русский".into(),
        "ja" | "ja_JP" => "日本語".into(),
        other => other.to_string(),
    }
}

// ── Dictionary scanning ──

fn scan_dir(dir: &Path) -> Vec<(String, PathBuf, PathBuf)> {
    let mut results = Vec::new();
    let Ok(entries) = fs::read_dir(dir) else {
        return results;
    };
    let mut aff_files: HashSet<String> = HashSet::new();
    let mut dic_files: HashSet<String> = HashSet::new();
    let mut aff_paths: HashMap<String, PathBuf> = HashMap::new();
    let mut dic_paths: HashMap<String, PathBuf> = HashMap::new();

    for entry in entries.flatten() {
        let path = entry.path();
        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            if let Some(stem) = name.strip_suffix(".aff") {
                aff_files.insert(stem.to_string());
                aff_paths.insert(stem.to_string(), path.clone());
            }
            if let Some(stem) = name.strip_suffix(".dic") {
                dic_files.insert(stem.to_string());
                dic_paths.insert(stem.to_string(), path);
            }
        }
    }

    for stem in aff_files.intersection(&dic_files) {
        if let (Some(aff), Some(dic)) = (aff_paths.get(stem), dic_paths.get(stem)) {
            results.push((stem.clone(), aff.clone(), dic.clone()));
        }
    }

    results
}

use std::collections::HashMap;

pub fn find_dictionaries() -> Vec<DictInfo> {
    let mut all = Vec::new();
    let mut seen = HashSet::new();
    for dir in system_dict_dirs() {
        for (stem, aff_path, dic_path) in scan_dir(&dir) {
            if seen.insert(stem.clone()) {
                let lang = stem.split('_').next().unwrap_or(&stem).to_string();
                all.push(DictInfo {
                    language: lang,
                    label: language_label(&stem),
                    aff_path,
                    dic_path,
                });
            }
        }
    }
    all
}

pub fn find_available_langs() -> Vec<DictInfo> {
    let dicts = find_dictionaries();
    let mut by_lang: HashMap<String, DictInfo> = HashMap::new();
    for d in dicts {
        by_lang.entry(d.language.clone()).or_insert(d);
    }
    by_lang.into_values().collect()
}

pub fn find_dictionary_for_lang(target: &str) -> Option<DictInfo> {
    let dicts = find_dictionaries();
    let target_lang = target.split('_').next().unwrap_or(target);
    dicts.into_iter().find(|d| {
        d.language == target || d.language == target_lang
    })
}

pub fn auto_detect_dictionary() -> Option<DictInfo> {
    let locale = detect_locale();
    let candidates = normalize_lang(&locale);
    for candidate in &candidates {
        if let Some(d) = find_dictionary_for_lang(candidate) {
            return Some(d);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detect_locale_returns_nonempty() {
        let loc = detect_locale();
        assert!(!loc.is_empty());
    }

    #[test]
    fn normalize_lang_variants() {
        let v = normalize_lang("es_AR");
        assert!(v.contains(&"es_AR".to_string()));
        assert!(v.contains(&"es".to_string()));
    }

    #[test]
    fn language_label_known() {
        assert_eq!(language_label("es"), "Español");
        assert_eq!(language_label("en_US"), "English");
    }

    #[test]
    fn find_dictionaries_does_not_panic() {
        let _ = find_dictionaries();
    }
}
