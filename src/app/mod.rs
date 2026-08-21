mod browser;

use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use browser::FileBrowser;
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

use crate::backup;
use crate::command::{self, Command};
use crate::config::Config;
use crate::document::Document;
use crate::editor::{char_cell_width, visual_col, Editor};
use crate::panels::*;
use crate::creative::CreativeContext;
use crate::project::{Project, ProjectMode};
use crate::search::Search;
use crate::session::Session;
use crate::spellcheck::{self, SpellcheckEngine};

// ── App ──

#[derive(Debug)]
pub struct App {
    pub doc: Document,
    pub editor: Editor,
    pub config: Config,
    pub scroll: Scroll,
    pub focus: bool,
    pub dirty: bool,
    pub message: Option<String>,
    pub clipboard: String,
    pub search: Search,
    pub dialog: Option<Dialog>,
    pub menu: Option<(MenuId, usize)>,
    pub word_count: usize,
    pub view_height: usize,
    pub view_width: usize,
    pub exit: bool,
    last_backup: Instant,
    backup_dirty: bool,
    recovery: Option<backup::Backup>,
    // ── Fase 2 ──
    pub active_panel: Option<PanelKind>,
    pub notes: NotesState,
    pub ideas: IdeasState,
    pub spellcheck: SpellcheckState,
    pub project: Option<Project>,
    pub project_mode: ProjectMode,
    pub session: Session,
    pub replace_active: bool,
    // ── Fase 3: ortografía ──
    pub spellcheck_engine: Option<SpellcheckEngine>,
    pub spellcheck_dirty: bool,
    // ── Fase 5: creativo ──
    pub creative: Option<CreativeContext>,
    pub creative_state: CreativeState,
    pub pending_project_title: Option<String>,
}

// ── Scroll ──

#[derive(Debug, Default, Clone, Copy)]
pub struct Scroll {
    pub top: usize,
    pub left: usize,
}

// ── Diálogos ──

