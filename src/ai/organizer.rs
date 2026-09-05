// Asking a local model where a file should go and what it should be called.
//
// Nothing the model returns is trusted: folder names and file stems both go
// through the sanitizer, extensions are always kept from the original file,
// and any failure falls back to the deterministic extension classifier.

use serde::Deserialize;

use crate::ai::ollama::{AiError, Ollama};
use crate::inspect::{Content, FileFacts};
use crate::naming::{sanitize_file_stem, sanitize_folder_name};
use crate::organizer::category::Category;

#[derive(Debug, PartialEq, Eq)]
pub struct Proposal {
    /// Folder name, already sanitized.
    pub folder: String,
    /// New stem, sanitized, or `None` to keep the current name.
    pub new_stem: Option<String>,
    pub reason: String,
    /// True when this came from the extension classifier, not the model.
    pub fell_back: bool,
}

#[derive(Debug, Deserialize)]
struct RawProposal {
    #[serde(default)]
    folder: String,
    #[serde(default)]
    file_name: String,
    #[serde(default)]
    reason: String,
}

fn schema() -> serde_json::Value {
    serde_json::json!({
        "type": "object",
        "properties": {
            "folder": { "type": "string" },
            "file_name": { "type": "string" },
            "reason": { "type": "string" }
        },
        "required": ["folder", "file_name", "reason"]
    })
}

