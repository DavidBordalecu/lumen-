pub mod undo;

use ropey::Rope;
use unicode_width::UnicodeWidthChar;

use crate::document::Document;
use undo::UndoManager;

#[derive(Debug)]
pub struct Editor {
    cursor: usize,
    anchor: Option<usize>,
    target_col: Option<usize>,
    undo: UndoManager,
}

impl Default for Editor {
    fn default() -> Self {
        Self::new()
    }
}

impl Editor {
    pub fn new() -> Self {
        Self {
            cursor: 0,
            anchor: None,
            target_col: None,
            undo: UndoManager::new(),
        }
    }

    pub fn cursor(&self) -> usize {
        self.cursor
    }

    pub fn set_cursor(&mut self, pos: usize) {
        self.cursor = pos;
        self.anchor = None;
        self.target_col = None;
    }

    /// Rango seleccionado `(inicio, fin)` si existe una selección real.
    pub fn selection(&self) -> Option<(usize, usize)> {
        let (a, b) = self.selection_range();
        if a != b {
            Some((a, b))
        } else {
            None
        }
    }

    pub fn selection_range(&self) -> (usize, usize) {
        match self.anchor {
            Some(a) if a != self.cursor => (a.min(self.cursor), a.max(self.cursor)),
            _ => (self.cursor, self.cursor),
        }
    }

    // ---------------------------------------------------------------- movimientos

    pub fn move_left(&mut self, _rope: &Rope, shift: bool) {
        if shift {
            if self.anchor.is_none() {
                self.anchor = Some(self.cursor);
            }
        } else if let Some((s, _)) = self.selection() {
            self.anchor = None;
            self.target_col = None;
            self.cursor = s;
            return;
        } else {
            self.anchor = None;
            self.target_col = None;
        }
        if self.cursor > 0 {
            self.cursor -= 1;
        }
    }

    pub fn move_right(&mut self, rope: &Rope, shift: bool) {
        if shift {
            if self.anchor.is_none() {
                self.anchor = Some(self.cursor);
            }
        } else if let Some((_, e)) = self.selection() {
            self.anchor = None;
            self.target_col = None;
            self.cursor = e;
            return;
        } else {
            self.anchor = None;
            self.target_col = None;
        }
        if self.cursor < rope.len_chars() {
            self.cursor += 1;
        }
    }

    pub fn move_up(&mut self, rope: &Rope, shift: bool) {
        self.move_vert(rope, -1, shift);
    }

    pub fn move_down(&mut self, rope: &Rope, shift: bool) {
        self.move_vert(rope, 1, shift);
    }

    pub fn page_up(&mut self, rope: &Rope, shift: bool, page: usize) {
        self.move_vert(rope, -(page.max(1) as isize), shift);
    }

    pub fn page_down(&mut self, rope: &Rope, shift: bool, page: usize) {
        self.move_vert(rope, page.max(1) as isize, shift);
    }

    fn move_vert(&mut self, rope: &Rope, delta: isize, shift: bool) {
        if shift {
            if self.anchor.is_none() {
                self.anchor = Some(self.cursor);
            }
        } else {
            self.anchor = None;
        }

        let lines = rope.len_lines();
        let line = rope.char_to_line(self.cursor);
        let line_start = rope.line_to_char(line);
        let col = self.cursor - line_start;
        let target = self.target_col.unwrap_or(col);

        let new_line = if delta >= 0 {
            (line + delta as usize).min(lines - 1)
        } else {
            line.saturating_sub(delta.unsigned_abs())
        };
        let (start, end) = line_bounds(rope, new_line);
        let new_col = target.min(end.saturating_sub(start));
        self.cursor = start + new_col;
        self.target_col = Some(target);
    }

    pub fn move_home(&mut self, rope: &Rope, shift: bool) {
        if shift {
            if self.anchor.is_none() {
                self.anchor = Some(self.cursor);
            }
        } else {
            self.anchor = None;
            self.target_col = None;
        }
        let line = rope.char_to_line(self.cursor);
        self.cursor = rope.line_to_char(line);
    }

    pub fn move_end(&mut self, rope: &Rope, shift: bool) {
        if shift {
            if self.anchor.is_none() {
                self.anchor = Some(self.cursor);
            }
        } else {
            self.anchor = None;
            self.target_col = None;
        }
        let line = rope.char_to_line(self.cursor);
        let (start, end) = line_bounds(rope, line);
        self.cursor = end.max(start);
    }

