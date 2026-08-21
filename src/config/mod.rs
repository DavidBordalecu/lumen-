use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub tab_width: usize,
    pub language: String,
    pub spellcheck_enabled: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            tab_width: 4,
            language: "auto".into(),
            spellcheck_enabled: true,
        }
    }
}

impl Config {
    pub fn path() -> PathBuf {
        let home = std::env::var("HOME")
            .or_else(|_| std::env::var("USERPROFILE"))
            .unwrap_or_else(|_| String::from("/"));
        PathBuf::from(home).join(".config").join("lumen").join("config.toml")
    }

    pub fn load() -> Self {
        let path = Self::path();
        let Ok(text) = std::fs::read_to_string(&path) else {
            return Config::default();
        };
        
        match toml::from_str::<Config>(&text) {
            Ok(config) => config,
            Err(_) => Config::default(),
        }
    }

    pub fn save(&self) -> bool {
        let path = Self::path();
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        match toml::to_string_pretty(self) {
            Ok(content) => std::fs::write(&path, content).is_ok(),
            Err(_) => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config() {
        let c = Config::default();
        assert_eq!(c.tab_width, 4);
        assert_eq!(c.language, "auto");
        assert!(c.spellcheck_enabled);
    }

    #[test]
    fn serialize_roundtrip() {
        let config = Config {
            tab_width: 8,
            language: "es".to_string(),
            spellcheck_enabled: false,
        };
        
        let toml_str = toml::to_string_pretty(&config).unwrap();
        let loaded: Config = toml::from_str(&toml_str).unwrap();
        
        assert_eq!(loaded.tab_width, 8);
        assert_eq!(loaded.language, "es");
        assert!(!loaded.spellcheck_enabled);
    }
}