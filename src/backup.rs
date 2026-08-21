use std::collections::hash_map::DefaultHasher;
use std::fs;
use std::hash::{Hash, Hasher};
use std::io;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

/// Copia de seguridad automática del documento para recuperar cambios ante
/// un cierre inesperado (caída, corte de energía, etc.).
///
/// Se guarda en el directorio temporal del sistema, con un nombre derivado
/// del hash de la ruta del archivo original. En el arranque, Lumen la
/// compara con el archivo real y ofrece restaurarla si es más reciente.

#[derive(Debug)]
pub struct Backup {
    /// Ruta del archivo original, o `None` si el documento no tenía título.
    pub original: Option<PathBuf>,
    /// Ruta del archivo temporal que contiene el texto.
    pub path: PathBuf,
    /// Momento de la última escritura de la copia.
    pub modified: SystemTime,
}

fn backup_dir() -> PathBuf {
    let user = std::env::var("USER")
        .or_else(|_| std::env::var("USERNAME"))
        .unwrap_or_else(|_| "user".into());
    std::env::temp_dir().join(format!("lumen-{user}"))
}

fn hash_of(path: &Path) -> u64 {
    let mut h = DefaultHasher::new();
    path.to_string_lossy().hash(&mut h);
    h.finish()
}

/// Ruta del archivo temporal correspondiente al documento `original`.
pub fn file_path(original: &Path) -> PathBuf {
    backup_dir().join(format!("{:016x}.bak", hash_of(original)))
}

/// Ruta del archivo temporal para un documento aún sin nombre.
pub fn unsaved_path() -> PathBuf {
    backup_dir().join("sin_titulo.bak")
}

fn write(path: &Path, text: &str) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let part = path.with_extension("part");
    fs::write(&part, text)?;
    if path.exists() {
        fs::remove_file(path)?;
    }
    fs::rename(&part, path)?;
    Ok(())
}

/// Guarda el texto como copia temporal de `original`.
pub fn save(original: &Path, text: &str) -> io::Result<()> {
    write(&file_path(original), text)
}

/// Guarda el texto como copia temporal de un documento sin nombre.
pub fn save_unsaved(text: &str) -> io::Result<()> {
    write(&unsaved_path(), text)
}

fn read(path: &Path, original: Option<PathBuf>) -> Option<Backup> {
    let meta = fs::metadata(path).ok()?;
    Some(Backup {
        original,
        path: path.to_path_buf(),
        modified: meta.modified().ok()?,
    })
}

/// Busca la copia temporal de `original`, si existe.
pub fn find(original: &Path) -> Option<Backup> {
    read(&file_path(original), Some(original.to_path_buf()))
}

/// Busca la copia temporal del documento sin nombre, si existe.
pub fn find_unsaved() -> Option<Backup> {
    read(&unsaved_path(), None)
}

/// Elimina la copia temporal de `original`.
pub fn remove(original: &Path) {
    let _ = fs::remove_file(file_path(original));
}

/// Elimina la copia temporal del documento sin nombre.
pub fn remove_unsaved() {
    let _ = fs::remove_file(unsaved_path());
}

#[cfg(test)]
mod tests {
    use super::*;

    fn orig() -> PathBuf {
        std::env::temp_dir().join(format!("lumen_backup_orig_{}.txt", std::process::id()))
    }

    #[test]
    fn roundtrip_save_find_remove() {
        let o = orig();
        save(&o, "contenido ñandú").unwrap();
        let b = find(&o).unwrap();
        assert_eq!(b.original.as_deref(), Some(o.as_path()));
        assert_eq!(fs::read_to_string(&b.path).unwrap(), "contenido ñandú");
        remove(&o);
        assert!(find(&o).is_none());
    }

    #[test]
    fn unsaved_roundtrip() {
        save_unsaved("sin título").unwrap();
        let b = find_unsaved().unwrap();
        assert!(b.original.is_none());
        assert_eq!(fs::read_to_string(&b.path).unwrap(), "sin título");
        remove_unsaved();
        assert!(find_unsaved().is_none());
    }
}
