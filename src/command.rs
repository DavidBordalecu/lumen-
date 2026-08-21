use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Command {
    // ── edición ──
    InsertChar(char),
    InsertNewline,
    InsertTab,
    Backspace,
    Delete,

    // ── portapapeles ──
    Copy,
    Cut,
    Paste,

    // ── deshacer / rehacer ──
    Undo,
    Redo,

    // ── navegación ──
    MoveLeft,
    MoveRight,
    MoveUp,
    MoveDown,
    MoveHome,
    MoveEnd,
    PageUp,
    PageDown,
    MoveWordLeft,
    MoveWordRight,
    MoveDocStart,
    MoveDocEnd,
    MoveParaUp,
    MoveParaDown,
    GoToLine,

    // ── selección (Shift + movimiento) ──
    SelectLeft,
    SelectRight,
    SelectUp,
    SelectDown,
    SelectHome,
    SelectEnd,
    SelectPageUp,
    SelectPageDown,
    SelectWordLeft,
    SelectWordRight,
    SelectAll,

    // ── archivo ──
    Save,
    SaveAs,
    Open,
    Quit,

    // ── búsqueda ──
    Search,
    SearchClose,
    SearchNext,
    SearchPrev,
    SearchChar(char),
    SearchBackspace,

    // ── reemplazo ──
    ReplaceOpen,
    ReplaceNext,
    ReplaceAll,
    ReplaceAccept,
    ReplaceChar(char),
    ReplaceBackspace,

    // ── paneles contextuales ──
    ToggleNotes,
    ToggleSpellcheck,
    ToggleIdeas,
    ToggleProject,

    // ── proyecto ──
    NewProject,

    // ── focus ──
    ToggleFocus,

    // ── menú ──
    MenuToggle,
    MenuLeft,
    MenuRight,
    MenuUp,
    MenuDown,
    MenuHome,
    MenuEnd,
    MenuEnter,
    MenuClose,
    MenuAlt(char),

    // ── diálogos ──
    DialogConfirm,
    DialogCancel,

    // ── paneles: entrada ──
    PanelChar(char),
    PanelBackspace,
    PanelEnter,
    PanelUp,
    PanelDown,
    PanelDelete,

    // ── ortografía ──
    SpellcheckNextError,
    SpellcheckPrevError,
    SpellcheckSelectSuggestion,
    SpellcheckAddWord,
    SpellcheckIgnore,
    SpellcheckChangeLang,

    Noop,
}

pub fn resolve(key: KeyEvent) -> Command {
    if key.kind == crossterm::event::KeyEventKind::Release {
        return Command::Noop;
    }
    let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
    let alt = key.modifiers.contains(KeyModifiers::ALT);
    let shift = key.modifiers.contains(KeyModifiers::SHIFT);

    match key.code {
        KeyCode::Char(c) if ctrl && !alt => {
            let ch = c.to_ascii_lowercase();
            match (ch, shift) {
                ('s', false) => Command::Save,
                ('s', true) => Command::SaveAs,
                ('o', _) => Command::Open,
                ('z', _) => Command::Undo,
                ('y', _) => Command::Redo,
                ('f', false) => Command::Search,
                ('f', true) => Command::ToggleFocus,
                ('h', _) => Command::ReplaceOpen,
                ('q', _) => Command::Quit,
                ('c', _) => Command::Copy,
                ('x', _) => Command::Cut,
                ('v', _) => Command::Paste,
                ('a', _) => Command::SelectAll,
                ('g', _) => Command::GoToLine,
                ('w', _) => Command::MoveWordLeft,
                ('b', _) => Command::MoveWordRight,
                ('n', true) => Command::NewProject,
                _ => Command::Noop,
            }
        }
        KeyCode::Char(c) if alt && !ctrl => {
            let ch = c.to_ascii_lowercase();
            match ch {
                'a' | 'e' | 'b' => Command::MenuAlt(ch),
                'r' => Command::ReplaceAccept,
                _ => Command::Noop,
            }
        }
        KeyCode::Char(c) if ctrl && alt => {
            let ch = c.to_ascii_lowercase();
            match ch {
                'r' => Command::ReplaceAll,
                _ => Command::Noop,
            }
        }
        KeyCode::Char(c) if !ctrl && !alt => Command::InsertChar(c),
        KeyCode::Enter => Command::InsertNewline,
        KeyCode::Tab => Command::InsertTab,
        KeyCode::Backspace => Command::Backspace,
        KeyCode::Delete => Command::Delete,
        KeyCode::Left if !ctrl && !shift => Command::MoveLeft,
        KeyCode::Right if !ctrl && !shift => Command::MoveRight,
        KeyCode::Up if !ctrl && !shift => Command::MoveUp,
        KeyCode::Down if !ctrl && !shift => Command::MoveDown,
        KeyCode::Home if !ctrl && !shift => Command::MoveHome,
        KeyCode::End if !ctrl && !shift => Command::MoveEnd,
        KeyCode::PageUp if !shift => Command::PageUp,
        KeyCode::PageDown if !shift => Command::PageDown,
        KeyCode::Left if shift => Command::SelectLeft,
        KeyCode::Right if shift => Command::SelectRight,
        KeyCode::Up if shift => Command::SelectUp,
        KeyCode::Down if shift => Command::SelectDown,
        KeyCode::Home if shift => Command::SelectHome,
        KeyCode::End if shift => Command::SelectEnd,
        KeyCode::PageUp if shift => Command::SelectPageUp,
        KeyCode::PageDown if shift => Command::SelectPageDown,
        KeyCode::Up if ctrl => Command::MoveParaUp,
        KeyCode::Down if ctrl => Command::MoveParaDown,
        KeyCode::Home if ctrl => Command::MoveDocStart,
        KeyCode::End if ctrl => Command::MoveDocEnd,
        KeyCode::F(2) => Command::ToggleNotes,
        KeyCode::F(3) => Command::ToggleSpellcheck,
        KeyCode::F(4) => Command::ToggleIdeas,
        KeyCode::F(5) => Command::ToggleProject,
        KeyCode::F(6) => Command::Search,
        KeyCode::F(10) => Command::MenuToggle,
        KeyCode::Esc => Command::SearchClose,
        _ => Command::Noop,
    }
}

