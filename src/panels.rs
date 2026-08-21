use std::path::PathBuf;

// ── Panel activo ──

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PanelKind {
    Notes,
    Spellcheck,
    Ideas,
    Creative,
}

// ── Notas ──

#[derive(Debug, Clone)]
pub struct Note {
    pub text: String,
}

#[derive(Debug)]
pub struct NotesState {
    pub notes: Vec<Note>,
    pub selected: usize,
    pub editing: bool,
    pub editing_text: String,
    pub file_path: Option<PathBuf>,
}

impl NotesState {
    pub fn new() -> Self {
        Self {
            notes: Vec::new(),
            selected: 0,
            editing: false,
            editing_text: String::new(),
            file_path: None,
        }
    }

    pub fn add_note(&mut self) {
        self.notes.push(Note { text: String::new() });
        self.selected = self.notes.len() - 1;
        self.editing = true;
        self.editing_text.clear();
    }

    pub fn delete_selected(&mut self) {
        if self.notes.is_empty() {
            return;
        }
        self.notes.remove(self.selected);
        if self.selected >= self.notes.len() && self.selected > 0 {
            self.selected -= 1;
        }
        self.editing = false;
    }

    pub fn move_up(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
        }
    }

    pub fn move_down(&mut self) {
        if self.selected + 1 < self.notes.len() {
            self.selected += 1;
        }
    }

    pub fn start_edit(&mut self) {
        if let Some(note) = self.notes.get(self.selected) {
            self.editing = true;
            self.editing_text = note.text.clone();
        }
    }

    pub fn commit_edit(&mut self) {
        if self.editing {
            if let Some(note) = self.notes.get_mut(self.selected) {
                note.text = self.editing_text.clone();
            }
            self.editing = false;
            self.editing_text.clear();
        }
    }

    pub fn push_char(&mut self, c: char) {
        if self.editing {
            self.editing_text.push(c);
        }
    }

    pub fn pop_char(&mut self) {
        if self.editing {
            self.editing_text.pop();
        }
    }

    pub fn save_to_file(&self) {
        if let Some(path) = &self.file_path {
            let content: Vec<String> = self.notes.iter().map(|n| n.text.clone()).collect();
            let _ = std::fs::write(path, content.join("\n---\n"));
        }
    }

    pub fn load_from_file(path: PathBuf) -> Self {
        let mut state = Self::new();
        state.file_path = Some(path.clone());
        if let Ok(text) = std::fs::read_to_string(&path) {
            state.notes = text
                .split("\n---\n")
                .map(|s| Note { text: s.trim().to_string() })
                .filter(|n| !n.text.is_empty())
                .collect();
        }
        state
    }
}

// ── Ideas ──

#[derive(Debug, Clone)]
pub struct Idea {
    pub text: String,
}

#[derive(Debug)]
pub struct IdeasState {
    pub ideas: Vec<Idea>,
    pub selected: usize,
    pub editing: bool,
    pub editing_text: String,
    pub file_path: Option<PathBuf>,
}

impl IdeasState {
    pub fn new() -> Self {
        Self {
            ideas: Vec::new(),
            selected: 0,
            editing: false,
            editing_text: String::new(),
            file_path: None,
        }
    }

    pub fn add_idea(&mut self) {
        self.ideas.push(Idea { text: String::new() });
        self.selected = self.ideas.len() - 1;
        self.editing = true;
        self.editing_text.clear();
    }

    pub fn delete_selected(&mut self) {
        if self.ideas.is_empty() {
            return;
        }
        self.ideas.remove(self.selected);
        if self.selected >= self.ideas.len() && self.selected > 0 {
            self.selected -= 1;
        }
        self.editing = false;
    }

