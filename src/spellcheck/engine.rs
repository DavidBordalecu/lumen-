use crate::spellcheck::personal::PersonalDictionary;
use crate::spellcheck::DictInfo;

// ── Error ortográfico ──

#[derive(Debug, Clone)]
pub struct SpellError {
    pub word: String,
    pub line: usize,
    pub col: usize,
    pub byte_offset: usize,
    pub byte_len: usize,
}

// ── Motor de ortografía ──

pub struct SpellcheckEngine {
    dict: Option<spellbook::Dictionary>,
    pub personal: PersonalDictionary,
    ignored: std::collections::HashSet<String>,
    pub errors: Vec<SpellError>,
    pub current_lang: String,
    pub dict_label: String,
    pub available_langs: Vec<DictInfo>,
}

impl std::fmt::Debug for SpellcheckEngine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SpellcheckEngine")
            .field("has_dict", &self.dict.is_some())
            .field("personal_len", &self.personal.len())
            .field("ignored_len", &self.ignored.len())
            .field("errors_len", &self.errors.len())
            .field("current_lang", &self.current_lang)
            .field("dict_label", &self.dict_label)
            .finish()
    }
}

impl SpellcheckEngine {
    pub fn new(info: &DictInfo) -> Option<Self> {
        let aff = std::fs::read_to_string(&info.aff_path).ok()?;
        let dic = std::fs::read_to_string(&info.dic_path).ok()?;
        let dict = spellbook::Dictionary::new(&aff, &dic).ok()?;
        let personal = PersonalDictionary::new();
        Some(Self {
            dict: Some(dict),
            personal,
            ignored: std::collections::HashSet::new(),
            errors: Vec::new(),
            current_lang: info.language.clone(),
            dict_label: info.label.clone(),
            available_langs: Vec::new(),
        })
    }

    pub fn empty() -> Self {
        Self {
            dict: None,
            personal: PersonalDictionary::new(),
            ignored: std::collections::HashSet::new(),
            errors: Vec::new(),
            current_lang: String::new(),
            dict_label: String::new(),
            available_langs: Vec::new(),
        }
    }

    pub fn has_dictionary(&self) -> bool {
        self.dict.is_some()
    }

    pub fn switch_dictionary(&mut self, info: &DictInfo) -> bool {
        let aff = match std::fs::read_to_string(&info.aff_path) {
            Ok(s) => s,
            Err(_) => return false,
        };
        let dic = match std::fs::read_to_string(&info.dic_path) {
            Ok(s) => s,
            Err(_) => return false,
        };
        match spellbook::Dictionary::new(&aff, &dic) {
            Ok(dict) => {
                self.dict = Some(dict);
                self.current_lang = info.language.clone();
                self.dict_label = info.label.clone();
                true
            }
            Err(_) => false,
        }
    }

    pub fn check_word(&self, word: &str) -> bool {
        if word.is_empty() || SpellcheckEngine::is_special_word(word) {
            return true;
        }
        if self.personal.contains(word) || self.ignored.contains(word) {
            return true;
        }
        if let Some(ref dict) = self.dict {
            dict.check(word)
        } else {
            true
        }
    }

    pub fn suggest(&self, word: &str, out: &mut Vec<String>) {
        if let Some(ref dict) = self.dict {
            dict.suggest(word, out);
        }
    }

    pub fn add_to_personal(&mut self, word: &str) {
        self.personal.add(word);
    }

    pub fn ignore_word(&mut self, word: &str) {
        self.ignored.insert(word.to_string());
    }

    pub fn is_special_word(word: &str) -> bool {
        if word.is_empty() {
            return true;
        }
        let chars: Vec<char> = word.chars().collect();
        if chars.len() == 1 {
            let c = chars[0];
            return c.is_ascii_digit() || c == '-' || c == '_';
        }
        if chars.iter().all(|c| c.is_ascii_digit()) {
            return true;
        }
        if chars.iter().all(|c| c.is_uppercase() || *c == '.' || *c == '-') && chars.len() > 1 {
            return true;
        }
        if word.contains('@') || word.contains("://") {
            return true;
        }
        false
    }

    pub fn check_document(&mut self, text: &str) {
        self.errors.clear();
        if self.dict.is_none() {
            return;
        }
        let mut byte_offset = 0usize;
        for (line_idx, line) in text.lines().enumerate() {
            for word in tokenize_line(line) {
                let word_str = &line[word.start..word.end];
                if !self.check_word(word_str) {
                    self.errors.push(SpellError {
                        word: word_str.to_string(),
                        line: line_idx,
                        col: word.start,
                        byte_offset: byte_offset + word.start,
                        byte_len: word.end - word.start,
                    });
                }
            }
            byte_offset += line.len() + 1;
        }
    }

    pub fn replace_word_at(&self, text: &mut String, error: &SpellError, new_word: &str) -> bool {
        if error.byte_offset + error.byte_len > text.len() {
            return false;
        }
        let before = text[..error.byte_offset].to_string();
        let after = text[error.byte_offset + error.byte_len..].to_string();
        *text = format!("{before}{new_word}{after}");
        true
    }

    pub fn next_error(&self, from_line: usize) -> Option<usize> {
        self.errors.iter().position(|e| e.line >= from_line)
    }

    pub fn prev_error(&self, from_line: usize) -> Option<usize> {
        self.errors.iter().rposition(|e| e.line <= from_line)
    }
}

// ── Tokenizer ──

struct Token {
    start: usize,
    end: usize,
}

fn tokenize_line(line: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let bytes = line.as_bytes();
    let len = bytes.len();
    let mut i = 0;
    while i < len {
        if bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_' || bytes[i] > 127 {
            let start = i;
            while i < len && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_' || bytes[i] > 127) {
                i += 1;
            }
            let word = &line[start..i];
            let cleaned = word.trim_matches(|c: char| c == '\'' || c == '\u{2019}' || c == '\u{2018}');
            if !cleaned.is_empty() {
                let clean_start = start + (word.len() - word.trim_start_matches(|c: char| c == '\'' || c == '\u{2019}' || c == '\u{2018}').len());
                let clean_end = clean_start + cleaned.len();
                tokens.push(Token { start: clean_start, end: clean_end });
            }
        } else {
            i += 1;
        }
    }
    tokens
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn special_words_are_valid() {
        assert!(SpellcheckEngine::is_special_word("42"));
        assert!(SpellcheckEngine::is_special_word("NASA"));
        assert!(SpellcheckEngine::is_special_word("user@email.com"));
        assert!(SpellcheckEngine::is_special_word("https://example.com"));
        assert!(!SpellcheckEngine::is_special_word("hello"));
    }

    #[test]
    fn tokenize_finds_words() {
        let tokens = tokenize_line("Hello world 123 test");
        assert_eq!(tokens.len(), 4);
    }

    #[test]
    fn empty_engine_has_no_errors() {
        let mut engine = SpellcheckEngine::empty();
        engine.check_document("hello world");
        assert!(engine.errors.is_empty());
    }

    #[test]
    fn next_and_prev_error() {
        let mut engine = SpellcheckEngine::empty();
        engine.errors = vec![
            SpellError { word: "a".into(), line: 0, col: 0, byte_offset: 0, byte_len: 1 },
            SpellError { word: "b".into(), line: 5, col: 0, byte_offset: 10, byte_len: 1 },
        ];
        assert_eq!(engine.next_error(0), Some(0));
        assert_eq!(engine.next_error(3), Some(1));
        assert_eq!(engine.prev_error(5), Some(1));
        assert_eq!(engine.prev_error(2), Some(0));
    }
}
