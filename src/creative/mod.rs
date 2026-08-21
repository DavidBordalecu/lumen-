pub mod character;
pub mod concept;
pub mod place;
pub mod timeline;

pub use character::Character;
pub use concept::Concept;
pub use place::Place;
pub use timeline::TimelineEvent;

use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::path::Path;

pub const CREATIVE_FILE: &str = "creative.toml";

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct CreativeContext {
    #[serde(default)]
    pub characters: Vec<Character>,
    #[serde(default)]
    pub places: Vec<Place>,
    #[serde(default)]
    pub concepts: Vec<Concept>,
    #[serde(default)]
    pub timeline: Vec<TimelineEvent>,
}

impl CreativeContext {
    pub fn new() -> Self {
        Self::default()
    }

    /// Save creative context to a TOML file.
    /// Uses atomic write (write to .tmp then rename) to prevent corruption.
    pub fn save(&self, path: &Path) -> io::Result<()> {
        let content = toml::to_string_pretty(self)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let tmp = path.with_extension("toml.tmp");
        fs::write(&tmp, &content)?;
        fs::rename(&tmp, path)?;
        Ok(())
    }

    /// Load creative context from a TOML file.
    /// Returns empty context if the file does not exist.
    pub fn load(path: &Path) -> io::Result<Self> {
        if !path.exists() {
            return Ok(Self::default());
        }
        let content = fs::read_to_string(path)?;
        let ctx: CreativeContext = toml::from_str(&content)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        Ok(ctx)
    }

    pub fn character_count(&self) -> usize {
        self.characters.len()
    }

    pub fn place_count(&self) -> usize {
        self.places.len()
    }

    pub fn concept_count(&self) -> usize {
        self.concepts.len()
    }

    pub fn event_count(&self) -> usize {
        self.timeline.len()
    }

    pub fn find_character(&self, id: &str) -> Option<&Character> {
        self.characters.iter().find(|c| c.id == id)
    }

    pub fn find_character_mut(&mut self, id: &str) -> Option<&mut Character> {
        self.characters.iter_mut().find(|c| c.id == id)
    }

    pub fn find_place(&self, id: &str) -> Option<&Place> {
        self.places.iter().find(|p| p.id == id)
    }

    pub fn find_place_mut(&mut self, id: &str) -> Option<&mut Place> {
        self.places.iter_mut().find(|p| p.id == id)
    }

    pub fn find_concept(&self, id: &str) -> Option<&Concept> {
        self.concepts.iter().find(|c| c.id == id)
    }

    pub fn find_concept_mut(&mut self, id: &str) -> Option<&mut Concept> {
        self.concepts.iter_mut().find(|c| c.id == id)
    }

    pub fn find_event(&self, id: &str) -> Option<&TimelineEvent> {
        self.timeline.iter().find(|e| e.id == id)
    }

    pub fn find_event_mut(&mut self, id: &str) -> Option<&mut TimelineEvent> {
        self.timeline.iter_mut().find(|e| e.id == id)
    }

