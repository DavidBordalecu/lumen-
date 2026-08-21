use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Place {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub notes: String,
    #[serde(default)]
    pub chapter_ids: Vec<String>,
    pub created: String,
    pub modified: String,
}

impl Place {
    pub fn new(id: String, name: String) -> Self {
        let now = timestamp_now();
        Self {
            id,
            name,
            description: String::new(),
            notes: String::new(),
            chapter_ids: Vec::new(),
            created: now.clone(),
            modified: now,
        }
    }
}

fn timestamp_now() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    format!("{}", secs)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_empty() {
        let p = Place {
            id: "pl_test_001".into(),
            name: String::new(),
            description: String::new(),
            notes: String::new(),
            chapter_ids: Vec::new(),
            created: "1234567890".into(),
            modified: "1234567890".into(),
        };
        let toml = toml::to_string_pretty(&p).unwrap();
        let loaded: Place = toml::from_str(&toml).unwrap();
        assert_eq!(p, loaded);
    }

    #[test]
    fn roundtrip_populated() {
        let p = Place {
            id: "pl_casa01".into(),
            name: "La casa abandonada".into(),
            description: "Casa antigua en las afueras de Mendoza.".into(),
            notes: "Tiene un jardín abandonado.".into(),
            chapter_ids: vec!["ch_001".into(), "ch_005".into()],
            created: "1700000000".into(),
            modified: "1700100000".into(),
        };
        let toml = toml::to_string_pretty(&p).unwrap();
        let loaded: Place = toml::from_str(&toml).unwrap();
        assert_eq!(p, loaded);
        assert_eq!(loaded.name, "La casa abandonada");
    }

    #[test]
    fn roundtrip_unicode() {
        let p = Place::new("id_lugar".into(), "Café — Ñoño".into());
        let toml = toml::to_string_pretty(&p).unwrap();
        let loaded: Place = toml::from_str(&toml).unwrap();
        assert_eq!(p.name, loaded.name);
    }

    #[test]
    fn new_defaults() {
        let p = Place::new("id1".into(), "Mendoza".into());
        assert!(p.description.is_empty());
        assert!(p.notes.is_empty());
        assert!(p.chapter_ids.is_empty());
    }
}