pub fn resolve_panel(key: KeyEvent) -> Command {
    if key.kind == crossterm::event::KeyEventKind::Release {
        return Command::Noop;
    }
    match key.code {
        KeyCode::Esc => Command::SearchClose,
        KeyCode::Enter => Command::PanelEnter,
        KeyCode::Up => Command::PanelUp,
        KeyCode::Down => Command::PanelDown,
        KeyCode::Backspace => Command::PanelBackspace,
        KeyCode::Delete => Command::PanelDelete,
        KeyCode::Char(c) => Command::PanelChar(c),
        _ => Command::Noop,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::KeyEvent;

    fn key(code: KeyCode, ctrl: bool, alt: bool, shift: bool) -> KeyEvent {
        let mut mods = KeyModifiers::empty();
        if ctrl { mods.insert(KeyModifiers::CONTROL); }
        if alt { mods.insert(KeyModifiers::ALT); }
        if shift { mods.insert(KeyModifiers::SHIFT); }
        KeyEvent::new(code, mods)
    }

    #[test]
    fn ctrl_s_maps_to_save() {
        assert_eq!(resolve(key(KeyCode::Char('s'), true, false, false)), Command::Save);
    }

    #[test]
    fn ctrl_shift_s_maps_to_save_as() {
        assert_eq!(resolve(key(KeyCode::Char('s'), true, false, true)), Command::SaveAs);
    }

    #[test]
    fn ctrl_f_maps_to_search() {
        assert_eq!(resolve(key(KeyCode::Char('f'), true, false, false)), Command::Search);
    }

    #[test]
    fn ctrl_h_maps_to_replace() {
        assert_eq!(resolve(key(KeyCode::Char('h'), true, false, false)), Command::ReplaceOpen);
    }

    #[test]
    fn f2_maps_to_notes() {
        assert_eq!(resolve(key(KeyCode::F(2), false, false, false)), Command::ToggleNotes);
    }

    #[test]
    fn f3_maps_to_spellcheck() {
        assert_eq!(resolve(key(KeyCode::F(3), false, false, false)), Command::ToggleSpellcheck);
    }

    #[test]
    fn f4_maps_to_ideas() {
        assert_eq!(resolve(key(KeyCode::F(4), false, false, false)), Command::ToggleIdeas);
    }

    #[test]
    fn f5_maps_to_project() {
        assert_eq!(resolve(key(KeyCode::F(5), false, false, false)), Command::ToggleProject);
    }

    #[test]
    fn f6_maps_to_search() {
        assert_eq!(resolve(key(KeyCode::F(6), false, false, false)), Command::Search);
    }

    #[test]
    fn ctrl_g_maps_to_go_to_line() {
        assert_eq!(resolve(key(KeyCode::Char('g'), true, false, false)), Command::GoToLine);
    }

    #[test]
    fn shift_left_maps_to_select_left() {
        assert_eq!(resolve(key(KeyCode::Left, false, false, true)), Command::SelectLeft);
    }

    #[test]
    fn ctrl_home_maps_to_doc_start() {
        assert_eq!(resolve(key(KeyCode::Home, true, false, false)), Command::MoveDocStart);
    }

    #[test]
    fn ctrl_end_maps_to_doc_end() {
        assert_eq!(resolve(key(KeyCode::End, true, false, false)), Command::MoveDocEnd);
    }

    #[test]
    fn ctrl_up_maps_to_para_up() {
        assert_eq!(resolve(key(KeyCode::Up, true, false, false)), Command::MoveParaUp);
    }

    #[test]
    fn ctrl_down_maps_to_para_down() {
        assert_eq!(resolve(key(KeyCode::Down, true, false, false)), Command::MoveParaDown);
    }

    #[test]
    fn alt_e_opens_menu() {
        assert_eq!(resolve(key(KeyCode::Char('e'), false, true, false)), Command::MenuAlt('e'));
    }

    #[test]
    fn alt_r_replaces() {
        assert_eq!(resolve(key(KeyCode::Char('r'), false, true, false)), Command::ReplaceAccept);
    }

    #[test]
    fn ctrl_alt_r_replace_all() {
        assert_eq!(resolve(key(KeyCode::Char('r'), true, true, false)), Command::ReplaceAll);
    }
}
