// The set of folders reorganize is allowed to create.
//
// This is deliberately a closed enum rather than a free string: it is the
// boundary that untrusted input (an AI-suggested category, a hand-edited
// rules file) has to pass through before it can name a folder on disk.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Category {
    Images,
    Videos,
    Audio,
    Documents,
    Archives,
    Applications,
    Code,
    Fonts,
    DiskImages,
    Shortcuts,
    EBooks,
    AndroidBuilds,
    IosBuilds,
    Other,
}

impl Category {
    /// Every variant, so callers can enumerate the categories without
    /// having to remember to update a second list.
    // Used by `parse` and the tests today; the rules/AI work will consume it
    // directly to build the list of categories offered to a model.
    #[allow(dead_code)]
    pub const ALL: [Category; 14] = [
        Category::Images,
        Category::Videos,
        Category::Audio,
        Category::Documents,
        Category::Archives,
        Category::Applications,
        Category::Code,
        Category::Fonts,
        Category::DiskImages,
        Category::Shortcuts,
        Category::EBooks,
        Category::AndroidBuilds,
        Category::IosBuilds,
        Category::Other,
    ];

    /// The folder name created on disk.
    pub fn folder_name(self) -> &'static str {
        match self {
            Category::Images => "Images",
            Category::Videos => "Videos",
            Category::Audio => "Audio",
            Category::Documents => "Documents",
            Category::Archives => "Archives",
            Category::Applications => "Applications",
            Category::Code => "Code",
            Category::Fonts => "Fonts",
            Category::DiskImages => "Disk Images",
            Category::Shortcuts => "Shortcuts",
            Category::EBooks => "E-books",
            Category::AndroidBuilds => "Android Builds",
            Category::IosBuilds => "iOS Builds",
            Category::Other => "Other",
        }
    }

    /// Parses a category name, returning `None` for anything unrecognised.
    ///
    /// Matching ignores case, spacing and punctuation so that "disk images",
    /// "Disk_Images" and "DiskImages" all resolve. Returning `Option` rather
    /// than defaulting to `Other` keeps "the model said something invalid"
    /// distinguishable from "the model said Other".
    // Nothing in the binary parses category names yet; this is the seam the
    // rules file and AI suggestions will pass through.
    #[allow(dead_code)]
    pub fn parse(input: &str) -> Option<Category> {
        let normalized = normalize(input);

        if normalized.is_empty() {
            return None;
        }

        Category::ALL
            .into_iter()
            .find(|category| normalize(category.folder_name()) == normalized)
    }
}

fn normalize(input: &str) -> String {
    input
        .chars()
        .filter(|character| character.is_ascii_alphanumeric())
        .map(|character| character.to_ascii_lowercase())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_folder_name_parses_back_to_its_variant() {
        for category in Category::ALL {
            assert_eq!(
                Category::parse(category.folder_name()),
                Some(category),
                "{:?} did not round-trip",
                category
            );
        }
    }

    #[test]
    fn all_contains_every_variant_exactly_once() {
        let mut names: Vec<&str> = Category::ALL.iter().map(|c| c.folder_name()).collect();
        let count = names.len();

        names.sort_unstable();
        names.dedup();

        assert_eq!(names.len(), count, "duplicate entry in Category::ALL");
    }

    #[test]
    fn parsing_ignores_case_spacing_and_punctuation() {
        assert_eq!(Category::parse("disk images"), Some(Category::DiskImages));
        assert_eq!(Category::parse("Disk_Images"), Some(Category::DiskImages));
        assert_eq!(Category::parse("DISKIMAGES"), Some(Category::DiskImages));
        assert_eq!(Category::parse("  Disk-Images  "), Some(Category::DiskImages));
        assert_eq!(Category::parse("ebooks"), Some(Category::EBooks));
        assert_eq!(Category::parse("ios builds"), Some(Category::IosBuilds));
    }

    // The validation boundary: anything a model might invent must be rejected
    // rather than silently becoming a folder name.
    #[test]
    fn rejects_unknown_categories() {
        assert_eq!(Category::parse("Spreadsheets"), None);
        assert_eq!(Category::parse("Tax Documents 2024"), None);
        assert_eq!(Category::parse(""), None);
        assert_eq!(Category::parse("   "), None);
        assert_eq!(Category::parse("../../etc"), None);
    }

    #[test]
    fn other_is_a_valid_answer_not_a_parse_failure() {
        assert_eq!(Category::parse("Other"), Some(Category::Other));
    }

    // These strings are folder names on users' disks. Changing one silently
    // strands files in the old folder and starts a new one beside it, so they
    // are pinned here deliberately.
    #[test]
    fn folder_names_are_stable() {
        assert_eq!(Category::Images.folder_name(), "Images");
        assert_eq!(Category::Videos.folder_name(), "Videos");
        assert_eq!(Category::Audio.folder_name(), "Audio");
        assert_eq!(Category::Documents.folder_name(), "Documents");
        assert_eq!(Category::Archives.folder_name(), "Archives");
        assert_eq!(Category::Applications.folder_name(), "Applications");
        assert_eq!(Category::Code.folder_name(), "Code");
        assert_eq!(Category::Fonts.folder_name(), "Fonts");
        assert_eq!(Category::DiskImages.folder_name(), "Disk Images");
        assert_eq!(Category::Shortcuts.folder_name(), "Shortcuts");
        assert_eq!(Category::EBooks.folder_name(), "E-books");
        assert_eq!(Category::AndroidBuilds.folder_name(), "Android Builds");
        assert_eq!(Category::IosBuilds.folder_name(), "iOS Builds");
        assert_eq!(Category::Other.folder_name(), "Other");
    }
}
