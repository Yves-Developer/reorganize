use std::fs;
use std::path::Path;

use crate::organizer::classifier::classify_file;
use crate::organizer::duplicate::resolve_duplicate;

pub fn relocate_file(currpath: &Path, rootpath: &Path) {
    let filename = match currpath.file_name() {
        Some(filename) => filename,
        None => return,
    };

    let extension = currpath
        .extension()
        .map(|extension| extension.to_string_lossy().to_lowercase());

    let category = classify_file(extension.as_deref());
    let newpath = rootpath.join(category);

    let mut destinationpath = newpath.join(filename);

    let filestem = match currpath.file_stem() {
        Some(filestem) => filestem.to_string_lossy(),
        None => return,
    };

    resolve_duplicate(
        &mut destinationpath,
        &filestem,
        extension.as_deref(),
        &newpath,
    );
    match fs::create_dir_all(&newpath) {
        Ok(_) => {}
        Err(error) => {
            println!("Failed to create folder: {}", error);
            return;
        }
    };

    match fs::rename(currpath, destinationpath) {
        Ok(_) => {}
        Err(error) => {
            println!("Failed to move file: {}", error);
            return;
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicUsize, Ordering};

    static COUNTER: AtomicUsize = AtomicUsize::new(0);

    /// A fresh, empty directory unique to this test run.
    fn temp_dir() -> PathBuf {
        let unique = COUNTER.fetch_add(1, Ordering::Relaxed);

        let dir = std::env::temp_dir().join(format!(
            "reorganize-reloc-{}-{}",
            std::process::id(),
            unique
        ));

        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();

        dir
    }

    fn write(path: PathBuf, contents: &str) -> PathBuf {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }

        fs::write(&path, contents).unwrap();

        path
    }

    fn read(path: PathBuf) -> String {
        fs::read_to_string(path).unwrap()
    }

    #[test]
    fn moves_file_into_its_category_folder() {
        let root = temp_dir();
        let file = write(root.join("holiday.jpg"), "photo");

        relocate_file(&file, &root);

        assert!(!file.exists(), "source file should have been moved");
        assert_eq!(read(root.join("Images/holiday.jpg")), "photo");

        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn uppercase_extension_still_classifies() {
        let root = temp_dir();
        let file = write(root.join("SCAN.PDF"), "doc");

        relocate_file(&file, &root);

        assert_eq!(read(root.join("Documents/SCAN.PDF")), "doc");

        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn renames_instead_of_overwriting_an_existing_file() {
        let root = temp_dir();
        write(root.join("Documents/notes.txt"), "original");
        let incoming = write(root.join("notes.txt"), "incoming");

        relocate_file(&incoming, &root);

        assert_eq!(read(root.join("Documents/notes.txt")), "original");
        assert_eq!(read(root.join("Documents/notes (1).txt")), "incoming");

        fs::remove_dir_all(&root).unwrap();
    }

    // Regression: a colliding extensionless file used to be renamed straight
    // over the existing one, destroying it without any warning.
    #[test]
    fn does_not_destroy_an_existing_extensionless_file() {
        let root = temp_dir();
        write(root.join("Other/LICENSE"), "original");
        let incoming = write(root.join("LICENSE"), "incoming");

        relocate_file(&incoming, &root);

        assert_eq!(
            read(root.join("Other/LICENSE")),
            "original",
            "the pre-existing file must survive"
        );
        assert_eq!(read(root.join("Other/LICENSE (1)")), "incoming");

        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn dotfile_is_treated_as_extensionless() {
        let root = temp_dir();
        write(root.join("Other/.gitignore"), "original");
        let incoming = write(root.join(".gitignore"), "incoming");

        relocate_file(&incoming, &root);

        assert_eq!(read(root.join("Other/.gitignore")), "original");
        assert_eq!(read(root.join("Other/.gitignore (1)")), "incoming");

        fs::remove_dir_all(&root).unwrap();
    }
}