    pub fn move_up(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
        }
    }

    pub fn move_down(&mut self) {
        if self.selected + 1 < self.ideas.len() {
            self.selected += 1;
        }
    }

    pub fn start_edit(&mut self) {
        if let Some(idea) = self.ideas.get(self.selected) {
            self.editing = true;
            self.editing_text = idea.text.clone();
        }
    }

    pub fn commit_edit(&mut self) {
        if self.editing {
            if let Some(idea) = self.ideas.get_mut(self.selected) {
                idea.text = self.editing_text.clone();
            }
            self.editing = false;
            self.editing_text.clear();
        }
    }

    pub fn push_char(&mut self, c: char) {
        if self.editing {
            self.editing_text.push(c);
        }
    }

    pub fn pop_char(&mut self) {
        if self.editing {
            self.editing_text.pop();
        }
    }

    pub fn save_to_file(&self) {
        if let Some(path) = &self.file_path {
            let content: Vec<String> = self.ideas.iter().map(|i| i.text.clone()).collect();
            let _ = std::fs::write(path, content.join("\n"));
        }
    }

    pub fn load_from_file(path: PathBuf) -> Self {
        let mut state = Self::new();
        state.file_path = Some(path.clone());
        if let Ok(text) = std::fs::read_to_string(&path) {
            state.ideas = text
                .lines()
                .map(|s| Idea { text: s.to_string() })
                .filter(|i| !i.text.is_empty())
                .collect();
        }
        state
    }
}

// ── Ortografía ──

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpellcheckMode {
    Errors,
    Suggestions,
    LanguageSelect,
}

#[derive(Debug)]
pub struct SpellcheckState {
    pub mode: SpellcheckMode,
    pub selected: usize,
    pub suggestion_selected: usize,
    pub suggestions: Vec<String>,
    pub lang_selected: usize,
}

impl SpellcheckState {
    pub fn new() -> Self {
        Self {
            mode: SpellcheckMode::Errors,
            selected: 0,
            suggestion_selected: 0,
            suggestions: Vec::new(),
            lang_selected: 0,
        }
    }

    pub fn reset(&mut self) {
        self.mode = SpellcheckMode::Errors;
        self.selected = 0;
        self.suggestion_selected = 0;
        self.suggestions.clear();
        self.lang_selected = 0;
    }
}

// ── Creative Panel ──

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CreativeSection {
    Chapters,
    Characters,
    Places,
    Timeline,
    Concepts,
    Statistics,
}

