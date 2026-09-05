// Gathering just enough about a file to describe it to a model.
//
// Two hard limits: only the first few kilobytes are ever read, and only text
// is quoted. Binary files are identified by their leading bytes instead, which
// also catches files whose extension does not match their contents.

use std::fs;
use std::io::Read;
use std::path::Path;

/// How much of a file is read. Enough to characterise a document, small
/// enough to keep prompts fast and bounded.
pub const EXCERPT_BYTES: usize = 4096;

/// How much text is kept after collapsing whitespace.
pub const EXCERPT_CHARS: usize = 1200;

#[derive(Debug, PartialEq, Eq)]
pub enum Content {
    /// Readable text, already trimmed and whitespace-collapsed.
    Text(String),
    /// Not text. `kind` is set when the leading bytes identify a known format.
    Binary { kind: Option<&'static str> },
    /// The file could not be read at all.
    Unreadable(String),
}

#[derive(Debug)]
pub struct FileFacts {
    pub name: String,
    pub extension: Option<String>,
    pub size: u64,
    pub content: Content,
}

/// Formats identified by their leading bytes.
fn sniff(bytes: &[u8]) -> Option<&'static str> {
    // Written as byte values rather than escapes so the table stays readable
    // and unambiguous.
    const SIGNATURES: [(&[u8], &str); 14] = [
        (b"%PDF-", "PDF document"),
        (&[0x50, 0x4B, 0x03, 0x04], "ZIP archive (also .docx, .xlsx, .pptx, .apk)"),
        (b"Rar!", "RAR archive"),
        (&[0x37, 0x7A, 0xBC, 0xAF], "7-Zip archive"),
        (&[0x1F, 0x8B], "gzip archive"),
        (&[0xFF, 0xD8, 0xFF], "JPEG image"),
        (&[0x89, 0x50, 0x4E, 0x47], "PNG image"),
        (b"GIF8", "GIF image"),
        (b"RIFF", "RIFF media (WAV or AVI)"),
        (b"OggS", "Ogg media"),
        (b"ID3", "MP3 audio"),
        (b"fLaC", "FLAC audio"),
        (b"MZ", "Windows executable"),
        (&[0x7F, 0x45, 0x4C, 0x46], "ELF executable"),
    ];

    SIGNATURES
        .iter()
        .find(|(signature, _)| bytes.starts_with(signature))
        .map(|(_, name)| *name)
}

/// Collapses runs of whitespace and caps the length, so one minified line
/// cannot dominate a prompt.
fn condense(text: &str) -> String {
    let mut condensed = String::new();
    let mut last_was_space = false;

    for character in text.chars() {
        if condensed.chars().count() >= EXCERPT_CHARS {
            break;
        }

        if character.is_whitespace() {
            if !last_was_space && !condensed.is_empty() {
                condensed.push(' ');
            }

            last_was_space = true;
        } else if character.is_control() {
            // Skip: control characters carry no meaning for the model and
            // can garble a terminal when the excerpt is shown in a preview.
            continue;
        } else {
            condensed.push(character);
            last_was_space = false;
        }
    }

    condensed.trim().to_string()
}

pub fn inspect(path: &Path) -> FileFacts {
    let name = path
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_default();

    let extension = path
        .extension()
        .map(|extension| extension.to_string_lossy().to_lowercase());

    let size = fs::metadata(path).map(|meta| meta.len()).unwrap_or(0);

    FileFacts {
        name,
        extension,
        size,
        content: read_content(path),
    }
}