    pub fn move_word_left(&mut self, rope: &Rope, shift: bool) {
        if shift {
            if self.anchor.is_none() {
                self.anchor = Some(self.cursor);
            }
        } else if let Some((s, _)) = self.selection() {
            self.anchor = None;
            self.target_col = None;
            self.cursor = s;
            return;
        } else {
            self.anchor = None;
            self.target_col = None;
        }
        if self.cursor == 0 {
            return;
        }
        let text = rope.to_string();
        let byte = rope.char_to_byte(self.cursor);
        let chars: Vec<char> = text.chars().collect();
        let char_idx = self.cursor;
        let mut i = char_idx - 1;
        while i > 0 && chars[i].is_whitespace() {
            i -= 1;
        }
        while i > 0 && !chars[i - 1].is_whitespace() {
            i -= 1;
        }
        self.cursor = i;
        let _ = byte;
    }

    pub fn move_word_right(&mut self, rope: &Rope, shift: bool) {
        if shift {
            if self.anchor.is_none() {
                self.anchor = Some(self.cursor);
            }
        } else if let Some((_, e)) = self.selection() {
            self.anchor = None;
            self.target_col = None;
            self.cursor = e;
            return;
        } else {
            self.anchor = None;
            self.target_col = None;
        }
        let len = rope.len_chars();
        if self.cursor >= len {
            return;
        }
        let chars: Vec<char> = rope.chars().collect();
        let mut i = self.cursor;
        let total = chars.len();
        while i < total && !chars[i].is_whitespace() {
            i += 1;
        }
        while i < total && chars[i].is_whitespace() {
            i += 1;
        }
        self.cursor = i.min(len);
    }

    pub fn move_doc_start(&mut self, _rope: &Rope, shift: bool) {
        if shift {
            if self.anchor.is_none() {
                self.anchor = Some(self.cursor);
            }
        } else {
            self.anchor = None;
        }
        self.cursor = 0;
        self.target_col = None;
    }

    pub fn move_doc_end(&mut self, rope: &Rope, shift: bool) {
        if shift {
            if self.anchor.is_none() {
                self.anchor = Some(self.cursor);
            }
        } else {
            self.anchor = None;
        }
        self.cursor = rope.len_chars();
        self.target_col = None;
    }

    pub fn move_para_up(&mut self, rope: &Rope, shift: bool) {
        if shift {
            if self.anchor.is_none() {
                self.anchor = Some(self.cursor);
            }
        } else {
            self.anchor = None;
        }
        let line = rope.char_to_line(self.cursor);
        let mut target_line = line;
        if target_line > 0 {
            target_line -= 1;
            while target_line > 0 && !rope.line(target_line).chars().all(|c| c.is_whitespace()) {
                target_line -= 1;
            }
            while target_line < line && rope.line(target_line).chars().all(|c| c.is_whitespace()) {
                target_line += 1;
            }
        }
        let start = rope.line_to_char(target_line);
        self.cursor = start;
        self.target_col = None;
    }

    pub fn move_para_down(&mut self, rope: &Rope, shift: bool) {
        if shift {
            if self.anchor.is_none() {
                self.anchor = Some(self.cursor);
            }
        } else {
            self.anchor = None;
        }
        let line = rope.char_to_line(self.cursor);
        let total = rope.len_lines();
        let mut target_line = line;
        if target_line + 1 < total {
            target_line += 1;
            while target_line < total && rope.line(target_line).chars().all(|c| c.is_whitespace()) {
                target_line += 1;
            }
            while target_line + 1 < total && !rope.line(target_line).chars().all(|c| c.is_whitespace()) {
                target_line += 1;
            }
        }
        let start = rope.line_to_char(target_line);
        self.cursor = start.min(rope.len_chars());
        self.target_col = None;
    }

    pub fn select_all(&mut self, rope: &Rope) {
        if rope.len_chars() == 0 {
            return;
        }
        self.anchor = Some(0);
        self.cursor = rope.len_chars();
        self.target_col = None;
    }

    pub fn go_to_line(&mut self, rope: &Rope, line_num: usize) {
        let total = rope.len_lines();
        let line = line_num.saturating_sub(1).min(total.saturating_sub(1));
        self.cursor = rope.line_to_char(line);
        self.anchor = None;
        self.target_col = None;
    }

    // -------------------------------------------------------------------- edición

    pub fn insert(&mut self, doc: &mut Document, text: &str) {
        if text.is_empty() {
            return;
        }
        let (start, end) = self.selection_range();
        let old = doc.rope().slice(start..end).to_string();
        let cursor_after = start + text.chars().count();
        let rope = doc.rope_mut();
        if end > start {
            rope.remove(start..end);
        }
        rope.insert(start, text);
        self.undo.record(start, old, text.to_string(), cursor_after);
        self.cursor = cursor_after;
        self.anchor = None;
        self.target_col = None;
    }