/// What the model is shown about a file. Only an excerpt, never the whole file.
pub fn describe(facts: &FileFacts) -> String {
    let mut description = format!("File name: {}
", facts.name);

    description.push_str(&format!("Size: {} bytes
", facts.size));

    match &facts.content {
        Content::Text(text) if text.is_empty() => {
            description.push_str("Contents: (empty file)
");
        }
        Content::Text(text) => {
            description.push_str(&format!("Beginning of contents: {text}
"));
        }
        Content::Binary { kind: Some(kind) } => {
            description.push_str(&format!("Contents: binary, detected as {kind}
"));
        }
        Content::Binary { kind: None } => {
            description.push_str("Contents: binary, format not recognised
");
        }
        Content::Unreadable(error) => {
            description.push_str(&format!("Contents: could not be read ({error})
"));
        }
    }

    description
}

fn prompt_for(facts: &FileFacts) -> String {
    let existing: Vec<&str> = Category::ALL
        .iter()
        .filter(|category| **category != Category::Other)
        .map(|category| category.folder_name())
        .collect();

    format!(
        "You are filing one file into a folder.

         {description}
         Choose a folder. Prefer one of these when it fits: {existing}.
         Otherwise propose a short, specific folder name (1-3 words).

         Also choose a file name. Keep the original name unless it is unclear          or meaningless, in which case propose a short descriptive one.          Do not include a file extension. Do not include any path or slashes.

         Give a one-sentence reason.",
        description = describe(facts),
        existing = existing.join(", "),
    )
}

/// The stem of the current file name, used to detect a no-op rename.
fn current_stem(name: &str) -> &str {
    match name.rfind('.') {
        // A leading dot is part of the name (".gitignore"), not an extension.
        Some(index) if index > 0 => &name[..index],
        _ => name,
    }
}

/// Falls back to the deterministic classifier, with no rename.
fn fallback(facts: &FileFacts, reason: String) -> Proposal {
    let category = crate::organizer::classifier::classify_file(facts.extension.as_deref());

    Proposal {
        folder: category.folder_name().to_string(),
        new_stem: None,
        reason,
        fell_back: true,
    }
}

/// Turns a model reply into a proposal, rejecting anything unusable.
pub fn interpret(facts: &FileFacts, raw: &str) -> Proposal {
    let parsed: RawProposal = match serde_json::from_str(raw) {
        Ok(parsed) => parsed,
        Err(error) => return fallback(facts, format!("model reply was not usable ({error})")),
    };

    let Some(folder) = sanitize_folder_name(&parsed.folder) else {
        return fallback(
            facts,
            format!("model proposed an unusable folder name {:?}", parsed.folder),
        );
    };

    // The model is told not to include an extension, but it often does anyway.
    let proposed_stem = current_stem(parsed.file_name.trim());

    let new_stem = sanitize_file_stem(proposed_stem).filter(|stem| {
        // Only a real change counts as a rename.
        stem != current_stem(&facts.name)
    });

    Proposal {
        folder,
        new_stem,
        reason: parsed.reason.trim().to_string(),
        fell_back: false,
    }
}

/// Asks the model about one file. Never fails: an unreachable server or an
/// unusable answer becomes a fallback proposal.
pub fn propose(ollama: &Ollama, facts: &FileFacts) -> Proposal {
    match ollama.generate_json(&prompt_for(facts), schema()) {
        Ok(raw) => interpret(facts, &raw),
        Err(AiError::Unavailable(detail)) => fallback(facts, format!("model unavailable ({detail})")),
        Err(AiError::Malformed(detail)) => fallback(facts, format!("model reply malformed ({detail})")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn facts(name: &str) -> FileFacts {
        FileFacts {
            name: name.to_string(),
            extension: name.rsplit_once('.').map(|(_, ext)| ext.to_lowercase()),
            size: 100,
            content: Content::Text("some text".to_string()),
        }
    }

    fn reply(folder: &str, file_name: &str) -> String {
        serde_json::json!({
            "folder": folder,
            "file_name": file_name,
            "reason": "because"
        })
        .to_string()
    }

    #[test]
    fn accepts_a_sensible_reply() {
        let proposal = interpret(&facts("scan001.pdf"), &reply("Invoices", "Acme invoice March"));

        assert_eq!(proposal.folder, "Invoices");
        assert_eq!(proposal.new_stem.as_deref(), Some("Acme invoice March"));
        assert!(!proposal.fell_back);
    }

    // The reason the sanitizer exists.
    #[test]
    fn a_traversal_folder_falls_back() {
        let proposal = interpret(&facts("a.txt"), &reply("../../Windows", "a"));

        assert!(proposal.fell_back);
        assert_eq!(proposal.folder, "Documents");
        assert_eq!(proposal.new_stem, None);
    }

    #[test]
    fn a_reserved_folder_name_falls_back() {
        let proposal = interpret(&facts("a.txt"), &reply("CON", "a"));

        assert!(proposal.fell_back);
        assert_eq!(proposal.folder, "Documents");
    }

    #[test]
    fn a_nested_folder_path_falls_back() {
        let proposal = interpret(&facts("a.txt"), &reply("Work/Invoices", "a"));

        assert!(proposal.fell_back);
    }

    #[test]
    fn an_unparseable_reply_falls_back_to_the_extension() {
        let proposal = interpret(&facts("holiday.jpg"), "not json at all");

        assert!(proposal.fell_back);
        assert_eq!(proposal.folder, "Images");
        assert_eq!(proposal.new_stem, None);
    }

    #[test]
    fn a_file_with_no_extension_falls_back_to_other() {
        let proposal = interpret(&facts("LICENSE"), "not json");

        assert!(proposal.fell_back);
        assert_eq!(proposal.folder, "Other");
    }

    // The model is told not to send an extension and routinely does anyway.
    #[test]
    fn an_extension_in_the_proposed_name_is_dropped() {
        let proposal = interpret(&facts("scan.pdf"), &reply("Invoices", "Acme invoice.pdf"));

        assert_eq!(proposal.new_stem.as_deref(), Some("Acme invoice"));
    }

    #[test]
    fn proposing_the_current_name_is_not_a_rename() {
        let proposal = interpret(&facts("holiday.jpg"), &reply("Images", "holiday"));

        assert_eq!(proposal.new_stem, None);
        assert_eq!(proposal.folder, "Images");
    }

    #[test]
    fn proposing_the_current_name_with_its_extension_is_not_a_rename() {
        let proposal = interpret(&facts("holiday.jpg"), &reply("Images", "holiday.jpg"));

        assert_eq!(proposal.new_stem, None);
    }

    #[test]
    fn an_unusable_file_name_leaves_the_name_alone() {
        // The folder is fine, so this is not a fallback: only the rename is dropped.
        let proposal = interpret(&facts("a.txt"), &reply("Notes", "???"));

        assert!(!proposal.fell_back);
        assert_eq!(proposal.folder, "Notes");
        assert_eq!(proposal.new_stem, None);
    }

    #[test]
    fn a_file_name_containing_a_path_is_not_used() {
        let proposal = interpret(&facts("a.txt"), &reply("Notes", "../../evil"));

        assert_eq!(proposal.new_stem, None);
    }

    #[test]
    fn a_dotfile_keeps_its_whole_name_as_the_stem() {
        assert_eq!(current_stem(".gitignore"), ".gitignore");
        assert_eq!(current_stem("archive.tar.gz"), "archive.tar");
        assert_eq!(current_stem("LICENSE"), "LICENSE");
    }

    #[test]
    fn describing_a_binary_file_does_not_quote_its_bytes() {
        let described = describe(&FileFacts {
            name: "shot.png".to_string(),
            extension: Some("png".to_string()),
            size: 2048,
            content: Content::Binary { kind: Some("PNG image") },
        });

        assert!(described.contains("PNG image"));
        assert!(described.contains("binary"));
    }
}
