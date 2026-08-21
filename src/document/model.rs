use std::io;
use std::path::Path;

use ropey::Rope;

/// Supported document formats
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DocumentFormat {
    Text,
    Odt,
}

impl DocumentFormat {
    /// Detect format from file extension
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_lowercase().as_str() {
            "txt" | "text" | "md" | "markdown" => Some(DocumentFormat::Text),
            "odt" => Some(DocumentFormat::Odt),
            _ => None,
        }
    }

    /// Get default extension for format
    pub fn extension(&self) -> &'static str {
        match self {
            DocumentFormat::Text => "txt",
            DocumentFormat::Odt => "odt",
        }
    }

    /// Get human-readable format name
    pub fn name(&self) -> &'static str {
        match self {
            DocumentFormat::Text => "Plain Text",
            DocumentFormat::Odt => "OpenDocument Text",
        }
    }
}

/// Trait for document format handlers
pub trait DocumentModel: Send + Sync {
    /// Get the format this model handles
    fn format(&self) -> DocumentFormat;

    /// Read a document from disk into a Rope
    fn read(&self, path: &Path) -> io::Result<Rope>;

    /// Write a Rope to disk in this format
    fn write(&self, rope: &Rope, path: &Path) -> io::Result<()>;

    /// Check if this model can handle the given file
    fn can_handle(&self, path: &Path) -> bool {
        path.extension()
            .and_then(|e| e.to_str())
            .and_then(DocumentFormat::from_extension)
            .map(|f| f == self.format())
            .unwrap_or(false)
    }
}

/// Plain text document model
pub struct TextModel;

impl DocumentModel for TextModel {
    fn format(&self) -> DocumentFormat {
        DocumentFormat::Text
    }

    fn read(&self, path: &Path) -> io::Result<Rope> {
        let bytes = std::fs::read(path)?;
        let text = String::from_utf8(bytes).map_err(|_| {
            io::Error::new(io::ErrorKind::InvalidData, "file is not valid UTF-8")
        })?;
        Ok(Rope::from_str(&text))
    }

    fn write(&self, rope: &Rope, path: &Path) -> io::Result<()> {
        use std::io::Write;
        
        let tmp = super::tmp_path(path);
        {
            let mut f = std::fs::File::create(&tmp)?;
            f.write_all(rope.to_string().as_bytes())?;
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
}

/// Get the appropriate model for a file path
pub fn model_for_path(path: &Path) -> Option<Box<dyn DocumentModel>> {
    let format = path.extension()
        .and_then(|e| e.to_str())
        .and_then(DocumentFormat::from_extension)?;
    
    match format {
        DocumentFormat::Text => Some(Box::new(TextModel)),
        DocumentFormat::Odt => Some(Box::new(super::odt::OdtModel)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_detection() {
        assert_eq!(DocumentFormat::from_extension("txt"), Some(DocumentFormat::Text));
        assert_eq!(DocumentFormat::from_extension("text"), Some(DocumentFormat::Text));
        assert_eq!(DocumentFormat::from_extension("md"), Some(DocumentFormat::Text));
        assert_eq!(DocumentFormat::from_extension("odt"), Some(DocumentFormat::Odt));
        assert_eq!(DocumentFormat::from_extension("pdf"), None);
        assert_eq!(DocumentFormat::from_extension("TXT"), Some(DocumentFormat::Text));
        assert_eq!(DocumentFormat::from_extension("ODT"), Some(DocumentFormat::Odt));
    }

    #[test]
    fn format_extensions() {
        assert_eq!(DocumentFormat::Text.extension(), "txt");
        assert_eq!(DocumentFormat::Odt.extension(), "odt");
    }

    #[test]
    fn format_names() {
        assert_eq!(DocumentFormat::Text.name(), "Plain Text");
        assert_eq!(DocumentFormat::Odt.name(), "OpenDocument Text");
    }
}