    pub fn backspace(&mut self, doc: &mut Document) {
        let (start, end) = self.selection_range();
        if start != end {
            let old = doc.rope().slice(start..end).to_string();
            doc.rope_mut().remove(start..end);
            self.undo.record(start, old, String::new(), start);
            self.cursor = start;
        } else if self.cursor > 0 {
            let pos = self.cursor;
            let old = doc.rope().char(pos - 1).to_string();
            doc.rope_mut().remove(pos - 1..pos);
            self.undo.record(pos - 1, old, String::new(), pos - 1);
            self.cursor = pos - 1;
        } else {
            return;
        }
        self.anchor = None;
        self.target_col = None;
    }

    pub fn delete(&mut self, doc: &mut Document) {
        let (start, end) = self.selection_range();
        if start != end {
            let old = doc.rope().slice(start..end).to_string();
            doc.rope_mut().remove(start..end);
            self.undo.record(start, old, String::new(), start);
            self.cursor = start;
        } else {
            let len = doc.rope().len_chars();
            if self.cursor >= len {
                return;
            }
            let pos = self.cursor;
            let old = doc.rope().char(pos).to_string();
            doc.rope_mut().remove(pos..pos + 1);
            self.undo.record(pos, old, String::new(), pos);
        }
        self.anchor = None;
        self.target_col = None;
    }

    pub fn undo(&mut self, doc: &mut Document) -> bool {
        if let Some(c) = self.undo.undo(doc.rope_mut()) {
            self.cursor = c;
            self.anchor = None;
            self.target_col = None;
            true
        } else {
            false
        }
    }

    pub fn redo(&mut self, doc: &mut Document) -> bool {
        if let Some(c) = self.undo.redo(doc.rope_mut()) {
            self.cursor = c;
            self.anchor = None;
            self.target_col = None;
            true
        } else {
            false
        }
    }

    /// Impide que el pegado se fusione con la siguiente escritura en el historial.
    pub fn seal(&mut self) {
        self.undo.seal();
    }

    // ------------------------------------------------------------------ portapapeles

    pub fn copy(&self, doc: &Document) -> Option<String> {
        let (s, e) = self.selection_range();
        if s == e {
            return None;
        }
        Some(doc.rope().slice(s..e).to_string())
    }

    pub fn cut(&mut self, doc: &mut Document) -> Option<String> {
        let (s, e) = self.selection_range();
        if s == e {
            return None;
        }
        let old = doc.rope().slice(s..e).to_string();
        doc.rope_mut().remove(s..e);
        self.undo.record(s, old.clone(), String::new(), s);
        self.cursor = s;
        self.anchor = None;
        self.target_col = None;
        Some(old)
    }
}

/// Límites de la línea `idx` en índices de carácter, excluyendo el salto de línea.
fn line_bounds(rope: &Rope, idx: usize) -> (usize, usize) {
    let start = rope.line_to_char(idx);
    let end = if idx + 1 < rope.len_lines() {
        rope.line_to_char(idx + 1) - 1
    } else {
        rope.len_chars()
    };
    (start, end)
}

/// Ancho visual (en celdas de terminal) de un carácter en una columna dada.
pub fn char_cell_width(c: char, tab_width: usize, col: usize) -> usize {
    if c == '\t' {
        let m = tab_width.max(1);
        m - (col % m)
    } else {
        UnicodeWidthChar::width(c).unwrap_or(0)
    }
}

/// Columna visual del cursor tras `char_offset` caracteres de la línea.
pub fn visual_col(line: &str, char_offset: usize, tab_width: usize) -> usize {
    let mut col = 0usize;
    for (i, c) in line.chars().enumerate() {
        if i >= char_offset {
            break;
        }
        col += char_cell_width(c, tab_width, col);
    }
    col
}

#[cfg(test)]
mod tests {
    use super::*;

    fn type_text(ed: &mut Editor, doc: &mut Document, text: &str) {
        ed.insert(doc, text);
    }

    #[test]
    fn insert_moves_cursor_and_records_undo() {
        let mut doc = Document::new();
        let mut ed = Editor::new();
        type_text(&mut ed, &mut doc, "hola");
        assert_eq!(doc.rope().to_string(), "hola");
        assert_eq!(ed.cursor(), 4);
    }

