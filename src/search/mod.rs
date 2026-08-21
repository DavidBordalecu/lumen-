use ropey::Rope;

#[derive(Debug, Default)]
pub struct Search {
    pub active: bool,
    pub query: String,
    pub replace_text: String,
    pub match_range: Option<(usize, usize)>,
}

impl Search {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn open(&mut self) {
        self.active = true;
        self.replace_text.clear();
        self.match_range = None;
    }

    pub fn close(&mut self) {
        self.active = false;
        self.match_range = None;
    }

    pub fn push_char(&mut self, c: char) {
        self.query.push(c);
    }

    pub fn pop_char(&mut self) {
        self.query.pop();
    }

    pub fn push_replace_char(&mut self, c: char) {
        self.replace_text.push(c);
    }

    pub fn pop_replace_char(&mut self) {
        self.replace_text.pop();
    }

    pub fn find_next(&self, rope: &Rope, from_char: usize) -> Option<usize> {
        if self.query.is_empty() {
            return None;
        }
        let text = rope.to_string();
        let from_byte = rope
            .char_to_byte(from_char.min(rope.len_chars()))
            .min(text.len());
        let q = self.query.as_str();

        if from_byte < text.len() {
            if let Some(m) = text[from_byte..].find(q) {
                let byte = from_byte + m;
                return Some(text[..byte].chars().count());
            }
        }
        if from_byte > 0 {
            if let Some(m) = text[..from_byte].find(q) {
                return Some(text[..m].chars().count());
            }
        }
        None
    }

    pub fn find_previous(&self, rope: &Rope, from_char: usize) -> Option<usize> {
        if self.query.is_empty() {
            return None;
        }
        let text = rope.to_string();
        let from_byte = rope
            .char_to_byte(from_char.min(rope.len_chars()))
            .min(text.len());
        let q = self.query.as_str();
        let q_byte_len = q.len();

        if from_byte > 0 {
            let search_end = (from_byte + 1).min(text.len());
            if let Some(m) = text[..search_end].rfind(q) {
                return Some(text[..m].chars().count());
            }
        }
        if from_byte < text.len() {
            if let Some(m) = text[from_byte..].rfind(q) {
                let byte = from_byte + m;
                return Some(text[..byte].chars().count());
            }
        }
        let _ = q_byte_len;
        None
    }

    pub fn replace_current(
        &self,
        rope: &mut Rope,
        from_char: usize,
    ) -> Option<usize> {
        if self.query.is_empty() {
            return None;
        }
        let pos = self.find_next(rope, from_char)?;
        let q_len = self.query.chars().count();
        let end = pos + q_len;
        let _old = rope.slice(pos..end).to_string();
        rope.remove(pos..end);
        rope.insert(pos, &self.replace_text);
        Some(pos + self.replace_text.chars().count())
    }

    pub fn replace_all(
        &self,
        rope: &mut Rope,
    ) -> usize {
        if self.query.is_empty() {
            return 0;
        }
        let text = rope.to_string();
        let q = self.query.as_str();
        let r = self.replace_text.as_str();
        let mut count = 0usize;
        let mut result = String::new();
        let mut last = 0;
        for mat in text.match_indices(q) {
            result.push_str(&text[last..mat.0]);
            result.push_str(r);
            last = mat.0 + mat.1.len();
            count += 1;
        }
        result.push_str(&text[last..]);
        if count > 0 {
            *rope = Rope::from_str(&result);
        }
        count
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_next_occurrence() {
        let rope = Rope::from_str("hola mundo hola");
        let s = Search { active: false, query: "hola".into(), replace_text: String::new(), match_range: None };
        assert_eq!(s.find_next(&rope, 0), Some(0));
        assert_eq!(s.find_next(&rope, 1), Some(11));
    }

    #[test]
    fn wraps_to_beginning_when_not_found_ahead() {
        let rope = Rope::from_str("hola mundo hola");
        let s = Search { active: false, query: "hola".into(), replace_text: String::new(), match_range: None };
        assert_eq!(s.find_next(&rope, 12), Some(0));
    }

    #[test]
    fn returns_none_when_absent() {
        let rope = Rope::from_str("hola mundo");
        let s = Search { active: false, query: "xyz".into(), replace_text: String::new(), match_range: None };
        assert_eq!(s.find_next(&rope, 0), None);
    }

    #[test]
    fn empty_query_never_finds() {
        let rope = Rope::from_str("hola");
        let s = Search { active: false, query: String::new(), replace_text: String::new(), match_range: None };
        assert_eq!(s.find_next(&rope, 0), None);
    }

    #[test]
    fn handles_unicode_offsets() {
        let rope = Rope::from_str("ñandú y ñandú");
        let s = Search { active: false, query: "ñandú".into(), replace_text: String::new(), match_range: None };
        assert_eq!(s.find_next(&rope, 0), Some(0));
        assert_eq!(s.find_next(&rope, 1), Some(8));
    }

    #[test]
    fn find_previous_works() {
        let rope = Rope::from_str("hola mundo hola");
        let s = Search { active: false, query: "hola".into(), replace_text: String::new(), match_range: None };
        assert_eq!(s.find_previous(&rope, 14), Some(11));
        assert_eq!(s.find_previous(&rope, 10), Some(0));
    }

    #[test]
    fn find_previous_wraps() {
        let rope = Rope::from_str("hola mundo hola");
        let s = Search { active: false, query: "hola".into(), replace_text: String::new(), match_range: None };
        assert_eq!(s.find_previous(&rope, 0), Some(11));
    }

    #[test]
    fn replace_current_works() {
        let mut rope = Rope::from_str("hola mundo hola");
        let s = Search { active: false, query: "mundo".into(), replace_text: "planeta".into(), match_range: None };
        let new_pos = s.replace_current(&mut rope, 0).unwrap();
        assert_eq!(rope.to_string(), "hola planeta hola");
        assert_eq!(new_pos, 12);
    }

    #[test]
    fn replace_all_works() {
        let mut rope = Rope::from_str("hola mundo hola");
        let s = Search { active: false, query: "hola".into(), replace_text: "adiós".into(), match_range: None };
        let count = s.replace_all(&mut rope);
        assert_eq!(rope.to_string(), "adiós mundo adiós");
        assert_eq!(count, 2);
    }

    #[test]
    fn replace_all_no_match() {
        let mut rope = Rope::from_str("hola mundo");
        let s = Search { active: false, query: "xyz".into(), replace_text: "abc".into(), match_range: None };
        let count = s.replace_all(&mut rope);
        assert_eq!(rope.to_string(), "hola mundo");
        assert_eq!(count, 0);
    }
}
