use std::path::{Path, PathBuf};

/// Advances `destpath` to the first free `name (n)` slot inside `destdir`.
///
/// `extension` is `None` for files that have none (`LICENSE`, `Makefile`),
/// which must still be de-duplicated — otherwise the caller's `fs::rename`
/// would silently overwrite an existing file.
pub fn resolve_duplicate(
    destpath: &mut PathBuf,
    filestem: &str,
    extension: Option<&str>,
    destdir: &Path,
) {
    let mut counter = 0;

    while destpath.exists() {
        counter += 1;

        let candidate = match extension {
            Some(extension) => format!("{filestem} ({counter}).{extension}"),
            None => format!("{filestem} ({counter})"),
        };

        *destpath = destdir.join(candidate);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::sync::atomic::{AtomicUsize, Ordering};

    static COUNTER: AtomicUsize = AtomicUsize::new(0);

    /// A fresh, empty directory unique to this test run.
    fn temp_dir() -> PathBuf {
        let unique = COUNTER.fetch_add(1, Ordering::Relaxed);

        let dir = std::env::temp_dir().join(format!(
            "reorganize-dup-{}-{}",
            std::process::id(),
            unique
        ));

        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();

        dir
    }

    fn touch(dir: &Path, name: &str) {
        fs::write(dir.join(name), b"x").unwrap();
    }

    #[test]
    fn leaves_path_untouched_when_nothing_collides() {
        let dir = temp_dir();
        let mut dest = dir.join("notes.txt");

        resolve_duplicate(&mut dest, "notes", Some("txt"), &dir);

        assert_eq!(dest, dir.join("notes.txt"));

        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn appends_counter_on_first_collision() {
        let dir = temp_dir();
        touch(&dir, "notes.txt");

        let mut dest = dir.join("notes.txt");
        resolve_duplicate(&mut dest, "notes", Some("txt"), &dir);

        assert_eq!(dest, dir.join("notes (1).txt"));

        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn skips_past_several_existing_duplicates() {
        let dir = temp_dir();
        touch(&dir, "notes.txt");
        touch(&dir, "notes (1).txt");
        touch(&dir, "notes (2).txt");

        let mut dest = dir.join("notes.txt");
        resolve_duplicate(&mut dest, "notes", Some("txt"), &dir);

        assert_eq!(dest, dir.join("notes (3).txt"));

        fs::remove_dir_all(&dir).unwrap();
    }

    // Regression: extensionless files used to skip de-duplication entirely,
    // so fs::rename silently overwrote the existing file.
    #[test]
    fn de_duplicates_files_without_an_extension() {
        let dir = temp_dir();
        touch(&dir, "LICENSE");

        let mut dest = dir.join("LICENSE");
        resolve_duplicate(&mut dest, "LICENSE", None, &dir);

        assert_eq!(dest, dir.join("LICENSE (1)"));

        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn skips_past_several_extensionless_duplicates() {
        let dir = temp_dir();
        touch(&dir, "Makefile");
        touch(&dir, "Makefile (1)");

        let mut dest = dir.join("Makefile");
        resolve_duplicate(&mut dest, "Makefile", None, &dir);

        assert_eq!(dest, dir.join("Makefile (2)"));

        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn does_not_append_a_bare_dot_when_extension_is_absent() {
        let dir = temp_dir();
        touch(&dir, "LICENSE");

        let mut dest = dir.join("LICENSE");
        resolve_duplicate(&mut dest, "LICENSE", None, &dir);

        let name = dest.file_name().unwrap().to_string_lossy().into_owned();
        assert!(!name.ends_with('.'), "unexpected trailing dot in {name:?}");

        fs::remove_dir_all(&dir).unwrap();
    }
}