#[derive(Debug, Clone)]
pub enum Dialog {
    Input {
        title: String,
        value: String,
        action: InputAction,
    },
    Confirm {
        message: String,
        action: ConfirmAction,
    },
    Message {
        title: String,
        text: String,
    },
    OpenBrowser(FileBrowser),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputAction {
    SaveAs,
    GoToLine,
    NewProjectTitle,
    NewProjectAuthor,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfirmAction {
    Open,
    Quit,
    Restore,
}

// ── Menú ──

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuId {
    File,
    Edit,
    Search,
}

impl MenuId {
    pub const ALL: [MenuId; 3] = [MenuId::File, MenuId::Edit, MenuId::Search];

    pub fn label(self) -> &'static str {
        match self {
            MenuId::File => "Archivo",
            MenuId::Edit => "Edición",
            MenuId::Search => "Buscar",
        }
    }

    pub fn accelerator_index(self) -> usize {
        0
    }

    pub fn from_accel(c: char) -> Option<MenuId> {
        match c {
            'a' => Some(MenuId::File),
            'e' => Some(MenuId::Edit),
            'b' => Some(MenuId::Search),
            _ => None,
        }
    }

    pub fn items(self) -> &'static [MenuItem] {
        match self {
            MenuId::File => &[
                MenuItem { label: "Abrir...", shortcut: "Ctrl+O", action: MenuAction::Open },
                MenuItem { label: "Nuevo proyecto...", shortcut: "Ctrl+Shift+N", action: MenuAction::NewProject },
                MenuItem { label: "Guardar", shortcut: "Ctrl+S", action: MenuAction::Save },
                MenuItem { label: "Guardar como...", shortcut: "Ctrl+Shift+S", action: MenuAction::SaveAs },
                MenuItem { label: "Salir", shortcut: "Ctrl+Q", action: MenuAction::Quit },
            ],
            MenuId::Edit => &[
                MenuItem { label: "Deshacer", shortcut: "Ctrl+Z", action: MenuAction::Undo },
                MenuItem { label: "Rehacer", shortcut: "Ctrl+Y", action: MenuAction::Redo },
                MenuItem { label: "Cortar", shortcut: "Ctrl+X", action: MenuAction::Cut },
                MenuItem { label: "Copiar", shortcut: "Ctrl+C", action: MenuAction::Copy },
                MenuItem { label: "Pegar", shortcut: "Ctrl+V", action: MenuAction::Paste },
            ],
            MenuId::Search => &[
                MenuItem { label: "Buscar...", shortcut: "Ctrl+F", action: MenuAction::Find },
                MenuItem { label: "Reemplazar...", shortcut: "Ctrl+H", action: MenuAction::Replace },
                MenuItem { label: "Ir a línea...", shortcut: "Ctrl+G", action: MenuAction::GoToLine },
                MenuItem { label: "Enfocar", shortcut: "Ctrl+Shift+F", action: MenuAction::Focus },
            ],
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MenuItem {
    pub label: &'static str,
    pub shortcut: &'static str,
    pub action: MenuAction,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuAction {
    Open,
    Save,
    SaveAs,
    Quit,
    Undo,
    Redo,
    Cut,
    Copy,
    Paste,
    Find,
    Replace,
    GoToLine,
    Focus,
    NewProject,
}

// ── App ──

impl App {
    pub fn new(file: Option<String>) -> Self {
        let config = Config::load();
        let spellcheck_engine = if config.spellcheck_enabled {
            let engine = if config.language != "auto" {
                spellcheck::find_dictionary_for_lang(&config.language)
                    .and_then(|info| SpellcheckEngine::new(&info))
                    .unwrap_or_else(SpellcheckEngine::empty)
            } else {
                spellcheck::auto_detect_dictionary()
                    .map(|info| {
                        let e = SpellcheckEngine::new(&info).unwrap_or_else(SpellcheckEngine::empty);
                        e
                    })
                    .unwrap_or_else(SpellcheckEngine::empty)
            };
            Some(engine)
        } else {
            None
        };
        let mut app = Self {
            doc: Document::new(),
            editor: Editor::new(),
            config,
            scroll: Scroll::default(),
            focus: false,
            dirty: false,
            message: None,
            clipboard: String::new(),
            search: Search::new(),
            dialog: None,
            menu: None,
            word_count: 0,
            view_height: 24,
            view_width: 80,
            exit: false,
            last_backup: Instant::now(),
            backup_dirty: false,
            recovery: None,
            active_panel: None,
            notes: NotesState::new(),
            ideas: IdeasState::new(),
            spellcheck: SpellcheckState::new(),
            project: None,
            project_mode: ProjectMode::Overview,
            session: Session::new(0),
            replace_active: false,
            spellcheck_engine,
            spellcheck_dirty: false,
            creative: None,
            creative_state: CreativeState::new(),
            pending_project_title: None,
        };
        if let Some(name) = file {
            let path = Path::new(&name);
            match Document::open(path) {
                Ok(doc) => app.doc = doc,
                Err(e) => {
                    app.doc = Document::create(path);
                    if e.kind() != std::io::ErrorKind::NotFound {
                        app.error(format!(
                            "No se pudo abrir \"{}\".\n\nMotivo:\n{e}",
                            path.display()
                        ));
                    }
                }
            }
        }
        app.refresh_word_count();
        app.session = Session::new(app.word_count);
        app.detect_and_load_project();
        app.check_recovery();
        app
    }

    pub fn refresh_word_count(&mut self) {
        self.word_count = self.doc.word_count();
    }

    fn mark_dirty(&mut self) {
        self.dirty = true;
        self.backup_dirty = true;
        self.spellcheck_dirty = true;
        self.refresh_word_count();
    }

    pub fn error(&mut self, text: String) {
        self.dialog = Some(Dialog::Message {
            title: "Error".into(),
            text,
        });
    }

    // ── Manejo de eventos ──

    pub fn handle_key(&mut self, key: KeyEvent) {
        if key.kind == KeyEventKind::Release {
            return;
        }
        if let Some(dialog) = self.dialog.clone() {
            self.handle_dialog_key(key, dialog);
            return;
        }
        if let Some((open, selected)) = self.menu {
            self.handle_menu_key(key, open, selected);
            return;
        }

        // ── Panel activo: captura total ──
        if self.active_panel.is_some() {
            self.handle_panel_key(key);
            return;
        }

        // ── Búsqueda activa ──
        if self.search.active {
            self.handle_search_key(key);
            return;
        }

        // ── Reemplazo activo ──
        if self.replace_active {
            self.handle_replace_key(key);
            return;
        }

        // ── Resolver comando ──
        self.message = None;
        let cmd = command::resolve(key);
        self.execute(cmd);
    }

    pub fn handle_paste(&mut self, text: String) {
        if self.dialog.is_some() || self.search.active || self.active_panel.is_some() {
            return;
        }
        if text.is_empty() {
            return;
        }
        self.editor.insert(&mut self.doc, &text);
        self.editor.seal();
        self.mark_dirty();
    }

    // ── Ejecución de comandos ──

    fn execute(&mut self, cmd: Command) {
        match cmd {
            Command::InsertChar(c) => self.insert_char_wrapped(c),
            Command::InsertNewline => {
                self.editor.insert(&mut self.doc, "\n");
                self.mark_dirty();
            }
            Command::InsertTab => {
                let w = self.config.tab_width.max(1);
                let spaces = " ".repeat(w);
                self.editor.insert(&mut self.doc, &spaces);
                self.mark_dirty();
            }
            Command::Backspace => {
                self.editor.backspace(&mut self.doc);
                self.mark_dirty();
            }
            Command::Delete => {
                self.editor.delete(&mut self.doc);
                self.mark_dirty();
            }
            Command::Copy => self.copy(),
            Command::Cut => self.cut(),
            Command::Paste => self.paste(),
            Command::Undo => {
                if self.editor.undo(&mut self.doc) {
                    self.mark_dirty();
                }
            }
            Command::Redo => {
                if self.editor.redo(&mut self.doc) {
                    self.mark_dirty();
                }
            }
            Command::MoveLeft => self.editor.move_left(self.doc.rope(), false),
            Command::MoveRight => self.editor.move_right(self.doc.rope(), false),
            Command::MoveUp => self.editor.move_up(self.doc.rope(), false),
            Command::MoveDown => self.editor.move_down(self.doc.rope(), false),
            Command::MoveHome => self.editor.move_home(self.doc.rope(), false),
            Command::MoveEnd => self.editor.move_end(self.doc.rope(), false),
            Command::PageUp => self.editor.page_up(self.doc.rope(), false, self.view_height),
            Command::PageDown => self.editor.page_down(self.doc.rope(), false, self.view_height),
            Command::MoveWordLeft => self.editor.move_word_left(self.doc.rope(), false),
            Command::MoveWordRight => self.editor.move_word_right(self.doc.rope(), false),
            Command::MoveDocStart => self.editor.move_doc_start(self.doc.rope(), false),
            Command::MoveDocEnd => self.editor.move_doc_end(self.doc.rope(), false),
            Command::MoveParaUp => self.editor.move_para_up(self.doc.rope(), false),
            Command::MoveParaDown => self.editor.move_para_down(self.doc.rope(), false),
            Command::GoToLine => self.open_go_to_line(),
            Command::SelectLeft => self.editor.move_left(self.doc.rope(), true),
            Command::SelectRight => self.editor.move_right(self.doc.rope(), true),
            Command::SelectUp => self.editor.move_up(self.doc.rope(), true),
            Command::SelectDown => self.editor.move_down(self.doc.rope(), true),
            Command::SelectHome => self.editor.move_home(self.doc.rope(), true),
            Command::SelectEnd => self.editor.move_end(self.doc.rope(), true),
            Command::SelectPageUp => self.editor.page_up(self.doc.rope(), true, self.view_height),
            Command::SelectPageDown => self.editor.page_down(self.doc.rope(), true, self.view_height),
            Command::SelectWordLeft => self.editor.move_word_left(self.doc.rope(), true),
            Command::SelectWordRight => self.editor.move_word_right(self.doc.rope(), true),
            Command::SelectAll => self.editor.select_all(self.doc.rope()),
            Command::Save => self.save(),
            Command::SaveAs => self.save_as(),
            Command::Open => self.open(),
            Command::Quit => self.quit(),
            Command::Search => self.find(),
            Command::SearchClose => self.search.close(),
            Command::SearchNext => self.search_next(),
            Command::SearchPrev => self.search_prev(),
            Command::SearchChar(c) => {
                self.search.push_char(c);
                self.search_update();
            }
            Command::SearchBackspace => {
                self.search.pop_char();
                self.search_update();
            }
            Command::ReplaceOpen => self.open_replace(),
            Command::ReplaceNext => self.replace_next(),
            Command::ReplaceAll => self.replace_all(),
            Command::ReplaceAccept => self.replace_next(),
            Command::ReplaceChar(c) => {
                self.search.push_replace_char(c);
            }
            Command::ReplaceBackspace => {
                self.search.pop_replace_char();
            }
            Command::ToggleNotes => self.toggle_panel(PanelKind::Notes),
            Command::ToggleSpellcheck => self.toggle_panel(PanelKind::Spellcheck),
            Command::ToggleIdeas => self.toggle_panel(PanelKind::Ideas),
            Command::ToggleProject => self.toggle_panel(PanelKind::Creative),
            Command::NewProject => {
                self.dialog = Some(Dialog::Input {
                    title: "Nombre del proyecto:".into(),
                    value: String::new(),
                    action: InputAction::NewProjectTitle,
                });
            }
            Command::ToggleFocus => self.toggle_focus(),
            Command::MenuToggle => {
                self.menu = if self.menu.is_some() { None } else { Some((MenuId::File, 0)) };
            }
            Command::MenuLeft => {
                if let Some((open, _)) = self.menu {
                    let order = MenuId::ALL;
                    let idx = order.iter().position(|m| *m == open).unwrap_or(0);
                    self.menu = Some((order[(idx + order.len() - 1) % order.len()], 0));
                }
            }
            Command::MenuRight => {
                if let Some((open, _)) = self.menu {
                    let order = MenuId::ALL;
                    let idx = order.iter().position(|m| *m == open).unwrap_or(0);
                    self.menu = Some((order[(idx + 1) % order.len()], 0));
                }
            }
            Command::MenuUp => {
                if let Some((open, sel)) = self.menu {
                    self.menu = Some((open, sel.saturating_sub(1)));
                }
            }
            Command::MenuDown => {
                if let Some((open, sel)) = self.menu {
                    let items = open.items();
                    self.menu = Some((open, (sel + 1).min(items.len().saturating_sub(1))));
                }
            }
            Command::MenuHome => {
                if let Some((open, _)) = self.menu {
                    self.menu = Some((open, 0));
                }
            }
            Command::MenuEnd => {
                if let Some((open, _)) = self.menu {
                    self.menu = Some((open, open.items().len().saturating_sub(1)));
                }
            }
            Command::MenuEnter => {
                if let Some((open, sel)) = self.menu {
                    let action = open.items()[sel].action;
                    self.menu = None;
                    self.run_menu_action(action);
                }
            }
            Command::MenuClose => {
                self.menu = None;
            }
            Command::MenuAlt(c) => {
                if let Some(id) = MenuId::from_accel(c) {
                    self.menu = Some((id, 0));
                }
            }
            Command::PanelUp => {
                if self.active_panel == Some(PanelKind::Notes) {
                    self.notes.move_up();
                } else if self.active_panel == Some(PanelKind::Ideas) {
                    self.ideas.move_up();
                }
            }
            Command::PanelDown => {
                if self.active_panel == Some(PanelKind::Notes) {
                    self.notes.move_down();
                } else if self.active_panel == Some(PanelKind::Ideas) {
                    self.ideas.move_down();
                }
            }
            Command::PanelEnter => {
                if self.active_panel == Some(PanelKind::Notes) {
                    if self.notes.editing {
                        self.notes.commit_edit();
                    } else if !self.notes.notes.is_empty() {
                        self.notes.start_edit();
                    } else {
                        self.notes.add_note();
                    }
                } else if self.active_panel == Some(PanelKind::Ideas) {
                    if self.ideas.editing {
                        self.ideas.commit_edit();
                    } else if !self.ideas.ideas.is_empty() {
                        self.ideas.start_edit();
                    } else {
                        self.ideas.add_idea();
                    }
                }
            }
            Command::PanelDelete => {
                if self.active_panel == Some(PanelKind::Notes) {
                    self.notes.delete_selected();
                } else if self.active_panel == Some(PanelKind::Ideas) {
                    self.ideas.delete_selected();
                }
            }
            Command::SpellcheckNextError => self.spellcheck_next_error(),
            Command::SpellcheckPrevError => self.spellcheck_prev_error(),
            Command::PanelChar(_) | Command::PanelBackspace
            | Command::SpellcheckSelectSuggestion | Command::SpellcheckAddWord
            | Command::SpellcheckIgnore | Command::SpellcheckChangeLang
            | Command::DialogConfirm | Command::DialogCancel => {}
            Command::Noop => {}
        }
    }

    // ── Edición con ajuste de línea ──

    fn insert_char_wrapped(&mut self, c: char) {
        let w = self.view_width.max(1);
        let rope = self.doc.rope();
        let line = rope.char_to_line(self.editor.cursor());
        let line_start = rope.line_to_char(line);
        let col_chars = self.editor.cursor() - line_start;
        let s = rope.line(line).to_string();
        let col = visual_col(&s, col_chars, self.config.tab_width);
        let char_w = char_cell_width(c, self.config.tab_width, col);
        let text = if col + char_w >= w {
            format!("\n{c}")
        } else {
            c.to_string()
        };
        self.editor.insert(&mut self.doc, &text);
        self.mark_dirty();
    }

    // ── Comandos de archivo ──

    fn save(&mut self) {
        match self.doc.path().map(|p| p.to_path_buf()) {
            Some(path) => match self.doc.save(&path) {
                Ok(_) => {
                    self.dirty = false;
                    self.backup_dirty = false;
                    self.last_backup = Instant::now();
                    backup::remove(&path);
                    backup::remove_unsaved();
                    self.message = Some(format!("Guardado: {}", path.display()));
                }
                Err(e) => self.error(format!(
                    "No se pudo guardar \"{}\".\n\nMotivo:\n{e}",
                    path.display()
                )),
            },
            None => self.save_as(),
        }
    }

    fn save_as(&mut self) {
        let value = self.doc.path()
            .map(|p| p.to_string_lossy().into_owned())
            .unwrap_or_else(|| "documento.txt".into());
        self.dialog = Some(Dialog::Input {
            title: "Guardar como".into(),
            value,
            action: InputAction::SaveAs,
        });
    }

    fn open(&mut self) {
        if self.dirty {
            self.dialog = Some(Dialog::Confirm {
                message: "Hay cambios sin guardar.\n¿Abrir otro archivo?".into(),
                action: ConfirmAction::Open,
            });
        } else {
            self.open_dialog();
        }
    }

    fn open_dialog(&mut self) {
        let start = self.doc.path()
            .and_then(|p| p.parent().map(|q| q.to_path_buf()))
            .filter(|p| !p.as_os_str().is_empty())
            .or_else(|| std::env::current_dir().ok())
            .unwrap_or_else(|| PathBuf::from("."));
        self.dialog = Some(Dialog::OpenBrowser(FileBrowser::open(start)));
    }

    fn open_path(&mut self, path: PathBuf) {
        match Document::open(&path) {
            Ok(doc) => {
                self.doc = doc;
                self.reset_after_load();
                backup::remove_unsaved();
                self.backup_dirty = false;
                self.last_backup = Instant::now();
                self.detect_and_load_project();
                self.message = Some(format!("Abierto: {}", path.display()));
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                self.doc = Document::create(&path);
                self.reset_after_load();
                backup::remove_unsaved();
                self.backup_dirty = false;
                self.last_backup = Instant::now();
                self.message = Some(format!("Nuevo documento: {}", path.display()));
            }
            Err(e) => self.error(format!(
                "No se pudo abrir \"{}\".\n\nMotivo:\n{e}",
                path.display()
            )),
        }
    }

    fn reset_after_load(&mut self) {
        self.editor = Editor::new();
        self.scroll = Scroll::default();
        self.clipboard.clear();
        self.dirty = false;
        self.refresh_word_count();
        self.session = Session::new(self.word_count);
    }

    fn quit(&mut self) {
        if self.dirty {
            self.dialog = Some(Dialog::Confirm {
                message: "Hay cambios sin guardar.\n¿Salir?".into(),
                action: ConfirmAction::Quit,
            });
        } else {
            self.exit = true;
        }
    }

    // ── Comandos de edición ──

    fn undo(&mut self) {
        if self.editor.undo(&mut self.doc) {
            self.mark_dirty();
        }
    }

    fn redo(&mut self) {
        if self.editor.redo(&mut self.doc) {
            self.mark_dirty();
        }
    }

    fn copy(&mut self) {
        if let Some(text) = self.editor.copy(&self.doc) {
            self.clipboard = text;
            self.message = Some("Copiado".into());
        }
    }

    fn cut(&mut self) {
        if let Some(text) = self.editor.cut(&mut self.doc) {
            self.clipboard = text;
            self.mark_dirty();
        }
    }

    fn paste(&mut self) {
        let text = self.clipboard.clone();
        if !text.is_empty() {
            self.editor.insert(&mut self.doc, &text);
            self.editor.seal();
            self.mark_dirty();
        }
    }

    // ── Búsqueda ──

    fn find(&mut self) {
        self.search.open();
        self.replace_active = false;
    }

    fn search_next(&mut self) {
        if let Some(pos) = self.search.find_next(self.doc.rope(), self.editor.cursor()) {
            self.editor.set_cursor(pos);
            self.set_search_match(pos);
        } else {
            self.search.match_range = None;
            self.message = Some(format!("No se encontró: \"{}\"", self.search.query));
        }
    }

    fn search_prev(&mut self) {
        if let Some(pos) = self.search.find_previous(self.doc.rope(), self.editor.cursor()) {
            self.editor.set_cursor(pos);
            self.set_search_match(pos);
        } else {
            self.search.match_range = None;
            self.message = Some(format!("No se encontró: \"{}\"", self.search.query));
        }
    }

    fn search_update(&mut self) {
        if let Some(pos) = self.search.find_next(self.doc.rope(), self.editor.cursor()) {
            self.editor.set_cursor(pos);
            self.set_search_match(pos);
        } else {
            self.search.match_range = None;
        }
    }

    fn set_search_match(&mut self, pos: usize) {
        let len = self.search.query.chars().count();
        self.search.match_range = Some((pos, pos + len));
    }

    // ── Reemplazo ──

    fn open_replace(&mut self) {
        self.search.open();
        self.replace_active = true;
    }

    fn replace_next(&mut self) {
        if self.search.query.is_empty() {
            return;
        }
        if let Some(pos) = self.search.replace_current(self.doc.rope_mut(), self.editor.cursor()) {
            self.editor.set_cursor(pos);
            self.set_search_match(pos);
            self.mark_dirty();
            self.search_next();
        } else {
            self.message = Some(format!("No se encontró: \"{}\"", self.search.query));
        }
    }

    fn replace_all(&mut self) {
        if self.search.query.is_empty() {
            return;
        }
        let count = self.search.replace_all(self.doc.rope_mut());
        if count > 0 {
            self.mark_dirty();
            self.message = Some(format!("Reemplazadas {count} ocurrencias"));
            self.search.match_range = None;
        } else {
            self.message = Some(format!("No se encontró: \"{}\"", self.search.query));
        }
    }

    // ── Focus ──

    fn toggle_focus(&mut self) {
        self.focus = !self.focus;
    }

    // ── Paneles ──

    fn toggle_panel(&mut self, kind: PanelKind) {
        if self.active_panel == Some(kind) {
            self.close_panel();
        } else {
            self.close_panel();
            self.active_panel = Some(kind);
            if kind == PanelKind::Spellcheck {
                self.spellcheck.reset();
                if let Some(ref mut engine) = self.spellcheck_engine {
                    let text = self.doc.rope().to_string();
                    engine.check_document(&text);
                    if !engine.has_dictionary() {
                        self.spellcheck.mode = SpellcheckMode::Errors;
                    }
                }
            }
            if kind == PanelKind::Creative {
                self.load_creative();
                self.creative_state.back_to_menu();
            }
        }
    }

    fn close_panel(&mut self) {
        if self.notes.editing {
            self.notes.commit_edit();
        }
        if self.ideas.editing {
            self.ideas.commit_edit();
        }
        self.notes.save_to_file();
        self.ideas.save_to_file();
        self.save_creative();
        self.active_panel = None;
    }

    // ── Creativo ──

    fn creative_path(&self) -> Option<PathBuf> {
        self.project.as_ref().map(|p| {
            p.root().join(crate::creative::CREATIVE_FILE)
        })
    }

    fn load_creative(&mut self) {
        if self.creative.is_some() {
            return;
        }
        if let Some(path) = self.creative_path() {
            match crate::creative::CreativeContext::load(&path) {
                Ok(ctx) => self.creative = Some(ctx),
                Err(_) => self.creative = Some(crate::creative::CreativeContext::new()),
            }
        }
    }

    fn save_creative(&self) {
        if let (Some(ctx), Some(path)) = (&self.creative, self.creative_path()) {
            let _ = ctx.save(&path);
        }
    }

    fn detect_and_load_project(&mut self) {
        if self.project.is_some() {
            return;
        }
        if let Some(file_path) = self.doc.path().map(|p| p.to_path_buf()) {
            if let Some(root) = crate::project::Project::detect_root_for_file(&file_path) {
                if let Ok(project) = crate::project::Project::open(root) {
                    self.project = Some(project);
                }
            }
        }
    }

    fn create_project(&mut self, title: String, author: String) {
        let base = self.doc.path()
            .and_then(|p| p.parent().map(|q| q.to_path_buf()))
            .or_else(|| std::env::current_dir().ok())
            .unwrap_or_else(|| PathBuf::from("."));

        let dir_name = if title.is_empty() { "mi-proyecto".to_string() }
        else {
            title.to_lowercase()
                .chars()
                .map(|c| if c.is_ascii_alphanumeric() || c == ' ' || c == '-' { c } else { '-' })
                .collect::<String>()
                .split_whitespace()
                .collect::<Vec<_>>()
                .join("-")
        };

        let root = base.join(&dir_name);
        match crate::project::Project::create(root, title, author) {
            Ok(project) => {
                self.project = Some(project);
                self.creative = Some(crate::creative::CreativeContext::new());
                self.message = Some("Proyecto creado".into());
            }
            Err(e) => self.error(format!("No se pudo crear el proyecto.\n\nMotivo:\n{e}")),
        }
    }

    fn open_go_to_line(&mut self) {
        let total = self.doc.rope().len_lines();
        self.dialog = Some(Dialog::Input {
            title: format!("Ir a línea (1–{total})"),
            value: String::new(),
            action: InputAction::GoToLine,
        });
    }

    // ── Manejo de panel ──

    fn handle_panel_key(&mut self, key: KeyEvent) {
        let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
        let alt = key.modifiers.contains(KeyModifiers::ALT);

        if ctrl || alt {
            self.execute(command::resolve(key));
            return;
        }

        let panel = self.active_panel;
        match panel {
            Some(PanelKind::Notes) => {
                if self.notes.editing {
                    match key.code {
                        KeyCode::Esc => { self.notes.editing = false; self.notes.editing_text.clear(); }
                        KeyCode::Enter => self.notes.commit_edit(),
                        KeyCode::Backspace => self.notes.pop_char(),
                        KeyCode::Char(c) => self.notes.push_char(c),
                        _ => {}
                    }
                } else {
                    match key.code {
                        KeyCode::Esc => self.close_panel(),
                        KeyCode::Char('n') => self.notes.add_note(),
                        KeyCode::Char('d') | KeyCode::Delete => self.notes.delete_selected(),
                        KeyCode::Up => self.notes.move_up(),
                        KeyCode::Down => self.notes.move_down(),
                        KeyCode::Enter => {
                            if self.notes.notes.is_empty() { self.notes.add_note(); }
                            else { self.notes.start_edit(); }
                        }
                        _ => {}
                    }
                }
            }
            Some(PanelKind::Ideas) => {
                if self.ideas.editing {
                    match key.code {
                        KeyCode::Esc => { self.ideas.editing = false; self.ideas.editing_text.clear(); }
                        KeyCode::Enter => self.ideas.commit_edit(),
                        KeyCode::Backspace => self.ideas.pop_char(),
                        KeyCode::Char(c) => self.ideas.push_char(c),
                        _ => {}
                    }
                } else {
                    match key.code {
                        KeyCode::Esc => self.close_panel(),
                        KeyCode::Char('n') => self.ideas.add_idea(),
                        KeyCode::Char('d') | KeyCode::Delete => self.ideas.delete_selected(),
                        KeyCode::Up => self.ideas.move_up(),
                        KeyCode::Down => self.ideas.move_down(),
                        KeyCode::Enter => {
                            if self.ideas.ideas.is_empty() { self.ideas.add_idea(); }
                            else { self.ideas.start_edit(); }
                        }
                        _ => {}
                    }
                }
            }
            Some(PanelKind::Spellcheck) => {
                match self.spellcheck.mode {
                    SpellcheckMode::Errors => {
                        let error_count = self.spellcheck_engine.as_ref().map(|e| e.errors.len()).unwrap_or(0);
                        match key.code {
                            KeyCode::Esc | KeyCode::F(3) => self.close_panel(),
                            KeyCode::Up => {
                                if self.spellcheck.selected > 0 {
                                    self.spellcheck.selected -= 1;
                                }
                            }
                            KeyCode::Down => {
                                if self.spellcheck.selected + 1 < error_count {
                                    self.spellcheck.selected += 1;
                                }
                            }
                            KeyCode::Enter => {
                                if let Some(ref engine) = self.spellcheck_engine {
                                    if let Some(error) = engine.errors.get(self.spellcheck.selected) {
                                        let word = error.word.clone();
                                        let mut suggestions = Vec::new();
                                        engine.suggest(&word, &mut suggestions);
                                        self.spellcheck.suggestions = suggestions;
                                        self.spellcheck.suggestion_selected = 0;
                                        self.spellcheck.mode = SpellcheckMode::Suggestions;
                                    }
                                }
                            }
                            KeyCode::Char('l') | KeyCode::Char('L') => {
                                self.spellcheck.mode = SpellcheckMode::LanguageSelect;
                                self.spellcheck.lang_selected = 0;
                            }
                            _ => {}
                        }
                    }
                    SpellcheckMode::Suggestions => {
                        match key.code {
                            KeyCode::Esc => {
                                self.spellcheck.mode = SpellcheckMode::Errors;
                                self.spellcheck.suggestions.clear();
                            }
                            KeyCode::Up => {
                                if self.spellcheck.suggestion_selected > 0 {
                                    self.spellcheck.suggestion_selected -= 1;
                                }
                            }
                            KeyCode::Down => {
                                let max = self.spellcheck.suggestions.len();
                                if self.spellcheck.suggestion_selected + 1 < max {
                                    self.spellcheck.suggestion_selected += 1;
                                }
                            }
                            KeyCode::Enter => {
                                if let Some(suggestion) = self.spellcheck.suggestions.get(self.spellcheck.suggestion_selected).cloned() {
                                    let error_info = self.spellcheck_engine.as_ref()
                                        .and_then(|e| e.errors.get(self.spellcheck.selected))
                                        .cloned();
                                    if let Some(error) = error_info {
                                        if let Some(ref mut engine) = self.spellcheck_engine {
                                            let mut text = self.doc.rope().to_string();
                                            engine.replace_word_at(&mut text, &error, &suggestion);
                                            let cursor = self.editor.cursor();
                                            self.doc = Document::from_text(&text, self.doc.path().map(|p| p.to_path_buf()));
                                            self.editor.set_cursor(cursor.min(self.doc.rope().len_chars()));
                                        }
                                        self.mark_dirty();
                                        if let Some(ref mut engine) = self.spellcheck_engine {
                                            let text = self.doc.rope().to_string();
                                            engine.check_document(&text);
                                        }
                                    }
                                    self.spellcheck.mode = SpellcheckMode::Errors;
                                    self.spellcheck.suggestions.clear();
                                    if self.spellcheck.selected >= self.spellcheck_engine.as_ref().map(|e| e.errors.len()).unwrap_or(0) {
                                        self.spellcheck.selected = self.spellcheck.selected.saturating_sub(1);
                                    }
                                }
                            }
                            KeyCode::Char('a') | KeyCode::Char('A') => {
                                if let Some(error) = self.spellcheck_engine.as_ref().and_then(|e| e.errors.get(self.spellcheck.selected)) {
                                    let word = error.word.clone();
                                    if let Some(ref mut engine) = self.spellcheck_engine {
                                        engine.add_to_personal(&word);
                                        let text = self.doc.rope().to_string();
                                        engine.check_document(&text);
                                    }
                                }
                                self.spellcheck.mode = SpellcheckMode::Errors;
                                self.spellcheck.suggestions.clear();
                                if self.spellcheck.selected >= self.spellcheck_engine.as_ref().map(|e| e.errors.len()).unwrap_or(0) {
                                    self.spellcheck.selected = self.spellcheck.selected.saturating_sub(1);
                                }
                            }
                            KeyCode::Char('i') | KeyCode::Char('I') => {
                                if let Some(error) = self.spellcheck_engine.as_ref().and_then(|e| e.errors.get(self.spellcheck.selected)) {
                                    let word = error.word.clone();
                                    if let Some(ref mut engine) = self.spellcheck_engine {
                                        engine.ignore_word(&word);
                                        let text = self.doc.rope().to_string();
                                        engine.check_document(&text);
                                    }
                                }
                                self.spellcheck.mode = SpellcheckMode::Errors;
                                self.spellcheck.suggestions.clear();
                                if self.spellcheck.selected >= self.spellcheck_engine.as_ref().map(|e| e.errors.len()).unwrap_or(0) {
                                    self.spellcheck.selected = self.spellcheck.selected.saturating_sub(1);
                                }
                            }
                            _ => {}
                        }
                    }
                    SpellcheckMode::LanguageSelect => {
                        let lang_count = self.spellcheck_engine.as_ref().map(|e| e.available_langs.len()).unwrap_or(0);
                        match key.code {
                            KeyCode::Esc => {
                                self.spellcheck.mode = SpellcheckMode::Errors;
                            }
                            KeyCode::Up => {
                                if self.spellcheck.lang_selected > 0 {
                                    self.spellcheck.lang_selected -= 1;
                                }
                            }
                            KeyCode::Down => {
                                if self.spellcheck.lang_selected + 1 < lang_count {
                                    self.spellcheck.lang_selected += 1;
                                }
                            }
                            KeyCode::Enter => {
                                if let Some(ref mut engine) = self.spellcheck_engine {
                                    if let Some(info) = engine.available_langs.get(self.spellcheck.lang_selected).cloned() {
                                        engine.switch_dictionary(&info);
                                        self.config.language = info.language;
                                        let _ = self.config.save();
                                        let text = self.doc.rope().to_string();
                                        engine.check_document(&text);
                                    }
                                }
                                self.spellcheck.mode = SpellcheckMode::Errors;
                            }
                            KeyCode::Char('a') | KeyCode::Char('A') => {
                                if let Some(ref mut engine) = self.spellcheck_engine {
                                    self.config.language = "auto".into();
                                    let _ = self.config.save();
                                    if let Some(info) = spellcheck::auto_detect_dictionary() {
                                        engine.switch_dictionary(&info);
                                        let text = self.doc.rope().to_string();
                                        engine.check_document(&text);
                                    }
                                }
                                self.spellcheck.mode = SpellcheckMode::Errors;
                            }
                            _ => {}
                        }
                    }
                }
            }
            Some(PanelKind::Creative) => {
                self.handle_creative_key(key);
            }
            None => {}
        }
    }

    // ── Manejo de búsqueda ──

    fn handle_creative_key(&mut self, key: KeyEvent) {
        match self.creative_state.mode {
            CreativeMode::Menu => {
                match key.code {
                    KeyCode::Esc | KeyCode::F(5) => self.close_panel(),
                    KeyCode::Up => {
                        let count = CreativeSection::all().len();
                        if self.creative_state.selected > 0 {
                            self.creative_state.selected -= 1;
                        }
                    }
                    KeyCode::Down => {
                        let count = CreativeSection::all().len();
                        if self.creative_state.selected + 1 < count {
                            self.creative_state.selected += 1;
                        }
                    }
                    KeyCode::Enter => {
                        if let Some(section) = CreativeSection::all().get(self.creative_state.selected) {
                            self.creative_state.open_section(*section);
                        }
                    }
                    KeyCode::Char('1') => self.creative_state.open_section(CreativeSection::Chapters),
                    KeyCode::Char('2') => self.creative_state.open_section(CreativeSection::Characters),
                    KeyCode::Char('3') => self.creative_state.open_section(CreativeSection::Places),
                    KeyCode::Char('4') => self.creative_state.open_section(CreativeSection::Timeline),
                    KeyCode::Char('5') => self.creative_state.open_section(CreativeSection::Concepts),
                    KeyCode::Char('6') => self.creative_state.open_section(CreativeSection::Statistics),
                    _ => {}
                }
            }
            CreativeMode::List => {
                match key.code {
                    KeyCode::Esc | KeyCode::F(5) => {
                        self.save_creative();
                        self.creative_state.back_to_menu();
                    }
                    KeyCode::Up => {
                        let max = self.creative_item_count();
                        self.creative_state.move_up(max);
                    }
                    KeyCode::Down => {
                        let max = self.creative_item_count();
                        self.creative_state.move_down(max);
                    }
                    KeyCode::Char('n') | KeyCode::Char('N') => {
                        self.creative_new_item();
                    }
                    KeyCode::Char('e') | KeyCode::Char('E') => {
                        self.creative_start_edit();
                    }
                    KeyCode::Char('d') | KeyCode::Char('D') | KeyCode::Delete => {
                        self.creative_state.confirm_delete();
                    }
                    KeyCode::Char('t') | KeyCode::Char('T') => {
                        self.creative_cycle_chapter_state();
                    }
                    KeyCode::Enter => {
                        self.creative_start_edit();
                    }
                    _ => {}
                }
            }
            CreativeMode::ConfirmDelete => {
                match key.code {
                    KeyCode::Char('y') | KeyCode::Char('Y') => {
                        self.creative_delete_item();
                        self.creative_state.mode = CreativeMode::List;
                    }
                    _ => {
                        self.creative_state.mode = CreativeMode::List;
                    }
                }
            }
            CreativeMode::EditName | CreativeMode::EditDescription | CreativeMode::EditNotes => {
                match key.code {
                    KeyCode::Esc => {
                        self.creative_state.cancel();
                    }
                    KeyCode::Enter => {
                        self.creative_commit_edit();
                    }
                    KeyCode::Backspace => {
                        self.creative_state.pop_char();
                    }
                    KeyCode::Char(c) => {
                        self.creative_state.push_char(c);
                    }
                    _ => {}
                }
            }
        }
    }

    fn creative_item_count(&self) -> usize {
        let ctx = match &self.creative {
            Some(c) => c,
            None => return 0,
        };
        match self.creative_state.section {
            CreativeSection::Chapters => self.project.as_ref().map(|p| p.chapters().len()).unwrap_or(0),
            CreativeSection::Characters => ctx.character_count(),
            CreativeSection::Places => ctx.place_count(),
            CreativeSection::Timeline => ctx.event_count(),
            CreativeSection::Concepts => ctx.concept_count(),
            CreativeSection::Statistics => 0,
        }
    }

    fn creative_new_item(&mut self) {
        use crate::creative::*;
        let ctx = match &mut self.creative {
            Some(c) => c,
            None => return,
        };
        match self.creative_state.section {
            CreativeSection::Characters => {
                let id = format!("ch_{}", std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_nanos());
                let c = Character::new(id, String::new());
                ctx.characters.push(c);
                self.creative_state.selected = ctx.characters.len() - 1;
                self.creative_state.start_edit(String::new());
            }
            CreativeSection::Places => {
                let id = format!("pl_{}", std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_nanos());
                let p = Place::new(id, String::new());
                ctx.places.push(p);
                self.creative_state.selected = ctx.places.len() - 1;
                self.creative_state.start_edit(String::new());
            }
            CreativeSection::Concepts => {
                let id = format!("co_{}", std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_nanos());
                let c = Concept::new(id, String::new());
                ctx.concepts.push(c);
                self.creative_state.selected = ctx.concepts.len() - 1;
                self.creative_state.start_edit(String::new());
            }
            CreativeSection::Timeline => {
                let id = format!("te_{}", std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_nanos());
                let e = TimelineEvent::new(id, String::new(), 0);
                ctx.timeline.push(e);
                self.creative_state.selected = ctx.timeline.len() - 1;
                self.creative_state.start_edit(String::new());
            }
            _ => {}
        }
    }

    fn creative_start_edit(&mut self) {
        let text = match self.creative_state.section {
            CreativeSection::Characters => {
                self.creative.as_ref()
                    .and_then(|c| c.characters.get(self.creative_state.selected))
                    .map(|c| c.name.clone())
                    .unwrap_or_default()
            }
            CreativeSection::Places => {
                self.creative.as_ref()
                    .and_then(|c| c.places.get(self.creative_state.selected))
                    .map(|p| p.name.clone())
                    .unwrap_or_default()
            }
            CreativeSection::Concepts => {
                self.creative.as_ref()
                    .and_then(|c| c.concepts.get(self.creative_state.selected))
                    .map(|c| c.name.clone())
                    .unwrap_or_default()
            }
            CreativeSection::Timeline => {
                self.creative.as_ref()
                    .and_then(|c| c.timeline.get(self.creative_state.selected))
                    .map(|e| e.label.clone())
                    .unwrap_or_default()
            }
            _ => return,
        };
        self.creative_state.start_edit(text);
    }

    fn creative_commit_edit(&mut self) {
        let new_text = self.creative_state.edit_buffer.clone();
        if let Some(ctx) = &mut self.creative {
            match self.creative_state.section {
                CreativeSection::Characters => {
                    if let Some(ch) = ctx.characters.get_mut(self.creative_state.selected) {
                        ch.name = new_text;
                    }
                }
                CreativeSection::Places => {
                    if let Some(p) = ctx.places.get_mut(self.creative_state.selected) {
                        p.name = new_text;
                    }
                }
                CreativeSection::Concepts => {
                    if let Some(c) = ctx.concepts.get_mut(self.creative_state.selected) {
                        c.name = new_text;
                    }
                }
                CreativeSection::Timeline => {
                    if let Some(e) = ctx.timeline.get_mut(self.creative_state.selected) {
                        e.label = new_text;
                    }
                }
                _ => {}
            }
        }
        self.creative_state.edit_buffer.clear();
        self.creative_state.mode = CreativeMode::List;
    }

    fn creative_delete_item(&mut self) {
        let ctx = match &mut self.creative {
            Some(c) => c,
            None => return,
        };
        let idx = self.creative_state.selected;
        match self.creative_state.section {
            CreativeSection::Characters => {
                if idx < ctx.characters.len() {
                    ctx.characters.remove(idx);
                }
            }
            CreativeSection::Places => {
                if idx < ctx.places.len() {
                    ctx.places.remove(idx);
                }
            }
            CreativeSection::Concepts => {
                if idx < ctx.concepts.len() {
                    ctx.concepts.remove(idx);
                }
            }
            CreativeSection::Timeline => {
                if idx < ctx.timeline.len() {
                    ctx.timeline.remove(idx);
                }
            }
            _ => {}
        }
        let max = self.creative_item_count();
        if self.creative_state.selected >= max && self.creative_state.selected > 0 {
            self.creative_state.selected -= 1;
        }
    }

    fn creative_cycle_chapter_state(&mut self) {
        if self.creative_state.section != CreativeSection::Chapters {
            return;
        }
        let idx = self.creative_state.selected;
        if let Some(project) = &mut self.project {
            if idx < project.chapters().len() {
                let new_state = project.chapters()[idx].state.cycle();
                let _ = project.set_chapter_state(idx, new_state);
            }
        }
    }

    // ── Manejo de búsqueda ──

    fn handle_search_key(&mut self, key: KeyEvent) {
        let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
        match key.code {
            KeyCode::Esc => self.search.close(),
            KeyCode::Enter => self.search_next(),
            KeyCode::Up => self.search_prev(),
            KeyCode::Char(c) if !ctrl => {
                self.search.push_char(c);
                self.search_update();
            }
            KeyCode::Backspace => {
                self.search.pop_char();
                self.search_update();
            }
            _ => {}
        }
    }

    // ── Manejo de reemplazo ──

    fn handle_replace_key(&mut self, key: KeyEvent) {
        let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
        let alt = key.modifiers.contains(KeyModifiers::ALT);
        match key.code {
            KeyCode::Esc => {
                self.replace_active = false;
                self.search.close();
            }
            KeyCode::Enter if !ctrl && !alt => self.replace_next(),
            KeyCode::Char('r') if alt && !ctrl => self.replace_next(),
            KeyCode::Char('r') if ctrl && alt => self.replace_all(),
            KeyCode::Char(c) if !ctrl && !alt => self.search.push_replace_char(c),
            KeyCode::Backspace => self.search.pop_replace_char(),
            _ => {}
        }
    }

    // ── Menú ──

    fn handle_menu_key(&mut self, key: KeyEvent, open: MenuId, selected: usize) {
        let alt = key.modifiers.contains(KeyModifiers::ALT);
        match key.code {
            KeyCode::Esc | KeyCode::F(10) => self.menu = None,
            KeyCode::Left => {
                let order = MenuId::ALL;
                let idx = order.iter().position(|m| *m == open).unwrap_or(0);
                self.menu = Some((order[(idx + order.len() - 1) % order.len()], 0));
            }
            KeyCode::Right => {
                let order = MenuId::ALL;
                let idx = order.iter().position(|m| *m == open).unwrap_or(0);
                self.menu = Some((order[(idx + 1) % order.len()], 0));
            }
            KeyCode::Down => {
                let items = open.items();
                self.menu = Some((open, (selected + 1).min(items.len().saturating_sub(1))));
            }
            KeyCode::Up => self.menu = Some((open, selected.saturating_sub(1))),
            KeyCode::Home => self.menu = Some((open, 0)),
            KeyCode::End => self.menu = Some((open, open.items().len().saturating_sub(1))),
            KeyCode::Enter => {
                let action = open.items()[selected].action;
                self.menu = None;
                self.run_menu_action(action);
            }
            KeyCode::Char(c) if alt => {
                if let Some(id) = MenuId::from_accel(c.to_ascii_lowercase()) {
                    self.menu = Some((id, 0));
                }
            }
            _ => {}
        }
    }

    fn run_menu_action(&mut self, action: MenuAction) {
        match action {
            MenuAction::Open => self.open(),
            MenuAction::Save => self.save(),
            MenuAction::SaveAs => self.save_as(),
            MenuAction::Quit => self.quit(),
            MenuAction::Undo => self.undo(),
            MenuAction::Redo => self.redo(),
            MenuAction::Cut => self.cut(),
            MenuAction::Copy => self.copy(),
            MenuAction::Paste => self.paste(),
            MenuAction::Find => self.find(),
            MenuAction::Replace => self.open_replace(),
            MenuAction::GoToLine => self.open_go_to_line(),
            MenuAction::Focus => self.toggle_focus(),
            MenuAction::NewProject => {
                self.dialog = Some(Dialog::Input {
                    title: "Nombre del proyecto:".into(),
                    value: String::new(),
                    action: InputAction::NewProjectTitle,
                });
            }
        }
    }

    // ── Diálogos ──

    fn handle_dialog_key(&mut self, key: KeyEvent, dialog: Dialog) {
        let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
        match dialog {
            Dialog::Input { .. } => match key.code {
                KeyCode::Esc => self.dialog = None,
                KeyCode::Enter => {
                    if let Some(Dialog::Input { value, action, .. }) = self.dialog.clone() {
                        self.dialog = None;
                        self.commit_input(action, value);
                    }
                }
                KeyCode::Char(c) if !ctrl => {
                    if let Some(Dialog::Input { value, .. }) = &mut self.dialog {
                        value.push(c);
                    }
                }
                KeyCode::Backspace => {
                    if let Some(Dialog::Input { value, .. }) = &mut self.dialog {
                        value.pop();
                    }
                }
                _ => {}
            },
            Dialog::Confirm { action, .. } => match key.code {
                KeyCode::Enter | KeyCode::Char('s' | 'y' | 'S' | 'Y') => {
                    self.dialog = None;
                    self.confirm_continue(action);
                }
                KeyCode::Esc | KeyCode::Char('n' | 'N') => {
                    self.dialog = None;
                    self.decline_confirm(action);
                }
                _ => {}
            },
            Dialog::OpenBrowser(_) => {
                let page = self.view_height.max(10);
                match key.code {
                    KeyCode::Esc => self.dialog = None,
                    KeyCode::Enter => {
                        let path = if let Some(Dialog::OpenBrowser(browser)) = &mut self.dialog {
                            browser.activate()
                        } else {
                            None
                        };
                        if let Some(path) = path {
                            self.dialog = None;
                            self.open_path(path);
                        }
                    }
                    KeyCode::Up => { if let Some(Dialog::OpenBrowser(b)) = &mut self.dialog { b.move_up(); } }
                    KeyCode::Down => { if let Some(Dialog::OpenBrowser(b)) = &mut self.dialog { b.move_down(); } }
                    KeyCode::Home => { if let Some(Dialog::OpenBrowser(b)) = &mut self.dialog { b.move_home(); } }
                    KeyCode::End => { if let Some(Dialog::OpenBrowser(b)) = &mut self.dialog { b.move_end(); } }
                    KeyCode::PageUp => { if let Some(Dialog::OpenBrowser(b)) = &mut self.dialog { b.page_up(page); } }
                    KeyCode::PageDown => { if let Some(Dialog::OpenBrowser(b)) = &mut self.dialog { b.page_down(page); } }
                    KeyCode::Backspace => {
                        if let Some(Dialog::OpenBrowser(b)) = &mut self.dialog {
                            if b.has_filter() { b.pop_filter_char(); } else { b.go_up(); }
                        }
                    }
                    KeyCode::Char(c) if !ctrl => {
                        if let Some(Dialog::OpenBrowser(b)) = &mut self.dialog { b.push_filter_char(c); }
                    }
                    _ => {}
                }
            }
            Dialog::Message { .. } => {
                if matches!(key.code, KeyCode::Enter | KeyCode::Esc) {
                    self.dialog = None;
                }
            }
        }
    }

    fn confirm_continue(&mut self, action: ConfirmAction) {
        match action {
            ConfirmAction::Open => self.open_dialog(),
            ConfirmAction::Quit => {
                self.cleanup_backups();
                self.exit = true;
            }
            ConfirmAction::Restore => self.restore_from_backup(),
        }
    }

    fn decline_confirm(&mut self, action: ConfirmAction) {
        if action != ConfirmAction::Restore {
            return;
        }
        if let Some(b) = self.recovery.take() {
            match b.original {
                Some(p) => backup::remove(&p),
                None => backup::remove_unsaved(),
            }
        }
    }

    fn commit_input(&mut self, action: InputAction, value: String) {
        match action {
            InputAction::SaveAs => {
                if value.trim().is_empty() {
                    self.message = Some("Nombre de archivo vacío".into());
                    return;
                }
                let path = Path::new(&value);
                match self.doc.save(path) {
                    Ok(_) => {
                        let old = self.doc.path().map(|p| p.to_path_buf());
                        self.doc.set_path(path.to_path_buf());
                        if let Some(old) = old { backup::remove(&old); }
                        backup::remove_unsaved();
                        self.dirty = false;
                        self.backup_dirty = false;
                        self.last_backup = Instant::now();
                        self.message = Some(format!("Guardado como: {}", path.display()));
                    }
                    Err(e) => self.error(format!(
                        "No se pudo guardar \"{}\".\n\nMotivo:\n{e}",
                        path.display()
                    )),
                }
            }
            InputAction::GoToLine => {
                if let Ok(n) = value.trim().parse::<usize>() {
                    self.editor.go_to_line(self.doc.rope(), n);
                    self.message = Some(format!("Línea {n}"));
                } else {
                    self.message = Some("Número de línea inválido".into());
                }
            }
            InputAction::NewProjectTitle => {
                let title = if value.trim().is_empty() { "Mi proyecto".to_string() } else { value.trim().to_string() };
                self.pending_project_title = Some(title);
                self.dialog = Some(Dialog::Input {
                    title: "Autor del proyecto:".into(),
                    value: String::new(),
                    action: InputAction::NewProjectAuthor,
                });
            }
            InputAction::NewProjectAuthor => {
                let title = self.pending_project_title.take().unwrap_or_else(|| "Mi proyecto".to_string());
                let author = value.trim().to_string();
                self.create_project(title, author);
            }
        }
    }

    // ── Copia temporal / autoguardado ──

    pub fn tick(&mut self) {
        if self.backup_dirty && self.last_backup.elapsed() >= Duration::from_secs(30) {
            self.backup_now();
        }
        if self.spellcheck_dirty {
            self.spellcheck_dirty = false;
            if let Some(ref mut engine) = self.spellcheck_engine {
                let text = self.doc.rope().to_string();
                engine.check_document(&text);
            }
        }
    }

    fn backup_now(&mut self) {
        let text = self.doc.rope().to_string();
        let result = match self.doc.path() {
            Some(p) => backup::save(p, &text),
            None => backup::save_unsaved(&text),
        };
        if result.is_ok() {
            self.backup_dirty = false;
            self.last_backup = Instant::now();
        }
    }

    // ── Recuperación ──

    fn check_recovery(&mut self) {
        if self.dialog.is_some() {
            return;
        }
        let found = match self.doc.path() {
            Some(p) => backup::find(p),
            None => backup::find_unsaved(),
        };
        let Some(b) = found else { return };
        let main_newer = self.doc.path().and_then(|p| {
            std::fs::metadata(p).ok()
                .and_then(|m| m.modified().ok())
                .map(|m| b.modified <= m)
        });
        if main_newer.unwrap_or(false) {
            if let Some(p) = self.doc.path() { backup::remove(p); } else { backup::remove_unsaved(); }
            return;
        }
        let what = match &b.original {
            Some(p) => format!("\"{}\"", p.display()),
            None => "el documento sin título".into(),
        };
        self.recovery = Some(b);
        self.dialog = Some(Dialog::Confirm {
            message: format!("Se encontró una copia recuperable de {what}.\n¿Restaurar?"),
            action: ConfirmAction::Restore,
        });
    }

    fn restore_from_backup(&mut self) {
        let Some(b) = self.recovery.take() else { return };
        match std::fs::read_to_string(&b.path) {
            Ok(text) => {
                self.doc = Document::from_text(&text, b.original);
                self.editor = Editor::new();
                self.scroll = Scroll::default();
                self.clipboard.clear();
                self.dirty = true;
                self.backup_dirty = false;
                self.last_backup = Instant::now();
                self.refresh_word_count();
                self.session = Session::new(self.word_count);
                self.message = Some("Cambios restaurados desde la copia temporal".into());
            }
            Err(e) => self.error(format!("No se pudo leer la copia temporal.\n\nMotivo:\n{e}")),
        }
    }

    fn cleanup_backups(&mut self) {
        if let Some(p) = self.doc.path().map(|p| p.to_path_buf()) {
            backup::remove(&p);
        }
        backup::remove_unsaved();
    }

    // ── Scroll ──

    fn spellcheck_next_error(&mut self) {
        if let Some(ref engine) = self.spellcheck_engine {
            if engine.errors.is_empty() { return; }
            let rope = self.doc.rope();
            let cursor_line = rope.char_to_line(self.editor.cursor());
            if let Some(idx) = engine.next_error(cursor_line) {
                let error = &engine.errors[idx];
                let char_start = rope.line_to_char(error.line) + error.col;
                self.editor.set_cursor(char_start);
                self.message = Some(format!("Error {}/{}: \"{}\"", idx + 1, engine.errors.len(), error.word));
            } else {
                if let Some(idx) = engine.next_error(0) {
                    let error = &engine.errors[idx];
                    let char_start = rope.line_to_char(error.line) + error.col;
                    self.editor.set_cursor(char_start);
                    self.message = Some(format!("Error 1/{}: \"{}\" (inicio)", engine.errors.len(), error.word));
                } else {
                    self.message = Some("Sin errores ortográficos".into());
                }
            }
        }
    }

    fn spellcheck_prev_error(&mut self) {
        if let Some(ref engine) = self.spellcheck_engine {
            if engine.errors.is_empty() { return; }
            let rope = self.doc.rope();
            let cursor_line = rope.char_to_line(self.editor.cursor());
            if let Some(idx) = engine.prev_error(cursor_line) {
                let error = &engine.errors[idx];
                let char_start = rope.line_to_char(error.line) + error.col;
                self.editor.set_cursor(char_start);
                self.message = Some(format!("Error {}/{}: \"{}\"", idx + 1, engine.errors.len(), error.word));
            } else {
                if let Some(idx) = engine.prev_error(usize::MAX) {
                    let error = &engine.errors[idx];
                    let char_start = rope.line_to_char(error.line) + error.col;
                    self.editor.set_cursor(char_start);
                    self.message = Some(format!("Error {}/{}: \"{}\" (fin)", engine.errors.len(), engine.errors.len(), error.word));
                } else {
                    self.message = Some("Sin errores ortográficos".into());
                }
            }
        }
    }

    pub fn update_scroll(&mut self, width: usize, height: usize) {
        self.view_width = width;
        let rope = self.doc.rope();
        let line = rope.char_to_line(self.editor.cursor());

        if height == 0 {
            self.scroll.top = 0;
        } else {
            if line < self.scroll.top { self.scroll.top = line; }
            if line >= self.scroll.top + height { self.scroll.top = line + 1 - height; }
            let max_top = rope.len_lines().saturating_sub(height);
            self.scroll.top = self.scroll.top.min(max_top);
        }

        if width == 0 { self.scroll.left = 0; return; }
        let line_start = rope.line_to_char(line);
        let col_chars = self.editor.cursor() - line_start;
        let s = rope.line(line).to_string();
        let col = visual_col(&s, col_chars, self.config.tab_width);
        let margin = 4usize;
        if col < self.scroll.left { self.scroll.left = col.saturating_sub(margin); }
        if col >= self.scroll.left + width { self.scroll.left = col.saturating_add(margin).saturating_sub(width); }
    }
}

// ── Tests ──

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn typing_wraps_at_terminal_edge() {
        let mut app = App::new(None);
        app.view_width = 10;
        for c in "abcdefghijkl".chars() {
            app.insert_char_wrapped(c);
        }
        assert_eq!(app.doc.rope().to_string(), "abcdefghi\njkl");
    }

    #[test]
    fn wrap_respects_wide_chars() {
        let mut app = App::new(None);
        app.view_width = 5;
        for c in "你你你你你".chars() {
            app.insert_char_wrapped(c);
        }
        assert_eq!(app.doc.rope().to_string(), "你你\n你你\n你");
    }

    #[test]
    fn menu_accelerators_roundtrip() {
        for id in MenuId::ALL {
            let label = id.label();
            let idx = id.accelerator_index();
            let c = label.chars().nth(idx).unwrap().to_ascii_lowercase();
            assert_eq!(MenuId::from_accel(c), Some(id));
        }
        assert_eq!(MenuId::from_accel('x'), None);
    }

    #[test]
    fn every_menu_has_items() {
        for id in MenuId::ALL {
            assert!(!id.items().is_empty());
            for item in id.items() {
                assert!(!item.label.is_empty());
                assert!(!item.shortcut.is_empty());
            }
        }
    }

    #[test]
    fn search_sets_and_clears_match_range() {
        let mut app = App::new(None);
        app.editor.insert(&mut app.doc, "hola mundo hola");
        app.search.active = true;
        app.search.query.push('h');
        app.search_update();
        assert_eq!(app.search.match_range, Some((0, 1)));
        app.search.query.push('o');
        app.search_update();
        assert_eq!(app.search.match_range, Some((0, 2)));
        app.search.query.push('x');
        app.search_update();
        assert_eq!(app.search.match_range, None);
    }

    #[test]
    fn replace_all_via_command() {
        let mut app = App::new(None);
        app.editor.insert(&mut app.doc, "hola mundo hola");
        app.search.query = "hola".into();
        app.search.replace_text = "adiós".into();
        app.execute(Command::ReplaceAll);
        assert_eq!(app.doc.rope().to_string(), "adiós mundo adiós");
        assert!(app.dirty);
    }

    #[test]
    fn toggle_panel_opens_and_closes() {
        let mut app = App::new(None);
        assert!(app.active_panel.is_none());
        app.execute(Command::ToggleNotes);
        assert_eq!(app.active_panel, Some(PanelKind::Notes));
        app.execute(Command::ToggleNotes);
        assert!(app.active_panel.is_none());
    }

    #[test]
    fn only_one_panel_at_a_time() {
        let mut app = App::new(None);
        app.execute(Command::ToggleNotes);
        assert_eq!(app.active_panel, Some(PanelKind::Notes));
        app.execute(Command::ToggleIdeas);
        assert_eq!(app.active_panel, Some(PanelKind::Ideas));
    }

    #[test]
    fn notes_add_and_delete() {
        let mut app = App::new(None);
        app.execute(Command::ToggleNotes);
        app.notes.add_note();
        assert_eq!(app.notes.notes.len(), 1);
        app.notes.delete_selected();
        assert!(app.notes.notes.is_empty());
    }

    #[test]
    fn ideas_add_and_delete() {
        let mut app = App::new(None);
        app.execute(Command::ToggleIdeas);
        app.ideas.add_idea();
        assert_eq!(app.ideas.ideas.len(), 1);
        app.ideas.delete_selected();
        assert!(app.ideas.ideas.is_empty());
    }

    #[test]
    fn session_tracks_words() {
        let mut app = App::new(None);
        app.editor.insert(&mut app.doc, "hola mundo");
        app.refresh_word_count();
        app.session = Session::new(app.word_count);
        assert_eq!(app.session.initial_words, 2);
        app.editor.insert(&mut app.doc, " y más");
        app.refresh_word_count();
        assert_eq!(app.session.words_written(app.word_count), 2);
    }
}
