use std::io::{self, Write};
use std::path::{Path, PathBuf};

use ropey::Rope;

pub mod model;
pub mod odt;

use model::{DocumentFormat, DocumentModel};

#[derive(Debug)]
pub struct Document {
    rope: Rope,
    path: Option<PathBuf>,
}

impl Default for Document {
    fn default() -> Self {
        Self::new()
    }
}

impl Document {
    pub fn new() -> Self {
        Self {
            rope: Rope::new(),
            path: None,
        }
    }

    /// Abre un archivo detectando automáticamente el formato.
    /// Soporta TXT, ODT y otros formatos soportados.
    pub fn open(path: &Path) -> io::Result<Self> {
        let format = path.extension()
            .and_then(|e| e.to_str())
            .and_then(DocumentFormat::from_extension)
            .unwrap_or(DocumentFormat::Text);
        
        let rope = match format {
            DocumentFormat::Text => {
                let bytes = std::fs::read(path)?;
                let text = String::from_utf8(bytes).map_err(|_| {
                    io::Error::new(io::ErrorKind::InvalidData, "el archivo no es UTF-8 válido")
                })?;
                Rope::from_str(&text)
            }
            DocumentFormat::Odt => {
                let model = odt::OdtModel;
                model.read(path)?
            }
        };
        
        Ok(Self {
            rope,
            path: Some(path.to_path_buf()),
        })
    }

    /// Crea un documento nuevo ya asociado a una ruta (para archivos que
    /// aún no existen pero que Lumen creará al guardar).
    pub fn create(path: &Path) -> Self {
        Self {
            rope: Rope::new(),
            path: Some(path.to_path_buf()),
        }
    }

    /// Crea un documento con texto ya cargado (p. ej. desde una copia
    /// temporal recuperada).
    pub fn from_text(text: &str, path: Option<PathBuf>) -> Self {
        Self {
            rope: Rope::from_str(text),
            path,
        }
    }

    pub fn path(&self) -> Option<&Path> {
        self.path.as_deref()
    }

    pub fn set_path(&mut self, path: PathBuf) {
        self.path = Some(path);
    }

    pub fn rope(&self) -> &Rope {
        &self.rope
    }

    pub fn rope_mut(&mut self) -> &mut Rope {
        &mut self.rope
    }

    pub fn word_count(&self) -> usize {
        let mut count = 0usize;
        let mut in_word = false;
        for c in self.rope.chars() {
            if c.is_whitespace() {
                in_word = false;
            } else if !in_word {
                count += 1;
                in_word = true;
            }
        }
        count
    }

    /// Escritura segura: detecta el formato por extensión y escribe
    /// usando el model apropiado.
    pub fn save(&self, path: &Path) -> io::Result<()> {
        let format = path.extension()
            .and_then(|e| e.to_str())
            .and_then(DocumentFormat::from_extension)
            .unwrap_or(DocumentFormat::Text);
        
        match format {
            DocumentFormat::Text => {
                let tmp = tmp_path(path);
                {
                    let mut f = std::fs::File::create(&tmp)?;
                    f.write_all(self.rope.to_string().as_bytes())?;
                    f.flush()?;
                    f.sync_all()?;
                }
                if let Err(e) = std::fs::rename(&tmp, path) {
                    let _ = std::fs::remove_file(&tmp);
                    return Err(e);
                }
                if let Some(dir) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
                    if let Ok(d) = std::fs::File::open(dir) {
                        let _ = d.sync_all();
                    }
                }
                Ok(())
            }
            DocumentFormat::Odt => {
                let model = odt::OdtModel;
                model.write(&self.rope, path)
            }
        }
    }
}

fn tmp_path(path: &Path) -> PathBuf {
    let mut os = path.as_os_str().to_os_string();
    os.push(".tmp");
    PathBuf::from(os)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn doc(text: &str) -> Document {
        Document {
            rope: Rope::from_str(text),
            path: None,
        }
    }

    #[test]
    fn word_count_basics() {
        assert_eq!(doc("").word_count(), 0);
        assert_eq!(doc("hola mundo").word_count(), 2);
        assert_eq!(doc("hola\nmundo\n").word_count(), 2);
        assert_eq!(doc("  hola   mundo  ").word_count(), 2);
        assert_eq!(doc("ñandú, año. ¿cómo?").word_count(), 3);
    }

    #[test]
    fn save_then_open_roundtrip() {
        let name = format!("lumen_test_{}.txt", std::process::id());
        let path = std::env::temp_dir().join(name);
        let d = doc("hola ñandú\nsegunda línea con acentos");
        d.save(&path).unwrap();

        let loaded = Document::open(&path).unwrap();
        assert_eq!(loaded.rope().to_string(), "hola ñandú\nsegunda línea con acentos");
        assert!(loaded.path().is_some());

        let tmp = tmp_path(&path);
        assert!(!tmp.exists(), "no debe quedar archivo temporal");
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn open_missing_file_errors_with_not_found() {
        let path = std::path::Path::new("no_existe_este_archivo_lumen_12345.txt");
        let err = Document::open(path).unwrap_err();
        assert_eq!(err.kind(), std::io::ErrorKind::NotFound);
    }
}
