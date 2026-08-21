use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use ropey::Rope;
use serde::{Deserialize, Serialize};

const PROJECT_DIR: &str = ".lumen";
const PROJECT_FILE: &str = "project.toml";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProjectMeta {
    pub title: String,
    pub author: String,
    pub language: String,
    pub created: String,
    pub modified: String,
}

impl Default for ProjectMeta {
    fn default() -> Self {
        let now = chrono_now();
        Self {
            title: String::new(),
            author: String::new(),
            language: "en".to_string(),
            created: now.clone(),
            modified: now,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ChapterState {
    Borrador,
    EnRevision,
    Revisado,
    Finalizado,
}

impl Default for ChapterState {
    fn default() -> Self {
        ChapterState::Borrador
    }
}

impl ChapterState {
    pub fn marker(&self) -> &'static str {
        match self {
            ChapterState::Borrador => "B",
            ChapterState::EnRevision => "R",
            ChapterState::Revisado => "•",
            ChapterState::Finalizado => "✓",
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            ChapterState::Borrador => "Borrador",
            ChapterState::EnRevision => "En revisión",
            ChapterState::Revisado => "Revisado",
            ChapterState::Finalizado => "Finalizado",
        }
    }

    pub fn cycle(&self) -> Self {
        match self {
            ChapterState::Borrador => ChapterState::EnRevision,
            ChapterState::EnRevision => ChapterState::Revisado,
            ChapterState::Revisado => ChapterState::Finalizado,
            ChapterState::Finalizado => ChapterState::Borrador,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ChapterEntry {
    #[serde(default = "generate_chapter_id")]
    pub id: String,
    pub title: String,
    pub filename: String,
    pub format: String,
    #[serde(default)]
    pub state: ChapterState,
}

fn generate_chapter_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let pid = std::process::id();
    format!("ch_{:016x}_{:08x}", secs, pid)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ChapterIndex {
    chapters: Vec<ChapterEntry>,
}

#[derive(Debug)]
pub struct Project {
    root: PathBuf,
    meta: ProjectMeta,
    chapters: Vec<ChapterEntry>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProjectMode {
    Overview,
    ChapterList,
    Metadata,
}

impl Project {
    /// Create a new project at the given directory
    pub fn create(root: PathBuf, title: String, author: String) -> io::Result<Self> {
        let project_dir = root.join(PROJECT_DIR);
        fs::create_dir_all(&project_dir)?;

        let meta = ProjectMeta {
            title,
            author,
            ..ProjectMeta::default()
        };

        let project = Self {
            root,
            meta,
            chapters: Vec::new(),
        };

        project.save()?;
        Ok(project)
    }

    /// Open an existing project from a directory
    pub fn open(root: PathBuf) -> io::Result<Self> {
        let meta_path = root.join(PROJECT_DIR).join(PROJECT_FILE);
        let content = fs::read_to_string(&meta_path)?;
        let meta: ProjectMeta = toml::from_str(&content)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        let chapters = Self::scan_chapters(&root)?;

        Ok(Self {
            root,
            meta,
            chapters,
        })
    }

    /// Check if a directory contains a Lumen project
    pub fn is_project_dir(path: &Path) -> bool {
        path.join(PROJECT_DIR).join(PROJECT_FILE).exists()
    }

    /// Walk up from a file path and return the project root if a `.lumen/` dir is found.
    /// Returns None for standalone files with no project ancestor.
    pub fn detect_root_for_file(file_path: &Path) -> Option<PathBuf> {
        let start = file_path.parent()?;
        let mut current = start.to_path_buf();
        loop {
            if Self::is_project_dir(&current) {
                return Some(current);
            }
            if !current.pop() {
                break;
            }
        }
        None
    }

    /// Save project metadata
    pub fn save(&self) -> io::Result<()> {
        let project_dir = self.root.join(PROJECT_DIR);
        fs::create_dir_all(&project_dir)?;

        let meta_path = project_dir.join(PROJECT_FILE);
        let content = toml::to_string_pretty(&self.meta)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        fs::write(meta_path, content)
    }

    /// Get project root
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Get project metadata
    pub fn meta(&self) -> &ProjectMeta {
        &self.meta
    }

    /// Get mutable metadata
    pub fn meta_mut(&mut self) -> &mut ProjectMeta {
        &mut self.meta
    }

    /// Get chapters
    pub fn chapters(&self) -> &[ChapterEntry] {
        &self.chapters
    }

    /// Get mutable chapters
    pub fn chapters_mut(&mut self) -> &mut [ChapterEntry] {
        &mut self.chapters
    }

    /// Set a chapter's state and persist
    pub fn set_chapter_state(&mut self, index: usize, state: ChapterState) -> io::Result<()> {
        if index >= self.chapters.len() {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, "chapter index out of bounds"));
        }
        self.chapters[index].state = state;
        self.save_chapter_index()
    }

    /// Add a new chapter
    pub fn add_chapter(&mut self, title: String) -> io::Result<usize> {
        let index = self.chapters.len();
        let filename = self.generate_filename(&title);
        let format = "txt".to_string();

        self.chapters.push(ChapterEntry {
            id: generate_chapter_id(),
            title,
            filename: filename.clone(),
            format,
            state: ChapterState::default(),
        });

        let chapter_path = self.root.join(&filename);
        fs::write(&chapter_path, "")?;

        self.meta.modified = chrono_now();
        self.save()?;
        self.save_chapter_index()?;
        Ok(index)
    }

    /// Remove a chapter by index
    pub fn remove_chapter(&mut self, index: usize) -> io::Result<()> {
        if index >= self.chapters.len() {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, "chapter index out of bounds"));
        }

        let chapter = self.chapters.remove(index);
        let chapter_path = self.root.join(&chapter.filename);
        if chapter_path.exists() {
            fs::remove_file(&chapter_path)?;
        }

        self.meta.modified = chrono_now();
        self.save()?;
        self.save_chapter_index()
    }

    /// Reorder chapters (move from one index to another)
    pub fn move_chapter(&mut self, from: usize, to: usize) -> io::Result<()> {
        if from >= self.chapters.len() || to >= self.chapters.len() {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, "chapter index out of bounds"));
        }

        let chapter = self.chapters.remove(from);
        self.chapters.insert(to, chapter);

        self.meta.modified = chrono_now();
        self.save()?;
        self.save_chapter_index()
    }

