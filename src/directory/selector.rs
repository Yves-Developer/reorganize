use std::fs;
use std::path::{Path, PathBuf};

use inquire::{Select, Text};

/// The menu entry that asks the user to type a path instead of picking one.
pub const CUSTOM_PATH: &str = "Custom path";

const ORGANIZE_HERE: &str = "· Organize this folder";
const GO_UP: &str = "· Go up";

pub fn select_directory() -> Result<String, String> {
    let options = vec!["Downloads", "Documents", "Desktop", CUSTOM_PATH];
    let selection = Select::new("What do you want to organize?", options).prompt();

    match selection {
        Ok(directory) => Ok(directory.to_string()),
        Err(_) => Err("Selection cancelled.".to_string()),
    }
}

pub fn prompt_custom_path() -> Result<String, String> {
    let entered = Text::new("Enter the full path of the folder to organize:").prompt();

    match entered {
        Ok(path) => {
            let path = path.trim().trim_matches('"').to_string();

            if path.is_empty() {
                return Err("No path entered.".to_string());
            }

            Ok(path)
        }

        Err(_) => Err("Custom path cancelled.".to_string()),
    }
}

/// Immediate subdirectories, sorted, with hidden ones left out.
///
/// Unreadable entries are skipped rather than failing the listing: one
/// permission-denied folder should not stop you browsing the rest.
fn subdirectories(directory: &Path) -> Vec<PathBuf> {
    let Ok(entries) = fs::read_dir(directory) else {
        return Vec::new();
    };

    let mut found: Vec<PathBuf> = entries
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| path.is_dir())
        .filter(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| !name.starts_with('.'))
        })
        .collect();

    found.sort();

    found
}

fn label_for(path: &Path) -> String {
    path.file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_else(|| path.to_string_lossy().into_owned())
}

/// Walks down from `start` so a nested folder can be picked without typing
/// its path. Returns the folder chosen, or `None` if the user backed out.
pub fn browse_from(start: PathBuf) -> Option<PathBuf> {
    let root = start.clone();
    let mut current = start;

    loop {
        let children = subdirectories(&current);

        // A leaf folder is almost always the one that was meant, so don't
        // make the user confirm a menu with nothing to descend into.
        if children.is_empty() && current != root {
            return Some(current);
        }

        let mut options = vec![ORGANIZE_HERE.to_string()];

        if current != root {
            options.push(GO_UP.to_string());
        }

        options.extend(children.iter().map(|child| label_for(child)));

        let prompt = format!("{}  ({} inside)", label_for(&current), children.len());

        let choice = Select::new(&prompt, options).prompt().ok()?;

        if choice == ORGANIZE_HERE {
            return Some(current);
        }

        if choice == GO_UP {
            current = current.parent().map(Path::to_path_buf).unwrap_or(current);
            continue;
        }

        // Match on the listed paths rather than rebuilding one from the label,
        // so a name that round-trips oddly still resolves to the real folder.
        if let Some(child) = children.iter().find(|child| label_for(child) == choice) {
            current = child.clone();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    static COUNTER: AtomicUsize = AtomicUsize::new(0);

    fn temp_dir() -> PathBuf {
        let unique = COUNTER.fetch_add(1, Ordering::Relaxed);

        let dir = std::env::temp_dir().join(format!(
            "reorganize-select-{}-{}",
            std::process::id(),
            unique
        ));

        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();

        dir
    }

    #[test]
    fn lists_immediate_subdirectories_only() {
        let root = temp_dir();
        fs::create_dir_all(root.join("Invoices/2024")).unwrap();
        fs::create_dir_all(root.join("Photos")).unwrap();
        fs::write(root.join("a.txt"), "x").unwrap();

        let found: Vec<String> = subdirectories(&root).iter().map(|p| label_for(p)).collect();

        assert_eq!(found, vec!["Invoices", "Photos"]);

        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn hides_dot_directories() {
        let root = temp_dir();
        fs::create_dir_all(root.join(".git")).unwrap();
        fs::create_dir_all(root.join("src")).unwrap();

        let found: Vec<String> = subdirectories(&root).iter().map(|p| label_for(p)).collect();

        assert_eq!(found, vec!["src"]);

        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn listing_is_sorted() {
        let root = temp_dir();

        for name in ["zeta", "alpha", "midway"] {
            fs::create_dir_all(root.join(name)).unwrap();
        }

        let found: Vec<String> = subdirectories(&root).iter().map(|p| label_for(p)).collect();

        assert_eq!(found, vec!["alpha", "midway", "zeta"]);

        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn an_unreadable_directory_lists_as_empty_rather_than_failing() {
        let root = temp_dir();

        assert!(subdirectories(&root.join("does-not-exist")).is_empty());

        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn labels_fall_back_to_the_full_path_at_a_root() {
        let root = temp_dir();

        assert_eq!(label_for(&root.join("Invoices")), "Invoices");
        assert_eq!(label_for(Path::new("C:\\")), "C:\\");

        fs::remove_dir_all(&root).unwrap();
    }
}