    #[test]
    fn undo_redo_restores_text() {
        let mut doc = Document::new();
        let mut ed = Editor::new();
        type_text(&mut ed, &mut doc, "hola mundo");
        assert!(ed.undo(&mut doc));
        assert_eq!(doc.rope().to_string(), "");
        assert!(ed.redo(&mut doc));
        assert_eq!(doc.rope().to_string(), "hola mundo");
        assert!(ed.undo(&mut doc));
        assert_eq!(doc.rope().to_string(), "");
        assert!(ed.redo(&mut doc));
        assert_eq!(doc.rope().to_string(), "hola mundo");
    }

    #[test]
    fn backspace_removes_char_before_cursor() {
        let mut doc = Document::new();
        let mut ed = Editor::new();
        type_text(&mut ed, &mut doc, "hola");
        ed.backspace(&mut doc);
        assert_eq!(doc.rope().to_string(), "hol");
        assert_eq!(ed.cursor(), 3);
        ed.undo(&mut doc);
        assert_eq!(doc.rope().to_string(), "hola");
    }

    #[test]
    fn delete_removes_char_at_cursor() {
        let mut doc = Document::new();
        let mut ed = Editor::new();
        type_text(&mut ed, &mut doc, "hola");
        ed.set_cursor(1);
        ed.delete(&mut doc);
        assert_eq!(doc.rope().to_string(), "hla");
        assert_eq!(ed.cursor(), 1);
    }

    #[test]
    fn typing_over_selection_replaces_it() {
        let mut doc = Document::new();
        let mut ed = Editor::new();
        type_text(&mut ed, &mut doc, "abcdef");
        ed.set_cursor(1);
        ed.move_right(doc.rope(), true);
        ed.move_right(doc.rope(), true);
        assert_eq!(ed.selection(), Some((1, 3)));
        ed.insert(&mut doc, "XY");
        assert_eq!(doc.rope().to_string(), "aXYdef");
        assert_eq!(ed.cursor(), 3);
        assert!(ed.selection().is_none());
    }

    #[test]
    fn cut_copy_paste() {
        let mut doc = Document::new();
        let mut ed = Editor::new();
        type_text(&mut ed, &mut doc, "abcdef");
        ed.set_cursor(2);
        ed.move_right(doc.rope(), true);
        ed.move_right(doc.rope(), true);
        let copied = ed.copy(&doc).unwrap();
        assert_eq!(copied, "cd");
        let cut = ed.cut(&mut doc).unwrap();
        assert_eq!(cut, "cd");
        assert_eq!(doc.rope().to_string(), "abef");
        ed.move_end(doc.rope(), false);
        ed.insert(&mut doc, &cut);
        assert_eq!(doc.rope().to_string(), "abefcd");
    }

    #[test]
    fn vertical_movement_clamps_and_preserves_column() {
        let mut doc = Document::new();
        let mut ed = Editor::new();
        type_text(&mut ed, &mut doc, "aaa\nbb\nc");
        ed.set_cursor(doc.rope().len_chars());
        ed.move_up(doc.rope(), false);
        assert_eq!(ed.cursor(), 5); // col 1 de "bb"
        ed.move_up(doc.rope(), false);
        assert_eq!(ed.cursor(), 1); // col 1 de "aaa"
        ed.move_up(doc.rope(), false);
        assert_eq!(ed.cursor(), 1); // ya no sube más
        ed.move_down(doc.rope(), false);
        assert_eq!(ed.cursor(), 5);
        ed.move_down(doc.rope(), false);
        assert_eq!(ed.cursor(), doc.rope().len_chars());
    }

    #[test]
    fn home_end_movements() {
        let mut doc = Document::new();
        let mut ed = Editor::new();
        type_text(&mut ed, &mut doc, "primera\nsegunda");
        ed.set_cursor(doc.rope().len_chars());
        ed.move_home(doc.rope(), false);
        assert_eq!(ed.cursor(), 8);
        ed.move_end(doc.rope(), false);
        assert_eq!(ed.cursor(), doc.rope().len_chars());
    }

    #[test]
    fn visual_col_handles_tabs_and_wide_chars() {
        assert_eq!(visual_col("hola", 2, 4), 2);
        assert_eq!(visual_col("", 0, 4), 0);
        assert_eq!(visual_col("\t\t", 2, 4), 8);
        assert_eq!(visual_col("é", 1, 4), 1);
        assert_eq!(visual_col("你", 1, 4), 2);
        assert_eq!(visual_col("ab", 2, 4), 2);
    }

    #[test]
    fn tab_width_advances_to_tab_stop() {
        assert_eq!(char_cell_width('\t', 4, 0), 4);
        assert_eq!(char_cell_width('\t', 4, 3), 1);
        assert_eq!(char_cell_width('\t', 4, 4), 4);
        assert_eq!(char_cell_width('a', 4, 0), 1);
    }
}