    /// Get total word count across all chapters
    pub fn total_word_count(&self) -> usize {
        self.chapters.iter().map(|ch| {
            let path = self.root.join(&ch.filename);
            if path.exists() {
                if let Ok(content) = fs::read_to_string(&path) {
                    return content.split_whitespace().count();
                }
            }
            0
        }).sum()
    }

    /// Open a chapter's content as a Rope
    pub fn open_chapter(&self, index: usize) -> io::Result<Rope> {
        let chapter = self.chapters.get(index)
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "chapter index out of bounds"))?;

        let path = self.root.join(&chapter.filename);
        if !path.exists() {
            return Ok(Rope::new());
        }

        let bytes = fs::read(&path)?;
        let text = String::from_utf8(bytes)
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "file is not valid UTF-8"))?;
        Ok(Rope::from_str(&text))
    }

    /// Save a chapter's content
    pub fn save_chapter(&self, index: usize, rope: &Rope) -> io::Result<()> {
        let chapter = self.chapters.get(index)
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "chapter index out of bounds"))?;

        let path = self.root.join(&chapter.filename);

        use std::io::Write;
        let tmp = path.with_extension("txt.tmp");
        {
            let mut f = fs::File::create(&tmp)?;
            f.write_all(rope.to_string().as_bytes())?;
            f.flush()?;
            f.sync_all()?;
        }
        fs::rename(&tmp, &path)
    }

    /// Rename a chapter
    pub fn rename_chapter(&mut self, index: usize, new_title: String) -> io::Result<()> {
        if index >= self.chapters.len() {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, "chapter index out of bounds"));
        }

        let new_filename = self.generate_filename(&new_title);
        let chapter = &mut self.chapters[index];
        let old_path = self.root.join(&chapter.filename);
        let new_path = self.root.join(&new_filename);

        if old_path.exists() {
            fs::rename(&old_path, &new_path)?;
        }

        chapter.title = new_title;
        chapter.filename = new_filename;

        self.meta.modified = chrono_now();
        self.save()?;
        self.save_chapter_index()
    }

    fn scan_chapters(root: &Path) -> io::Result<Vec<ChapterEntry>> {
        let project_dir = root.join(PROJECT_DIR);
        let index_path = project_dir.join("chapters.toml");

        if index_path.exists() {
            let content = fs::read_to_string(&index_path)?;
            let index: ChapterIndex = toml::from_str(&content)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
            Ok(index.chapters)
        } else {
            let mut chapters = Vec::new();
            // Fallback: scan for .txt and .odt files in root
            for entry in fs::read_dir(root)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_file() {
                    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                        match ext {
                            "txt" | "odt" => {
                                let filename = path.file_name()
                                    .and_then(|f| f.to_str())
                                    .unwrap_or("")
                                    .to_string();
                                let title = path.file_stem()
                                    .and_then(|f| f.to_str())
                                    .unwrap_or("Untitled")
                                    .to_string();
                                chapters.push(ChapterEntry {
                                    id: generate_chapter_id(),
                                    title,
                                    filename,
                                    format: ext.to_string(),
                                    state: ChapterState::default(),
                                });
                            }
                            _ => {}
                        }
                    }
                }
            }
            Ok(chapters)
        }
    }

    fn generate_filename(&self, title: &str) -> String {
        let base: String = title.chars()
            .map(|c| if c.is_alphanumeric() || c == '-' || c == '_' { c } else { '_' })
            .collect();
        let base = if base.is_empty() { "chapter".to_string() } else { base };

        let mut filename = format!("{}.txt", base);
        let mut counter = 1;
        while self.chapters.iter().any(|ch| ch.filename == filename) {
            filename = format!("{}_{}.txt", base, counter);
            counter += 1;
        }
        filename
    }

    /// Save the chapter index file
    pub fn save_chapter_index(&self) -> io::Result<()> {
        let project_dir = self.root.join(PROJECT_DIR);
        fs::create_dir_all(&project_dir)?;

        let index_path = project_dir.join("chapters.toml");
        let index = ChapterIndex {
            chapters: self.chapters.clone(),
        };
        let content = toml::to_string_pretty(&index)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        fs::write(index_path, content)
    }
}

