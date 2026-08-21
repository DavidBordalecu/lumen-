use ropey::Rope;

#[derive(Debug)]
struct Edit {
    start: usize,
    old: String,
    new: String,
    cursor: usize,
}

#[derive(Debug)]
pub struct UndoManager {
    undo: Vec<Edit>,
    redo: Vec<Edit>,
    pending: Option<Edit>,
}

const MAX_MERGE: usize = 256;
const MAX_STACK: usize = 500;

impl Default for UndoManager {
    fn default() -> Self {
        Self::new()
    }
}

impl UndoManager {
    pub fn new() -> Self {
        Self {
            undo: Vec::new(),
            redo: Vec::new(),
            pending: None,
        }
    }

    /// Registra una edición: la parte `[start, start + old.chars().count())`
    /// fue reemplazada por `new`, y tras la edición el cursor quedó en `cursor`.
    /// Las inserciones contiguas (escribir) y las supresiones contiguas
    /// (Backspace / Delete) se agrupan en una sola entrada de historial.
    pub fn record(&mut self, start: usize, old: String, new: String, cursor: usize) {
        let old_len = old.chars().count();
        let new_len = new.chars().count();

        if let Some(p) = self.pending.as_mut() {
            // Inserciones contiguas (escritura normal).
            if old_len == 0
                && p.old.is_empty()
                && start == p.start + p.new.chars().count()
                && p.new.chars().count() + new_len <= MAX_MERGE
            {
                p.new.push_str(&new);
                p.cursor = cursor;
                return;
            }
            // Supresiones hacia atrás (Backspace).
            if new_len == 0
                && p.new.is_empty()
                && p.start == start + old_len
                && p.old.chars().count() + old_len <= MAX_MERGE
            {
                let mut merged = old;
                merged.push_str(&p.old);
                p.old = merged;
                p.start = start;
                p.cursor = cursor;
                return;
            }
            // Supresiones hacia adelante (Delete).
            if new_len == 0
                && p.new.is_empty()
                && start == p.start
                && p.old.chars().count() + old_len <= MAX_MERGE
            {
                let mut merged = std::mem::take(&mut p.old);
                merged.push_str(&old);
                p.old = merged;
                p.cursor = cursor;
                return;
            }
        }

        self.push_pending();
        self.pending = Some(Edit {
            start,
            old,
            new,
            cursor,
        });
        self.redo.clear();
    }

    /// Fuerza a que la operación pendiente actual pase al historial, de modo
    /// que la siguiente edición no se fusione con ella. Se usa tras un pegado.
    pub fn seal(&mut self) {
        self.push_pending();
    }

    fn push_pending(&mut self) {
        if let Some(edit) = self.pending.take() {
            if edit.old.is_empty() && edit.new.is_empty() {
                return;
            }
            self.undo.push(edit);
            while self.undo.len() > MAX_STACK {
                self.undo.remove(0);
            }
        }
    }

    pub fn undo(&mut self, rope: &mut Rope) -> Option<usize> {
        self.push_pending();
        let edit = self.undo.pop()?;
        rope.remove(edit.start..edit.start + edit.new.chars().count());
        rope.insert(edit.start, &edit.old);
        let cursor = edit.start + edit.old.chars().count();
        self.redo.push(edit);
        Some(cursor)
    }

    pub fn redo(&mut self, rope: &mut Rope) -> Option<usize> {
        let edit = self.redo.pop()?;
        rope.remove(edit.start..edit.start + edit.old.chars().count());
        rope.insert(edit.start, &edit.new);
        let cursor = edit.cursor;
        self.undo.push(edit);
        Some(cursor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn contiguous_inserts_merge_into_one_undo_step() {
        let mut rope = Rope::from_str("");
        let mut u = UndoManager::new();
        rope.insert(0, "hola");
        u.record(0, String::new(), "hola".into(), 4);
        rope.insert(4, " mundo");
        u.record(4, String::new(), " mundo".into(), 10);
        assert_eq!(rope.to_string(), "hola mundo");

        assert_eq!(u.undo(&mut rope), Some(0));
        assert_eq!(rope.to_string(), "");
        assert!(u.undo(&mut rope).is_none(), "debe ser una sola entrada");
    }

    #[test]
    fn contiguous_backspaces_merge_into_one_undo_step() {
        let mut rope = Rope::from_str("abc");
        let mut u = UndoManager::new();
        rope.remove(2..3);
        u.record(2, "c".into(), String::new(), 2);
        rope.remove(1..2);
        u.record(1, "b".into(), String::new(), 1);

        assert_eq!(u.undo(&mut rope), Some(3));
        assert_eq!(rope.to_string(), "abc");
        assert!(u.undo(&mut rope).is_none());
    }

    #[test]
    fn paste_is_sealed_from_next_typing() {
        let mut rope = Rope::from_str("");
        let mut u = UndoManager::new();
        rope.insert(0, "abc");
        u.record(0, String::new(), "abc".into(), 3);
        u.seal();
        rope.insert(3, "d");
        u.record(3, String::new(), "d".into(), 4);

        assert_eq!(u.undo(&mut rope), Some(3));
        assert_eq!(rope.to_string(), "abc");
        assert_eq!(u.undo(&mut rope), Some(0));
        assert_eq!(rope.to_string(), "");
    }

    #[test]
    fn redo_reapplies_after_undo() {
        let mut rope = Rope::from_str("");
        let mut u = UndoManager::new();
        rope.insert(0, "xy");
        u.record(0, String::new(), "xy".into(), 2);

        u.undo(&mut rope);
        assert_eq!(u.redo(&mut rope), Some(2));
        assert_eq!(rope.to_string(), "xy");
    }
}
