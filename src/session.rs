use std::time::Instant;

#[derive(Debug)]
pub struct Session {
    pub started_at: Instant,
    pub initial_words: usize,
}

impl Session {
    pub fn new(word_count: usize) -> Self {
        Self {
            started_at: Instant::now(),
            initial_words: word_count,
        }
    }

    pub fn words_written(&self, current_words: usize) -> isize {
        current_words as isize - self.initial_words as isize
    }

    pub fn elapsed_display(&self) -> String {
        let secs = self.started_at.elapsed().as_secs();
        let h = secs / 3600;
        let m = (secs % 3600) / 60;
        if h > 0 {
            format!("{}h {}min", h, m)
        } else {
            format!("{}min", m)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn words_written_positive() {
        let s = Session::new(100);
        assert_eq!(s.words_written(150), 50);
    }

    #[test]
    fn words_written_negative() {
        let s = Session::new(200);
        assert_eq!(s.words_written(150), -50);
    }

    #[test]
    fn words_written_zero() {
        let s = Session::new(100);
        assert_eq!(s.words_written(100), 0);
    }
}