fn chrono_now() -> String {
    // Simple timestamp without chrono dependency
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
    use std::sync::atomic::{AtomicUsize, Ordering};

    static COUNTER: AtomicUsize = AtomicUsize::new(0);

    fn temp_project() -> (PathBuf, Project) {
        let id = COUNTER.fetch_add(1, Ordering::SeqCst);
        let dir = std::env::temp_dir().join(format!("lumen_proj_{}_{}", std::process::id(), id));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let project = Project::create(dir.clone(), "Test Novel".to_string(), "Author".to_string()).unwrap();
        (dir, project)
    }

    fn cleanup(dir: &Path) {
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn create_and_open_project() {
        let (dir, project) = temp_project();
        assert_eq!(project.meta().title, "Test Novel");
        assert_eq!(project.meta().author, "Author");
        assert!(project.chapters().is_empty());

        let opened = Project::open(dir.clone()).unwrap();
        assert_eq!(opened.meta().title, "Test Novel");

        cleanup(&dir);
    }

    #[test]
    fn add_and_remove_chapters() {
        let (dir, mut project) = temp_project();

        let idx = project.add_chapter("Chapter 1".to_string()).unwrap();
        assert_eq!(idx, 0);
        assert_eq!(project.chapters().len(), 1);
        assert_eq!(project.chapters()[0].title, "Chapter 1");

        project.add_chapter("Chapter 2".to_string()).unwrap();
        assert_eq!(project.chapters().len(), 2);

        project.remove_chapter(0).unwrap();
        assert_eq!(project.chapters().len(), 1);
        assert_eq!(project.chapters()[0].title, "Chapter 2");

        cleanup(&dir);
    }

    #[test]
    fn chapter_read_write() {
        let (dir, mut project) = temp_project();
        project.add_chapter("Test Chapter".to_string()).unwrap();

        let rope = Rope::from_str("Hello, world!\nSecond line.");
        project.save_chapter(0, &rope).unwrap();

        let loaded = project.open_chapter(0).unwrap();
        assert_eq!(loaded.to_string(), "Hello, world!\nSecond line.");

        cleanup(&dir);
    }

    #[test]
    fn move_chapter_reorders() {
        let (dir, mut project) = temp_project();
        project.add_chapter("First".to_string()).unwrap();
        project.add_chapter("Second".to_string()).unwrap();
        project.add_chapter("Third".to_string()).unwrap();

        project.move_chapter(0, 2).unwrap();
        assert_eq!(project.chapters()[0].title, "Second");
        assert_eq!(project.chapters()[1].title, "Third");
        assert_eq!(project.chapters()[2].title, "First");

        cleanup(&dir);
    }

    #[test]
    fn rename_chapter() {
        let (dir, mut project) = temp_project();
        project.add_chapter("Old Name".to_string()).unwrap();

        project.rename_chapter(0, "New Name".to_string()).unwrap();
        assert_eq!(project.chapters()[0].title, "New Name");
        assert!(project.chapters()[0].filename.contains("New_Name"));

        cleanup(&dir);
    }

    #[test]
    fn is_project_dir_detection() {
        let (dir, _) = temp_project();
        assert!(Project::is_project_dir(&dir));

        let not_project = std::env::temp_dir().join("not_a_project");
        let _ = fs::create_dir_all(&not_project);
        assert!(!Project::is_project_dir(&not_project));

        cleanup(&dir);
        cleanup(&not_project);
    }

    #[test]
    fn total_word_count() {
        let (dir, mut project) = temp_project();
        project.add_chapter("Ch1".to_string()).unwrap();
        project.add_chapter("Ch2".to_string()).unwrap();

        project.save_chapter(0, &Rope::from_str("one two three")).unwrap();
        project.save_chapter(1, &Rope::from_str("four five")).unwrap();

        assert_eq!(project.total_word_count(), 5);

        cleanup(&dir);
    }

    #[test]
    fn chapter_index_persists() {
        let (dir, mut project) = temp_project();
        project.add_chapter("Ch1".to_string()).unwrap();
        project.add_chapter("Ch2".to_string()).unwrap();

        let opened = Project::open(dir.clone()).unwrap();
        assert_eq!(opened.chapters().len(), 2);
        assert_eq!(opened.chapters()[0].title, "Ch1");
        assert_eq!(opened.chapters()[1].title, "Ch2");

        cleanup(&dir);
    }

    #[test]
    fn chapter_has_stable_id() {
        let (dir, mut project) = temp_project();
        let _ = project.add_chapter("Ch1".to_string()).unwrap();

        let id = project.chapters()[0].id.clone();
        assert!(id.starts_with("ch_"));

        let opened = Project::open(dir.clone()).unwrap();
        assert_eq!(opened.chapters()[0].id, id, "chapter ID must persist across load");

        cleanup(&dir);
    }

    #[test]
    fn chapter_id_survives_reorder() {
        let (dir, mut project) = temp_project();
        project.add_chapter("A".to_string()).unwrap();
        project.add_chapter("B".to_string()).unwrap();

        let id_a = project.chapters()[0].id.clone();
        let id_b = project.chapters()[1].id.clone();

        project.move_chapter(0, 1).unwrap();

        assert_eq!(project.chapters()[0].id, id_b);
        assert_eq!(project.chapters()[1].id, id_a);

        cleanup(&dir);
    }

    #[test]
    fn chapter_state_markers() {
        assert_eq!(ChapterState::Borrador.marker(), "B");
        assert_eq!(ChapterState::EnRevision.marker(), "R");
        assert_eq!(ChapterState::Revisado.marker(), "•");
        assert_eq!(ChapterState::Finalizado.marker(), "✓");
    }

    #[test]
    fn chapter_state_cycle() {
        let s = ChapterState::Borrador;
        assert_eq!(s.cycle(), ChapterState::EnRevision);
        assert_eq!(s.cycle().cycle(), ChapterState::Revisado);
        assert_eq!(s.cycle().cycle().cycle(), ChapterState::Finalizado);
        assert_eq!(s.cycle().cycle().cycle().cycle(), ChapterState::Borrador);
    }

    #[test]
    fn chapter_state_default_is_borrador() {
        let s = ChapterState::default();
        assert_eq!(s, ChapterState::Borrador);
    }

    #[test]
    fn chapter_state_persists() {
        let (dir, mut project) = temp_project();
        project.add_chapter("Ch1".to_string()).unwrap();

        project.set_chapter_state(0, ChapterState::Finalizado).unwrap();

        let opened = Project::open(dir.clone()).unwrap();
        assert_eq!(opened.chapters()[0].state, ChapterState::Finalizado);

        cleanup(&dir);
    }

    #[test]
    fn chapter_backward_compat_no_id_field() {
        // Simulate an old chapters.toml without id or state fields
        let toml_str = r#"
[[chapters]]
title = "Old Chapter"
filename = "old.txt"
format = "txt"
"#;
        let index: ChapterIndex = toml::from_str(toml_str).unwrap();
        assert_eq!(index.chapters.len(), 1);
        assert_eq!(index.chapters[0].title, "Old Chapter");
        assert!(!index.chapters[0].id.is_empty(), "id should be auto-generated");
        assert_eq!(index.chapters[0].state, ChapterState::Borrador, "state should default to Borrador");
    }

    #[test]
    fn detect_root_for_file_inside_project() {
        let dir = std::env::temp_dir().join("lumen_test_detect_inside");
        let _ = cleanup(&dir);
        let _ = Project::create(dir.clone(), "Test".into(), "A".into());

        let file = dir.join("chapter1.txt");
        std::fs::write(&file, "hola").unwrap();

        let detected = Project::detect_root_for_file(&file);
        assert_eq!(detected, Some(dir.clone()), "should detect project root for file inside project");

        cleanup(&dir);
    }

    #[test]
    fn detect_root_for_file_in_subdirectory() {
        let dir = std::env::temp_dir().join("lumen_test_detect_sub");
        let _ = cleanup(&dir);
        let _ = Project::create(dir.clone(), "Test".into(), "A".into());

        let subdir = dir.join("chapters");
        std::fs::create_dir_all(&subdir).unwrap();
        let file = subdir.join("chapter1.txt");
        std::fs::write(&file, "hola").unwrap();

        let detected = Project::detect_root_for_file(&file);
        assert_eq!(detected, Some(dir.clone()), "should detect project root from subdirectory");

        cleanup(&dir);
    }

    #[test]
    fn detect_root_for_file_outside_project() {
        let standalone = std::env::temp_dir().join("lumen_test_detect_outside.txt");
        std::fs::write(&standalone, "hola").unwrap();

        let detected = Project::detect_root_for_file(&standalone);
        assert_eq!(detected, None, "standalone file should have no project");

        std::fs::remove_file(&standalone).ok();
    }

    #[test]
    fn open_nonexistent_project_dir() {
        let dir = std::env::temp_dir().join("lumen_test_nonexistent_project");
        let result = Project::open(dir.clone());
        assert!(result.is_err(), "opening non-existent project should fail");
    }
}