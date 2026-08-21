use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Concept {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub notes: String,
    pub created: String,
    pub modified: String,
}

impl Concept {
    pub fn new(id: String, name: String) -> Self {
        let now = timestamp_now();
        Self {
            id,
            name,
            description: String::new(),
            notes: String::new(),
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
        let c = Concept {
            id: "co_test".into(),
            name: String::new(),
            description: String::new(),
            notes: String::new(),
            created: "1234567890".into(),
            modified: "1234567890".into(),
        };
        let toml = toml::to_string_pretty(&c).unwrap();
        let loaded: Concept = toml::from_str(&toml).unwrap();
        assert_eq!(c, loaded);
    }

    #[test]
    fn roundtrip_populated() {
        let c = Concept {
            id: "co_secreto".into(),
            name: "El secreto familiar".into(),
            description: "El incendio no fue accidental.".into(),
            notes: "Revelar poco a poco.".into(),
            created: "1700000000".into(),
            modified: "1700100000".into(),
        };
        let toml = toml::to_string_pretty(&c).unwrap();
        let loaded: Concept = toml::from_str(&toml).unwrap();
        assert_eq!(c, loaded);
    }

    #[test]
    fn roundtrip_unicode() {
        let c = Concept::new("id1".into(), "El reloj — ñ".into());
        let toml = toml::to_string_pretty(&c).unwrap();
        let loaded: Concept = toml::from_str(&toml).unwrap();
        assert_eq!(c.name, loaded.name);
    }

    #[test]
    fn new_defaults() {
        let c = Concept::new("id1".into(), "El incendio".into());
        assert!(c.description.is_empty());
        assert!(c.notes.is_empty());
    }
}