    /// Remove all associations referencing a deleted chapter ID
    pub fn remove_chapter_associations(&mut self, chapter_id: &str) {
        for ch in &mut self.characters {
            ch.chapter_ids.retain(|id| id != chapter_id);
        }
        for pl in &mut self.places {
            pl.chapter_ids.retain(|id| id != chapter_id);
        }
        for ev in &mut self.timeline {
            ev.chapter_ids.retain(|id| id != chapter_id);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_empty() {
        let ctx = CreativeContext::default();
        assert!(ctx.characters.is_empty());
        assert!(ctx.places.is_empty());
        assert!(ctx.concepts.is_empty());
        assert!(ctx.timeline.is_empty());
    }

    #[test]
    fn roundtrip_empty() {
        let ctx = CreativeContext::new();
        let toml = toml::to_string_pretty(&ctx).unwrap();
        let loaded: CreativeContext = toml::from_str(&toml).unwrap();
        assert_eq!(ctx, loaded);
    }

    #[test]
    fn roundtrip_populated() {
        let ctx = CreativeContext {
            characters: vec![
                Character::new("ch1".into(), "Martín".into()),
                Character::new("ch2".into(), "Elena".into()),
            ],
            places: vec![
                Place::new("pl1".into(), "La casa".into()),
            ],
            concepts: vec![
                Concept::new("co1".into(), "El secreto".into()),
            ],
            timeline: vec![
                TimelineEvent::new("te1".into(), "1984".into(), 1984),
                TimelineEvent::new("te2".into(), "2024".into(), 2024),
            ],
        };
        let toml = toml::to_string_pretty(&ctx).unwrap();
        let loaded: CreativeContext = toml::from_str(&toml).unwrap();
        assert_eq!(ctx, loaded);
        assert_eq!(loaded.characters.len(), 2);
        assert_eq!(loaded.places.len(), 1);
        assert_eq!(loaded.concepts.len(), 1);
        assert_eq!(loaded.timeline.len(), 2);
    }

    #[test]
    fn roundtrip_unicode() {
        let ctx = CreativeContext {
            characters: vec![
                Character::new("id1".into(), "Ñoño — 日本語".into()),
            ],
            places: vec![
                Place::new("id2".into(), "Café Ñ".into()),
            ],
            concepts: vec![
                Concept::new("id3".into(), "Übermensch — ñ".into()),
            ],
            timeline: vec![
                TimelineEvent::new("id4".into(), "Año nuevo — 日本語".into(), 0),
            ],
        };
        let toml = toml::to_string_pretty(&ctx).unwrap();
        let loaded: CreativeContext = toml::from_str(&toml).unwrap();
        assert_eq!(ctx, loaded);
    }

    #[test]
    fn count_methods() {
        let mut ctx = CreativeContext::new();
        assert_eq!(ctx.character_count(), 0);
        assert_eq!(ctx.place_count(), 0);
        assert_eq!(ctx.concept_count(), 0);
        assert_eq!(ctx.event_count(), 0);

        ctx.characters.push(Character::new("c1".into(), "A".into()));
        ctx.characters.push(Character::new("c2".into(), "B".into()));
        ctx.places.push(Place::new("p1".into(), "X".into()));
        ctx.concepts.push(Concept::new("k1".into(), "Y".into()));
        ctx.timeline.push(TimelineEvent::new("t1".into(), "Z".into(), 1));

        assert_eq!(ctx.character_count(), 2);
        assert_eq!(ctx.place_count(), 1);
        assert_eq!(ctx.concept_count(), 1);
        assert_eq!(ctx.event_count(), 1);
    }

    #[test]
    fn find_methods() {
        let ctx = CreativeContext {
            characters: vec![Character::new("ch1".into(), "Martín".into())],
            places: vec![Place::new("pl1".into(), "Casa".into())],
            concepts: vec![Concept::new("co1".into(), "Secreto".into())],
            timeline: vec![TimelineEvent::new("te1".into(), "Incendio".into(), 1)],
        };

        assert!(ctx.find_character("ch1").is_some());
        assert!(ctx.find_character("nope").is_none());
        assert!(ctx.find_place("pl1").is_some());
        assert!(ctx.find_place("nope").is_none());
        assert!(ctx.find_concept("co1").is_some());
        assert!(ctx.find_concept("nope").is_none());
        assert!(ctx.find_event("te1").is_some());
        assert!(ctx.find_event("nope").is_none());
    }

    #[test]
    fn remove_chapter_associations() {
        let mut ctx = CreativeContext {
            characters: vec![{
                let mut c = Character::new("ch1".into(), "Martín".into());
                c.chapter_ids = vec!["ch_a".into(), "ch_b".into(), "ch_c".into()];
                c
            }],
            places: vec![{
                let mut p = Place::new("pl1".into(), "Casa".into());
                p.chapter_ids = vec!["ch_a".into(), "ch_b".into()];
                p
            }],
            timeline: vec![{
                let mut e = TimelineEvent::new("te1".into(), "Evento".into(), 1);
                e.chapter_ids = vec!["ch_a".into()];
                e
            }],
            ..CreativeContext::default()
        };

        ctx.remove_chapter_associations("ch_b");

        assert_eq!(ctx.characters[0].chapter_ids, vec!["ch_a", "ch_c"]);
        assert_eq!(ctx.places[0].chapter_ids, vec!["ch_a"]);
        assert_eq!(ctx.timeline[0].chapter_ids, vec!["ch_a"]);
    }

    #[test]
    fn empty_toml_deserializes_to_default() {
        let toml_str = "[[characters]]\nname = \"Test\"\nid = \"c1\"\ncreated = \"1\"\nmodified = \"1\"\n";
        let ctx: CreativeContext = toml::from_str(toml_str).unwrap();
        assert_eq!(ctx.characters.len(), 1);
        assert_eq!(ctx.characters[0].name, "Test");
        assert!(ctx.places.is_empty());
    }

    #[test]
    fn save_and_load_empty() {
        let dir = std::env::temp_dir().join("lumen_creative_test_empty");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join(CREATIVE_FILE);

        let ctx = CreativeContext::new();
        ctx.save(&path).unwrap();

        let loaded = CreativeContext::load(&path).unwrap();
        assert_eq!(ctx, loaded);

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn save_and_load_populated() {
        let dir = std::env::temp_dir().join("lumen_creative_test_pop");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join(CREATIVE_FILE);

        let ctx = CreativeContext {
            characters: vec![Character::new("ch1".into(), "Martín".into())],
            places: vec![Place::new("pl1".into(), "Casa".into())],
            concepts: vec![Concept::new("co1".into(), "Secreto".into())],
            timeline: vec![TimelineEvent::new("te1".into(), "Incendio".into(), 2018)],
        };
        ctx.save(&path).unwrap();

        let loaded = CreativeContext::load(&path).unwrap();
        assert_eq!(ctx, loaded);
        assert_eq!(loaded.characters[0].name, "Martín");
        assert_eq!(loaded.timeline[0].order, 2018);

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn load_missing_file_returns_empty() {
        let dir = std::env::temp_dir().join("lumen_creative_test_miss");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("nonexistent.toml");

        let loaded = CreativeContext::load(&path).unwrap();
        assert_eq!(loaded, CreativeContext::default());

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn save_creates_parent_dirs() {
        let dir = std::env::temp_dir().join("lumen_creative_test_dirs").join("deep").join("path");
        let _ = fs::remove_dir_all(std::env::temp_dir().join("lumen_creative_test_dirs"));
        let path = dir.join(CREATIVE_FILE);

        let ctx = CreativeContext::new();
        ctx.save(&path).unwrap();
        assert!(path.exists());

        let _ = fs::remove_dir_all(std::env::temp_dir().join("lumen_creative_test_dirs"));
    }

    #[test]
    fn save_atomic_no_tmp_left_on_success() {
        let dir = std::env::temp_dir().join("lumen_creative_test_atomic");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join(CREATIVE_FILE);

        let ctx = CreativeContext::new();
        ctx.save(&path).unwrap();

        assert!(path.exists());
        assert!(!path.with_extension("toml.tmp").exists(), "tmp file should be cleaned up");

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn save_unicode_roundtrip() {
        let dir = std::env::temp_dir().join("lumen_creative_test_uni");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join(CREATIVE_FILE);

        let ctx = CreativeContext {
            characters: vec![Character::new("c1".into(), "Ñoño — 日本語".into())],
            ..CreativeContext::default()
        };
        ctx.save(&path).unwrap();

        let loaded = CreativeContext::load(&path).unwrap();
        assert_eq!(loaded.characters[0].name, "Ñoño — 日本語");

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn save_load_modify_reload() {
        let dir = std::env::temp_dir().join("lumen_creative_test_modify");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join(CREATIVE_FILE);

        let mut ctx = CreativeContext::new();
        ctx.characters.push(Character::new("c1".into(), "María".into()));
        ctx.save(&path).unwrap();

        let mut loaded = CreativeContext::load(&path).unwrap();
        loaded.characters[0].name = "María改名".into();
        loaded.places.push(Place::new("p1".into(), "Casa".into()));
        loaded.save(&path).unwrap();

        let reloaded = CreativeContext::load(&path).unwrap();
        assert_eq!(reloaded.characters[0].name, "María改名");
        assert_eq!(reloaded.places.len(), 1);
        assert_eq!(reloaded.places[0].name, "Casa");

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn character_ids_stable_across_save_reload() {
        let dir = std::env::temp_dir().join("lumen_creative_test_stable_id");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join(CREATIVE_FILE);

        let mut ctx = CreativeContext::new();
        ctx.characters.push(Character::new("ch_abc123".into(), "Elena".into()));
        ctx.save(&path).unwrap();

        let loaded = CreativeContext::load(&path).unwrap();
        assert_eq!(loaded.characters[0].id, "ch_abc123", "ID must not change on reload");

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn creative_file_in_project_dir_structure() {
        let dir = std::env::temp_dir().join("lumen_creative_test_projdir");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(dir.join(".lumen")).unwrap();
        let path = dir.join(CREATIVE_FILE);

        let ctx = CreativeContext {
            characters: vec![Character::new("c1".into(), "Ana".into())],
            places: vec![Place::new("p1".into(), "Buenos Aires".into())],
            concepts: vec![Concept::new("k1".into(), "Libertad".into())],
            timeline: vec![TimelineEvent::new("t1".into(), "Nacimiento".into(), 1990)],
        };
        ctx.save(&path).unwrap();

        let loaded = CreativeContext::load(&path).unwrap();
        assert_eq!(loaded.characters.len(), 1);
        assert_eq!(loaded.places.len(), 1);
        assert_eq!(loaded.concepts.len(), 1);
        assert_eq!(loaded.timeline.len(), 1);

        let _ = fs::remove_dir_all(&dir);
    }
}