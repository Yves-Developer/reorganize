// Turning model-proposed text into names that are safe to put on disk.
//
// Once a model can invent folder and file names, its output reaches the
// filesystem directly. Everything here is about making that survivable:
// no traversal, no reserved device names, no characters Windows rejects,
// and nothing that silently resolves somewhere other than where it reads.

// Wired up by the AI organizer; until then only the tests exercise this.
#![allow(dead_code)]

/// Characters Windows forbids in a path component.
const ILLEGAL: [char; 9] = ['<', '>', ':', '"', '/', '\\', '|', '?', '*'];

/// Names Windows treats as devices, with or without an extension.
const RESERVED: [&str; 22] = [
    "con", "prn", "aux", "nul", "com1", "com2", "com3", "com4", "com5", "com6", "com7", "com8",
    "com9", "lpt1", "lpt2", "lpt3", "lpt4", "lpt5", "lpt6", "lpt7", "lpt8", "lpt9",
];

/// Folder names are kept short so the tree stays readable.
const MAX_FOLDER: usize = 48;

/// Leaves room for a ` (12)` duplicate suffix and an extension inside the
/// 255-character limit on a path component.
const MAX_STEM: usize = 120;

/// Cleans one path component, or returns `None` if nothing usable survives.
fn sanitize(input: &str, max: usize) -> Option<String> {
    // A separator means the model tried to propose a path, not a name.
    // Rejecting outright is safer than flattening it into something that
    // looks similar but points elsewhere.
    if input.contains('/') || input.contains('\\') {
        return None;
    }

    let cleaned: String = input
        .trim()
        .chars()
        .filter_map(|character| {
            if character.is_whitespace() {
                // Covers tabs and newlines, which are control characters too:
                // dropping them outright would run two words together.
                Some(' ')
            } else if character.is_control() {
                // Illegal in names, and can hide what a name really says
                // when it is printed to a terminal.
                None
            } else if ILLEGAL.contains(&character) {
                Some(' ')
            } else {
                Some(character)
            }
        })
        .collect();

    // Collapse the runs of whitespace that replacing illegal characters
    // tends to leave behind.
    let mut collapsed = String::with_capacity(cleaned.len());
    let mut last_was_space = false;

    for character in cleaned.chars() {
        let is_space = character.is_whitespace();

        if is_space && last_was_space {
            continue;
        }

        collapsed.push(if is_space { ' ' } else { character });
        last_was_space = is_space;
    }

    // Windows silently strips trailing dots and spaces, so a name ending in
    // one would not be the name that was actually created.
    let trimmed = collapsed.trim().trim_end_matches(['.', ' ']).trim();

    if trimmed.is_empty() {
        return None;
    }

    // "." and ".." are traversal, not names.
    if trimmed.chars().all(|character| character == '.') {
        return None;
    }

    let stem_for_reserved = trimmed
        .split('.')
        .next()
        .unwrap_or(trimmed)
        .to_ascii_lowercase();

    if RESERVED.contains(&stem_for_reserved.as_str()) {
        return None;
    }

    // Truncate on a character boundary, then re-trim in case the cut landed
    // next to a space or dot.
    let truncated: String = trimmed.chars().take(max).collect();
    let truncated = truncated.trim().trim_end_matches(['.', ' ']).trim();

    if truncated.is_empty() {
        return None;
    }

    Some(truncated.to_string())
}

/// A folder name proposed by a model.
pub fn sanitize_folder_name(input: &str) -> Option<String> {
    sanitize(input, MAX_FOLDER)
}