impl CreativeSection {
    pub fn all() -> &'static [CreativeSection] {
        &[
            CreativeSection::Chapters,
            CreativeSection::Characters,
            CreativeSection::Places,
            CreativeSection::Timeline,
            CreativeSection::Concepts,
            CreativeSection::Statistics,
        ]
    }

    pub fn index(&self) -> usize {
        Self::all().iter().position(|s| s == self).unwrap_or(0)
    }

    pub fn label(&self) -> &'static str {
        match self {
            CreativeSection::Chapters => "Capítulos",
            CreativeSection::Characters => "Personajes",
            CreativeSection::Places => "Lugares",
            CreativeSection::Timeline => "Línea de tiempo",
            CreativeSection::Concepts => "Conceptos",
            CreativeSection::Statistics => "Estadísticas",
        }
    }

    pub fn key_hint(&self) -> &'static str {
        match self {
            CreativeSection::Chapters => "1",
            CreativeSection::Characters => "2",
            CreativeSection::Places => "3",
            CreativeSection::Timeline => "4",
            CreativeSection::Concepts => "5",
            CreativeSection::Statistics => "6",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CreativeMode {
    Menu,
    List,
    EditName,
    EditDescription,
    EditNotes,
    ConfirmDelete,
}

#[derive(Debug)]
pub struct CreativeState {
    pub section: CreativeSection,
    pub mode: CreativeMode,
    pub selected: usize,
    pub edit_buffer: String,
    pub scroll_offset: usize,
}

impl CreativeState {
    pub fn new() -> Self {
        Self {
            section: CreativeSection::Chapters,
            mode: CreativeMode::Menu,
            selected: 0,
            edit_buffer: String::new(),
            scroll_offset: 0,
        }
    }

    pub fn open_section(&mut self, section: CreativeSection) {
        self.section = section;
        self.mode = CreativeMode::List;
        self.selected = 0;
        self.scroll_offset = 0;
    }

    pub fn back_to_menu(&mut self) {
        self.mode = CreativeMode::Menu;
        self.selected = self.section.index();
        self.scroll_offset = 0;
    }

    pub fn move_up(&mut self, max_items: usize) {
        if self.selected > 0 {
            self.selected -= 1;
        }
        if self.selected < self.scroll_offset {
            self.scroll_offset = self.selected;
        }
    }

    pub fn move_down(&mut self, max_items: usize) {
        if max_items > 0 && self.selected + 1 < max_items {
            self.selected += 1;
        }
    }

    pub fn start_edit(&mut self, initial: String) {
        self.mode = CreativeMode::EditName;
        self.edit_buffer = initial;
    }

    pub fn start_edit_desc(&mut self, initial: String) {
        self.mode = CreativeMode::EditDescription;
        self.edit_buffer = initial;
    }

    pub fn start_edit_notes(&mut self, initial: String) {
        self.mode = CreativeMode::EditNotes;
        self.edit_buffer = initial;
    }

    pub fn push_char(&mut self, c: char) {
        self.edit_buffer.push(c);
    }

    pub fn pop_char(&mut self) {
        self.edit_buffer.pop();
    }

    pub fn confirm_delete(&mut self) {
        self.mode = CreativeMode::ConfirmDelete;
    }

    pub fn cancel(&mut self) {
        match self.mode {
            CreativeMode::Menu => {}
            _ => self.mode = CreativeMode::List,
        }
        self.edit_buffer.clear();
    }
}

// ── Proyecto (stub) ──

#[derive(Debug, Clone)]
pub struct ProjectChapter {
    pub title: String,
}

#[derive(Debug)]
pub struct ProjectState {
    pub title: String,
    pub chapters: Vec<ProjectChapter>,
}

impl ProjectState {
    pub fn new() -> Self {
        Self {
            title: String::new(),
            chapters: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creative_section_all_has_6() {
        assert_eq!(CreativeSection::all().len(), 6);
    }

    #[test]
    fn creative_section_index_and_key() {
        let sections = CreativeSection::all();
        for (i, s) in sections.iter().enumerate() {
            assert_eq!(s.index(), i);
            assert_eq!(s.key_hint(), format!("{}", i + 1));
        }
    }

    #[test]
    fn creative_section_labels() {
        assert_eq!(CreativeSection::Chapters.label(), "Capítulos");
        assert_eq!(CreativeSection::Characters.label(), "Personajes");
        assert_eq!(CreativeSection::Places.label(), "Lugares");
        assert_eq!(CreativeSection::Timeline.label(), "Línea de tiempo");
        assert_eq!(CreativeSection::Concepts.label(), "Conceptos");
        assert_eq!(CreativeSection::Statistics.label(), "Estadísticas");
    }

    #[test]
    fn creative_state_new_defaults() {
        let s = CreativeState::new();
        assert_eq!(s.section, CreativeSection::Chapters);
        assert_eq!(s.mode, CreativeMode::Menu);
        assert_eq!(s.selected, 0);
    }

    #[test]
    fn creative_state_open_section() {
        let mut s = CreativeState::new();
        s.open_section(CreativeSection::Characters);
        assert_eq!(s.section, CreativeSection::Characters);
        assert_eq!(s.mode, CreativeMode::List);
        assert_eq!(s.selected, 0);
    }

    #[test]
    fn creative_state_back_to_menu() {
        let mut s = CreativeState::new();
        s.open_section(CreativeSection::Places);
        s.selected = 3;
        s.back_to_menu();
        assert_eq!(s.mode, CreativeMode::Menu);
        assert_eq!(s.selected, 2, "should select Places index in menu");
    }

    #[test]
    fn creative_state_move() {
        let mut s = CreativeState::new();
        s.mode = CreativeMode::List;

        s.move_down(10);
        assert_eq!(s.selected, 1);
        s.move_up(10);
        assert_eq!(s.selected, 0);
        s.move_up(10);
        assert_eq!(s.selected, 0, "should not go below 0");
    }

    #[test]
    fn creative_state_edit() {
        let mut s = CreativeState::new();
        s.start_edit("initial".into());
        assert_eq!(s.mode, CreativeMode::EditName);
        assert_eq!(s.edit_buffer, "initial");

        s.push_char('!');
        assert_eq!(s.edit_buffer, "initial!");
        s.pop_char();
        assert_eq!(s.edit_buffer, "initial");
    }

    #[test]
    fn creative_state_cancel() {
        let mut s = CreativeState::new();
        s.open_section(CreativeSection::Chapters);
        s.mode = CreativeMode::EditName;
        s.edit_buffer = "test".into();

        s.cancel();
        assert_eq!(s.mode, CreativeMode::List);
        assert!(s.edit_buffer.is_empty());
    }

    #[test]
    fn creative_state_confirm_delete() {
        let mut s = CreativeState::new();
        s.open_section(CreativeSection::Characters);
        s.confirm_delete();
        assert_eq!(s.mode, CreativeMode::ConfirmDelete);
    }
}
