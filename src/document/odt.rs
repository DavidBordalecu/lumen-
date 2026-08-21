use std::io::{self, Write, BufReader};
use std::path::Path;

use ropey::Rope;

use super::model::{DocumentFormat, DocumentModel};

pub struct OdtModel;

impl DocumentModel for OdtModel {
    fn format(&self) -> DocumentFormat {
        DocumentFormat::Odt
    }

    fn read(&self, path: &Path) -> io::Result<Rope> {
        let file = std::fs::File::open(path)?;
        let mut archive = zip::ZipArchive::new(file)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        
        let content_xml = archive.by_name("content.xml")
            .map_err(|e| io::Error::new(io::ErrorKind::NotFound, e))?;
        
        let text = extract_text_from_odt(content_xml)?;
        Ok(Rope::from_str(&text))
    }

    fn write(&self, rope: &Rope, path: &Path) -> io::Result<()> {
        let file = std::fs::File::create(path)?;
        let mut zip = zip::ZipWriter::new(file);
        
        let options = zip::write::FileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated);
        
        zip.start_file("content.xml", options)?;
        let xml = generate_odt_xml(rope);
        zip.write_all(xml.as_bytes())?;
        
        zip.finish()?;
        Ok(())
    }
}

fn extract_text_from_odt(reader: impl io::Read) -> io::Result<String> {
    let mut paragraphs = Vec::new();
    let mut current_text = String::new();
    let mut in_paragraph = false;
    
    let buf_reader = BufReader::new(reader);
    let mut parser = quick_xml::Reader::from_reader(buf_reader);
    let mut buf = Vec::new();
    
    loop {
        match parser.read_event_into(&mut buf) {
            Ok(quick_xml::events::Event::Start(ref e)) => {
                if e.name().as_ref() == b"text:p" {
                    in_paragraph = true;
                    current_text.clear();
                }
            }
            Ok(quick_xml::events::Event::Text(ref e)) => {
                if in_paragraph {
                    if let Ok(t) = e.unescape() {
                        current_text.push_str(&t);
                    }
                }
            }
            Ok(quick_xml::events::Event::End(ref e)) => {
                if e.name().as_ref() == b"text:p" && in_paragraph {
                    in_paragraph = false;
                    paragraphs.push(current_text.clone());
                }
            }
            Ok(quick_xml::events::Event::Eof) => break,
            Err(e) => return Err(io::Error::new(io::ErrorKind::InvalidData, e)),
            _ => {}
        }
        buf.clear();
    }
    
    Ok(paragraphs.join("\n"))
}

fn generate_odt_xml(rope: &Rope) -> String {
    let mut xml = String::from(r#"<?xml version="1.0" encoding="UTF-8"?>
<office:document-content
  xmlns:office="urn:oasis:names:tc:opendocument:xmlns:office:1.0"
  xmlns:text="urn:oasis:names:tc:opendocument:xmlns:text:1.0"
  office:version="1.2">
  <office:body>
    <office:text>"#);
    
    for line in rope.lines() {
        xml.push_str("    <text:p>");
        let line_str = line.to_string();
        let trimmed = line_str.trim_end_matches('\n');
        xml.push_str(&xml_escape(trimmed));
        xml.push_str("</text:p>\n");
    }
    
    xml.push_str("  </office:text>\n</office:body>\n</office:document-content>");
    xml
}

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
     .replace('<', "&lt;")
     .replace('>', "&gt;")
     .replace('"', "&quot;")
     .replace('\'', "&apos;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn xml_escape_test() {
        assert_eq!(xml_escape("hello"), "hello");
        assert_eq!(xml_escape("a & b"), "a &amp; b");
        assert_eq!(xml_escape("<tag>"), "&lt;tag&gt;");
    }

    #[test]
    fn generate_odt_xml_basic() {
        let rope = Rope::from_str("Hello\nWorld");
        let xml = generate_odt_xml(&rope);
        assert!(xml.contains("<text:p>Hello</text:p>"));
        assert!(xml.contains("<text:p>World</text:p>"));
    }

    #[test]
    fn odt_roundtrip() {
        let path = std::env::temp_dir().join(format!("lumen_test_{}.odt", std::process::id()));
        let rope = Rope::from_str("Hello from Lumen\nSecond line with ñ");
        
        let model = OdtModel;
        model.write(&rope, &path).unwrap();
        let loaded = model.read(&path).unwrap();
        
        assert_eq!(loaded.to_string(), rope.to_string());
        let _ = std::fs::remove_file(&path);
    }
}