/// The stem of a file name proposed by a model.
///
/// Extensions are never taken from a model: changing one breaks the file's
/// association and can disguise what the file actually is. Callers keep the
/// original extension and rename only the stem.
pub fn sanitize_file_stem(input: &str) -> Option<String> {
    sanitize(input, MAX_STEM)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_reasonable_names() {
        assert_eq!(sanitize_folder_name("Invoices").as_deref(), Some("Invoices"));
        assert_eq!(sanitize_folder_name("Tax 2024").as_deref(), Some("Tax 2024"));
        assert_eq!(sanitize_folder_name("3D Models").as_deref(), Some("3D Models"));
        assert_eq!(sanitize_folder_name("  Photos  ").as_deref(), Some("Photos"));
    }

    #[test]
    fn keeps_non_ascii_names() {
        assert_eq!(sanitize_folder_name("Documentos").as_deref(), Some("Documentos"));
        assert_eq!(sanitize_folder_name("写真").as_deref(), Some("写真"));
    }

    // The important one: a proposed name must never climb out of the folder
    // being organized.
    #[test]
    fn rejects_traversal() {
        assert_eq!(sanitize_folder_name(".."), None);
        assert_eq!(sanitize_folder_name("."), None);
        assert_eq!(sanitize_folder_name("..."), None);
        assert_eq!(sanitize_folder_name("../../Windows"), None);
        assert_eq!(sanitize_folder_name("..\\..\\Windows"), None);
        assert_eq!(sanitize_folder_name("Work/Invoices"), None);
        assert_eq!(sanitize_folder_name("Work\\Invoices"), None);
    }

    #[test]
    fn rejects_absolute_and_drive_paths() {
        assert_eq!(sanitize_folder_name("C:\\Windows\\System32"), None);
        assert_eq!(sanitize_folder_name("/etc/passwd"), None);
        assert_eq!(sanitize_folder_name("\\\\server\\share"), None);
    }

    // Creating one of these can hang or fail in ways that look like a bug.
    #[test]
    fn rejects_reserved_device_names() {
        for name in ["CON", "con", "PRN", "aux", "NUL", "com1", "LPT9"] {
            assert_eq!(sanitize_folder_name(name), None, "{name} should be rejected");
        }
    }

    #[test]
    fn rejects_reserved_names_even_with_an_extension() {
        assert_eq!(sanitize_file_stem("con.txt"), None);
        assert_eq!(sanitize_file_stem("NUL.pdf"), None);
    }

    #[test]
    fn replaces_characters_windows_forbids() {
        assert_eq!(
            sanitize_folder_name("Invoices: 2024?").as_deref(),
            Some("Invoices 2024")
        );
        assert_eq!(sanitize_folder_name("a<b>c|d*e").as_deref(), Some("a b c d e"));
    }

    #[test]
    fn strips_control_characters() {
        assert_eq!(sanitize_folder_name("Inv\u{0007}oices").as_deref(), Some("Invoices"));
        assert_eq!(sanitize_folder_name("a\nb").as_deref(), Some("a b"));
        assert_eq!(sanitize_folder_name("\u{0000}").as_deref(), None);
    }

    // Windows drops these silently, so the folder created would not match the
    // name that was previewed.
    #[test]
    fn strips_trailing_dots_and_spaces() {
        assert_eq!(sanitize_folder_name("Invoices.").as_deref(), Some("Invoices"));
        assert_eq!(sanitize_folder_name("Invoices...").as_deref(), Some("Invoices"));
        assert_eq!(sanitize_folder_name("Invoices ").as_deref(), Some("Invoices"));
    }

    #[test]
    fn rejects_names_that_sanitize_to_nothing() {
        assert_eq!(sanitize_folder_name(""), None);
        assert_eq!(sanitize_folder_name("   "), None);
        assert_eq!(sanitize_folder_name("???"), None);
        assert_eq!(sanitize_folder_name(":::"), None);
    }

    #[test]
    fn truncates_long_names_on_a_character_boundary() {
        let long = "a".repeat(500);

        let folder = sanitize_folder_name(&long).unwrap();
        assert_eq!(folder.chars().count(), MAX_FOLDER);

        let stem = sanitize_file_stem(&long).unwrap();
        assert_eq!(stem.chars().count(), MAX_STEM);
    }

    #[test]
    fn truncating_multibyte_text_does_not_panic() {
        let long = "写".repeat(500);

        let folder = sanitize_folder_name(&long).unwrap();

        assert_eq!(folder.chars().count(), MAX_FOLDER);
    }

    #[test]
    fn a_model_sentence_still_yields_a_usable_name() {
        assert_eq!(
            sanitize_folder_name("Financial Documents / Invoices"),
            None,
            "a separator is a path, not a name"
        );
        assert_eq!(
            sanitize_folder_name("Financial Documents - Invoices").as_deref(),
            Some("Financial Documents - Invoices")
        );
    }
}
