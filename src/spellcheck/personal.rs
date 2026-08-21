use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;

// ── Diccionario personal ──

#[derive(Debug)]
pub struct PersonalDictionary {
    words: HashSet<String>,
    path: PathBuf,
}

impl PersonalDictionary {
    pub fn new() -> Self {
        let path = Self::default_path();
        let mut pd = Self {
            words: HashSet::new(),
            path,
        };
        pd.load();
        pd
    }

    pub fn with_path(path: PathBuf) -> Self {
        let mut pd = Self {
            words: HashSet::new(),
            path,
        };
        pd.load();
        pd
    }

    fn default_path() -> PathBuf {
        let config_dir = std::env::var("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| {
                let home = std::env::var("HOME")
                    .or_else(|_| std::env::var("USERPROFILE"))
                    .unwrap_or_else(|_| String::from("/"));
                PathBuf::from(home).join(".config")
            });
        config_dir.join("lumen").join("dictionary.txt")
    }

    fn load(&mut self) {
        if let Ok(text) = fs::read_to_string(&self.path) {
            for line in text.lines() {
                let word = line.trim().to_string();
                if !word.is_empty() && !word.starts_with('#') {
                    self.words.insert(word);
                }
            }
        }
    }

    pub fn save(&self) -> bool {
        if let Some(parent) = self.path.parent() {
            if let Err(_) = fs::create_dir_all(parent) {
                return false;
            }
        }
        let mut lines: Vec<&str> = self.words.iter().map(|w| w.as_str()).collect();
        lines.sort();
        let content = lines.join("\n");
        match fs::write(&self.path, content) {
            Ok(_) => true,
            Err(_) => false,
        }
    }

    pub fn add(&mut self, word: &str) {
        if self.words.insert(word.to_string()) {
            self.save();
        }
    }

    pub fn contains(&self, word: &str) -> bool {
        self.words.contains(word)
    }

    pub fn remove(&mut self, word: &str) {
        if self.words.remove(word) {
            self.save();
        }
    }

    pub fn words(&self) -> &HashSet<String> {
        &self.words
    }

    pub fn len(&self) -> usize {
        self.words.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn test_pd() -> PersonalDictionary {
        let dir = std::env::temp_dir().join("lumen_test_pd");
        let _ = fs::create_dir_all(&dir);
        let path = dir.join("dictionary.txt");
        let _ = fs::remove_file(&path);
        PersonalDictionary::with_path(path)
    }

    #[test]
    fn add_and_contains() {
        let mut pd = test_pd();
        pd.add("Arandor");
        assert!(pd.contains("Arandor"));
        assert!(!pd.contains("Zaruk"));
    }

    #[test]
    fn remove_works() {
        let mut pd = test_pd();
        pd.add("TestWord");
        assert!(pd.contains("TestWord"));
        pd.remove("TestWord");
        assert!(!pd.contains("TestWord"));
    }

    #[test]
    fn persistence() {
        let path = std::env::temp_dir().join("lumen_test_pd").join("dictionary.txt");
        let _ = fs::create_dir_all(path.parent().unwrap());
        {
            let mut pd = PersonalDictionary::with_path(path.clone());
            pd.add("Persist");
        }
        let pd2 = PersonalDictionary::with_path(path);
        assert!(pd2.contains("Persist"));
        let _ = fs::remove_file(std::env::temp_dir().join("lumen_test_pd").join("dictionary.txt"));
    }

    #[test]
    fn empty_at_start() {
        let pd = test_pd();
        assert_eq!(pd.len(), 0);
        let _ = fs::remove_file(std::env::temp_dir().join("lumen_test_pd").join("dictionary.txt"));
    }
}
