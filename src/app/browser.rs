use std::path::{Path, PathBuf};

/// Navegador de archivos para el diálogo "Abrir archivo".
///
/// Muestra los directorios primero (orden alfabético) y después los archivos.
/// Permite filtrar por nombre mientras se escribe y navegar con teclado.
#[derive(Debug, Clone)]
pub struct FileBrowser {
    dir: PathBuf,
    entries: Vec<PathBuf>,
    shown: Vec<usize>,
    selected: usize,
    scroll: usize,
    filter: String,
    error: Option<String>,
}

impl FileBrowser {
    pub fn open(start: PathBuf) -> Self {
        let mut browser = Self {
            dir: start,
            entries: Vec::new(),
            shown: Vec::new(),
            selected: 0,
            scroll: 0,
            filter: String::new(),
            error: None,
        };
        browser.load();
        browser
    }

    pub fn dir(&self) -> &Path {
        &self.dir
    }

    pub fn filter(&self) -> &str {
        &self.filter
    }

    pub fn error(&self) -> Option<&str> {
        self.error.as_deref()
    }

    pub fn shown_len(&self) -> usize {
        self.shown.len()
    }

    pub fn selected(&self) -> usize {
        self.selected
    }

    pub fn scroll(&self) -> usize {
        self.scroll
    }

    pub fn has_filter(&self) -> bool {
        !self.filter.is_empty()
    }

    /// Entrada seleccionada de la lista filtrada (`shown[shown_idx]`).
    pub fn entry(&self, shown_idx: usize) -> Option<&PathBuf> {
        self.shown.get(shown_idx).map(|&i| &self.entries[i])
    }

    fn load(&mut self) {
        self.entries.clear();
        self.error = None;
        match std::fs::read_dir(&self.dir) {
            Ok(read) => {
                for entry in read.flatten() {
                    self.entries.push(entry.path());
                }
            }
            Err(e) => {
                self.error = Some(format!(
                    "No se pudo leer \"{}\": {e}",
                    self.dir.display()
                ));
            }
        }
        self.entries.sort_by(|a, b| {
            b.is_dir()
                .cmp(&a.is_dir())
                .then_with(|| name(a).cmp(&name(b)))
        });
        self.filter.clear();
        self.refresh_filter();
    }

    fn refresh_filter(&mut self) {
        self.shown.clear();
        let f = self.filter.to_lowercase();
        for (i, p) in self.entries.iter().enumerate() {
            if f.is_empty() || name(p).to_lowercase().contains(&f) {
                self.shown.push(i);
            }
        }
        self.selected = 0;
        self.scroll = 0;
    }

    pub fn push_filter_char(&mut self, c: char) {
        self.filter.push(c);
        self.refresh_filter();
    }

    pub fn pop_filter_char(&mut self) {
        self.filter.pop();
        self.refresh_filter();
    }

    /// Activa la entrada seleccionada. Si es un directorio, entra en él y
    /// devuelve `None`; si es un archivo, devuelve su ruta para abrirla.
    pub fn activate(&mut self) -> Option<PathBuf> {
        let path = self.entry(self.selected)?.clone();
        if path.is_dir() {
            self.dir = path;
            self.load();
            None
        } else {
            Some(path)
        }
    }

    pub fn go_up(&mut self) {
        if let Some(parent) = self.dir.parent() {
            self.dir = parent.to_path_buf();
            self.load();
        }
    }

    pub fn move_up(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
        }
    }

    pub fn move_down(&mut self) {
        if self.selected + 1 < self.shown.len() {
            self.selected += 1;
        }
    }

    pub fn move_home(&mut self) {
        self.selected = 0;
    }

    pub fn move_end(&mut self) {
        self.selected = self.shown.len().saturating_sub(1);
    }

    pub fn page_up(&mut self, page: usize) {
        self.selected = self.selected.saturating_sub(page.max(1));
    }

    pub fn page_down(&mut self, page: usize) {
        let max = self.shown.len().saturating_sub(1);
        self.selected = (self.selected + page.max(1)).min(max);
    }

    /// Mantiene la selección dentro de la ventana visible de la lista.
    pub fn ensure_visible(&mut self, height: usize) {
        if height == 0 {
            return;
        }
        if self.selected < self.scroll {
            self.scroll = self.selected;
        }
        if self.selected >= self.scroll + height {
            self.scroll = self.selected + 1 - height;
        }
        let max_scroll = self.shown.len().saturating_sub(height);
        self.scroll = self.scroll.min(max_scroll);
    }
}

fn name(p: &Path) -> String {
    p.file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_dir(label: &str) -> PathBuf {
        std::env::temp_dir().join(format!("lumen_browser_{}_{}", label, std::process::id()))
    }

    #[test]
    fn lists_directories_first_sorted() {
        let dir = temp_dir("sort");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("zeta")).unwrap();
        std::fs::create_dir_all(dir.join("alpha")).unwrap();
        std::fs::write(dir.join("bbb.txt"), "x").unwrap();
        std::fs::write(dir.join("aaa.txt"), "x").unwrap();

        let b = FileBrowser::open(dir.clone());
        assert_eq!(b.entry(0).unwrap().file_name().unwrap(), "alpha");
        assert_eq!(b.entry(1).unwrap().file_name().unwrap(), "zeta");
        assert_eq!(b.entry(2).unwrap().file_name().unwrap(), "aaa.txt");
        assert_eq!(b.entry(3).unwrap().file_name().unwrap(), "bbb.txt");

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn filter_narrows_list() {
        let dir = temp_dir("filter");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("cuento.txt"), "x").unwrap();
        std::fs::write(dir.join("notas.md"), "x").unwrap();

        let mut b = FileBrowser::open(dir.clone());
        b.push_filter_char('c');
        assert_eq!(b.shown_len(), 1);
        assert_eq!(b.entry(0).unwrap().file_name().unwrap(), "cuento.txt");

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn activate_directory_enters_it() {
        let dir = temp_dir("activate");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("sub")).unwrap();
        std::fs::write(dir.join("sub").join("x.txt"), "x").unwrap();

        let mut b = FileBrowser::open(dir.clone());
        assert!(b.activate().is_none(), "sub es directorio");
        assert_eq!(b.dir().file_name().unwrap(), "sub");
        assert_eq!(b.shown_len(), 1);
        assert_eq!(b.entry(0).unwrap().file_name().unwrap(), "x.txt");

        let _ = std::fs::remove_dir_all(&dir);
    }
}