fn read_content(path: &Path) -> Content {
    let mut file = match fs::File::open(path) {
        Ok(file) => file,
        Err(error) => return Content::Unreadable(error.to_string()),
    };

    let mut buffer = vec![0u8; EXCERPT_BYTES];

    let read = match file.read(&mut buffer) {
        Ok(read) => read,
        Err(error) => return Content::Unreadable(error.to_string()),
    };

    buffer.truncate(read);

    if buffer.is_empty() {
        return Content::Text(String::new());
    }

    if let Some(kind) = sniff(&buffer) {
        return Content::Binary { kind: Some(kind) };
    }

    // A NUL byte is the cheapest reliable signal that this is not text.
    if buffer.contains(&0) {
        return Content::Binary { kind: None };
    }

    let text = String::from_utf8_lossy(&buffer);

    // A high share of replacement characters means the bytes were not UTF-8,
    // so treat it as binary rather than quoting mojibake at the model.
    let replacements = text.chars().filter(|character| *character == char::REPLACEMENT_CHARACTER).count();

    if replacements * 10 > text.chars().count() {
        return Content::Binary { kind: None };
    }

    Content::Text(condense(&text))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    static COUNTER: AtomicUsize = AtomicUsize::new(0);

    fn temp_dir() -> std::path::PathBuf {
        let unique = COUNTER.fetch_add(1, Ordering::Relaxed);

        let dir = std::env::temp_dir().join(format!(
            "reorganize-inspect-{}-{}",
            std::process::id(),
            unique
        ));

        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();

        dir
    }

    fn write(dir: &Path, name: &str, bytes: &[u8]) -> std::path::PathBuf {
        let path = dir.join(name);
        fs::write(&path, bytes).unwrap();

        path
    }

    #[test]
    fn reads_text_content() {
        let dir = temp_dir();
        let file = write(&dir, "notes.txt", b"Invoice 4823 for Acme Ltd, due 2024-03-01");

        let facts = inspect(&file);

        assert_eq!(facts.name, "notes.txt");
        assert_eq!(facts.extension.as_deref(), Some("txt"));
        assert_eq!(
            facts.content,
            Content::Text("Invoice 4823 for Acme Ltd, due 2024-03-01".to_string())
        );

        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn collapses_whitespace_in_excerpts() {
        let dir = temp_dir();
        let file = write(&dir, "a.txt", b"line one

	line   two  ");

        let facts = inspect(&file);

        assert_eq!(facts.content, Content::Text("line one line two".to_string()));

        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn caps_the_excerpt_length() {
        let dir = temp_dir();
        let file = write(&dir, "big.txt", "abcdefghij".repeat(5000).as_bytes());

        let facts = inspect(&file);

        match facts.content {
            Content::Text(text) => assert!(
                text.chars().count() <= EXCERPT_CHARS,
                "excerpt was {} chars",
                text.chars().count()
            ),
            other => panic!("expected text, got {other:?}"),
        }

        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn only_reads_the_first_chunk_of_a_large_file() {
        let dir = temp_dir();
        let marker = "X".repeat(EXCERPT_BYTES * 4);
        let file = write(&dir, "huge.txt", format!("{marker}NEEDLE").as_bytes());

        let facts = inspect(&file);

        match facts.content {
            Content::Text(text) => assert!(!text.contains("NEEDLE"), "read past the limit"),
            other => panic!("expected text, got {other:?}"),
        }

        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn identifies_binary_formats_by_their_leading_bytes() {
        let dir = temp_dir();

        let pdf = write(&dir, "scan.pdf", b"%PDF-1.7 trailing bytes");
        assert_eq!(
            inspect(&pdf).content,
            Content::Binary { kind: Some("PDF document") }
        );

        let png = write(&dir, "shot.png", &[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A]);
        assert_eq!(
            inspect(&png).content,
            Content::Binary { kind: Some("PNG image") }
        );

        fs::remove_dir_all(&dir).unwrap();
    }

    // A file whose extension lies still gets described accurately.
    #[test]
    fn a_misnamed_file_is_identified_by_content() {
        let dir = temp_dir();
        let file = write(&dir, "invoice.txt", b"%PDF-1.4 not really text");

        let facts = inspect(&file);

        assert_eq!(facts.extension.as_deref(), Some("txt"));
        assert_eq!(
            facts.content,
            Content::Binary { kind: Some("PDF document") }
        );

        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn unrecognised_binary_is_still_binary() {
        let dir = temp_dir();
        let file = write(&dir, "blob.bin", &[0x01, 0x00, 0x02, 0x00, 0x03]);

        assert_eq!(inspect(&file).content, Content::Binary { kind: None });

        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn an_empty_file_is_empty_text() {
        let dir = temp_dir();
        let file = write(&dir, "empty.txt", b"");

        assert_eq!(inspect(&file).content, Content::Text(String::new()));

        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn a_missing_file_is_unreadable() {
        let dir = temp_dir();

        assert!(matches!(
            inspect(&dir.join("nope.txt")).content,
            Content::Unreadable(_)
        ));

        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn records_the_file_size() {
        let dir = temp_dir();
        let file = write(&dir, "sized.txt", b"12345");

        assert_eq!(inspect(&file).size, 5);

        fs::remove_dir_all(&dir).unwrap();
    }
}
