use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TimelineEvent {
    pub id: String,
    pub label: String,
    #[serde(default)]
    pub description: String,
    pub order: i64,
    #[serde(default)]
    pub chapter_ids: Vec<String>,
    pub created: String,
}

impl TimelineEvent {
    pub fn new(id: String, label: String, order: i64) -> Self {
        Self {
            id,
            label,
            description: String::new(),
            order,
            chapter_ids: Vec::new(),
            created: timestamp_now(),
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
        let e = TimelineEvent {
            id: "te_test".into(),
            label: String::new(),
            description: String::new(),
            order: 0,
            chapter_ids: Vec::new(),
            created: "1234567890".into(),
        };
        let toml = toml::to_string_pretty(&e).unwrap();
        let loaded: TimelineEvent = toml::from_str(&toml).unwrap();
        assert_eq!(e, loaded);
    }

    #[test]
    fn roundtrip_populated() {
        let e = TimelineEvent {
            id: "te_incendio".into(),
            label: "El incendio de la casa".into(),
            description: "La casa arde durante la noche.".into(),
            order: 2018,
            chapter_ids: vec!["ch_007".into()],
            created: "1700000000".into(),
        };
        let toml = toml::to_string_pretty(&e).unwrap();
        let loaded: TimelineEvent = toml::from_str(&toml).unwrap();
        assert_eq!(e, loaded);
        assert_eq!(loaded.order, 2018);
    }

    #[test]
    fn roundtrip_negative_order() {
        let e = TimelineEvent::new("te1".into(), "Antes del incendio".into(), -100);
        let toml = toml::to_string_pretty(&e).unwrap();
        let loaded: TimelineEvent = toml::from_str(&toml).unwrap();
        assert_eq!(loaded.order, -100);
    }

    #[test]
    fn roundtrip_unicode() {
        let e = TimelineEvent::new("id1".into(), "日本語 — ñ".into(), 0);
        let toml = toml::to_string_pretty(&e).unwrap();
        let loaded: TimelineEvent = toml::from_str(&toml).unwrap();
        assert_eq!(e.label, loaded.label);
    }

    #[test]
    fn new_defaults() {
        let e = TimelineEvent::new("id1".into(), "Nacimiento".into(), 1984);
        assert!(e.description.is_empty());
        assert!(e.chapter_ids.is_empty());
        assert_eq!(e.order, 1984);
    }
}