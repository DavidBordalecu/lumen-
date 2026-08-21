use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Character {
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

impl Character {
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
        let c = Character {
            id: "ch_test_001".into(),
            name: String::new(),
            description: String::new(),
            notes: String::new(),
            chapter_ids: Vec::new(),
            created: "1234567890".into(),
            modified: "1234567890".into(),
        };
        let toml = toml::to_string_pretty(&c).unwrap();
        let loaded: Character = toml::from_str(&toml).unwrap();
        assert_eq!(c, loaded);
    }

    #[test]
    fn roundtrip_populated() {
        let c = Character {
            id: "ch_a1b2c3".into(),
            name: "Martín".into(),
            description: "Un hombre de 42 años.\nVive solo desde la muerte de su esposa.".into(),
            notes: "Personaje principal.".into(),
            chapter_ids: vec!["ch_001".into(), "ch_003".into(), "ch_007".into()],
            created: "1700000000".into(),
            modified: "1700100000".into(),
        };
        let toml = toml::to_string_pretty(&c).unwrap();
        let loaded: Character = toml::from_str(&toml).unwrap();
        assert_eq!(c, loaded);
        assert_eq!(loaded.name, "Martín");
        assert_eq!(loaded.chapter_ids.len(), 3);
    }

    #[test]
    fn roundtrip_unicode() {
        let c = Character::new("id_日本語".into(), "Ñoño — 日本語".into());
        let toml = toml::to_string_pretty(&c).unwrap();
        let loaded: Character = toml::from_str(&toml).unwrap();
        assert_eq!(c.name, loaded.name);
        assert_eq!(c.id, loaded.id);
    }

    #[test]
    fn new_has_timestamps() {
        let c = Character::new("id1".into(), "Test".into());
        assert!(!c.created.is_empty());
        assert_eq!(c.created, c.modified);
        assert!(c.chapter_ids.is_empty());
    }

    #[test]
    fn chapter_ids_persist() {
        let c = Character {
            id: "id1".into(),
            name: "Elena".into(),
            description: String::new(),
            notes: String::new(),
            chapter_ids: vec!["ch_a".into(), "ch_b".into()],
            created: "100".into(),
            modified: "100".into(),
        };
        let toml = toml::to_string_pretty(&c).unwrap();
        assert!(toml.contains("ch_a"));
        assert!(toml.contains("ch_b"));
        let loaded: Character = toml::from_str(&toml).unwrap();
        assert_eq!(loaded.chapter_ids, vec!["ch_a", "ch_b"]);
    }
}