use std::collections::HashSet;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::organizer::classifier::classify_file;
use crate::organizer::duplicate::resolve_duplicate;

/// Works out where files should go, and (outside a dry run) moves them.
///
/// Destinations chosen earlier in the same run are remembered, because during
/// a dry run nothing is written to disk: without this, three loose `LICENSE`
/// files would all be reported as moving to the same path.
#[derive(Default)]
pub struct Relocator {
    claimed: HashSet<PathBuf>,
}

impl Relocator {
    pub fn new() -> Self {
        Relocator::default()
    }

    /// Decides the destination using the built-in extension classifier.
    pub fn plan(&mut self, currpath: &Path, rootpath: &Path) -> io::Result<PathBuf> {
        let category = classify_file(lowercase_extension(currpath).as_deref());

        self.plan_into(currpath, rootpath, category.folder_name(), None)
    }

    /// Decides the destination using a caller-supplied folder and, optionally,
    /// a new file stem. Both must already be sanitized.
    ///
    /// The extension always comes from the original file, so a rename can
    /// never change what a file claims to be.
    pub fn plan_into(
        &mut self,
        currpath: &Path,
        rootpath: &Path,
        folder: &str,
        new_stem: Option<&str>,
    ) -> io::Result<PathBuf> {
        let current_stem = currpath
            .file_stem()
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "path has no file stem"))?
            .to_string_lossy()
            .into_owned();

        let stem = new_stem.unwrap_or(&current_stem);

        // Kept as written on disk: lowercasing it here would quietly rename
        // every `.JPG` to `.jpg`.
        let extension = currpath
            .extension()
            .map(|extension| extension.to_string_lossy().into_owned());

        let destdir = rootpath.join(folder);

        let filename = match extension.as_deref() {
            Some(extension) => format!("{stem}.{extension}"),
            None => stem.to_string(),
        };

        let mut destpath = destdir.join(filename);

        let claimed = &self.claimed;
        resolve_duplicate(&mut destpath, stem, extension.as_deref(), &destdir, |path| {
            path.exists() || claimed.contains(path)
        });

        self.claimed.insert(destpath.clone());

        Ok(destpath)
    }

    /// Plans with the built-in classifier and performs the move.
    // A convenience wrapper: the binary plans everything up front and then
    // calls `perform`, but this keeps the simple path available and tested.
    #[allow(dead_code)]
    pub fn relocate(&mut self, currpath: &Path, rootpath: &Path) -> io::Result<PathBuf> {
        let destpath = self.plan(currpath, rootpath)?;

        self.perform(currpath, destpath)
    }

    /// Moves a file to an already-planned destination.
    pub fn perform(&mut self, currpath: &Path, destpath: PathBuf) -> io::Result<PathBuf> {
        if let Some(destdir) = destpath.parent() {
            fs::create_dir_all(destdir)?;
        }

        fs::rename(currpath, &destpath)?;

        Ok(destpath)
    }
}

fn lowercase_extension(path: &Path) -> Option<String> {
    path.extension()
        .map(|extension| extension.to_string_lossy().to_lowercase())
}

#[cfg(test)]
mod tests {
    use super::*;
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

        Relocator::new().relocate(&file, &root).unwrap();

        assert!(!file.exists(), "source file should have been moved");
        assert_eq!(read(root.join("Images/holiday.jpg")), "photo");

        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn uppercase_extension_still_classifies() {
        let root = temp_dir();
        let file = write(root.join("SCAN.PDF"), "doc");

        Relocator::new().relocate(&file, &root).unwrap();

        assert_eq!(read(root.join("Documents/SCAN.PDF")), "doc");

        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn renames_instead_of_overwriting_an_existing_file() {
        let root = temp_dir();
        write(root.join("Documents/notes.txt"), "original");
        let incoming = write(root.join("notes.txt"), "incoming");

        Relocator::new().relocate(&incoming, &root).unwrap();

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

        Relocator::new().relocate(&incoming, &root).unwrap();

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

        Relocator::new().relocate(&incoming, &root).unwrap();

        assert_eq!(read(root.join("Other/.gitignore")), "original");
        assert_eq!(read(root.join("Other/.gitignore (1)")), "incoming");

        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn returns_an_error_instead_of_printing_when_the_move_fails() {
        let root = temp_dir();
        let missing = root.join("ghost.txt");

        let result = Relocator::new().relocate(&missing, &root);

        assert!(result.is_err(), "expected an error for a missing source file");

        fs::remove_dir_all(&root).unwrap();
    }

    // A dry run writes nothing, so the filesystem cannot tell the planner that
    // an earlier file already claimed a name. Without the claimed set, all
    // three of these would be reported as landing on the same path.
    #[test]
    fn planning_does_not_reuse_a_destination_within_one_run() {
        let root = temp_dir();
        let a = write(root.join("LICENSE"), "a");
        let b = write(root.join("nested/LICENSE"), "b");
        let c = write(root.join("nested/deeper/LICENSE"), "c");

        let mut relocator = Relocator::new();

        let first = relocator.plan(&a, &root).unwrap();
        let second = relocator.plan(&b, &root).unwrap();
        let third = relocator.plan(&c, &root).unwrap();

        assert_eq!(first, root.join("Other/LICENSE"));
        assert_eq!(second, root.join("Other/LICENSE (1)"));
        assert_eq!(third, root.join("Other/LICENSE (2)"));

        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn planning_creates_nothing_on_disk() {
        let root = temp_dir();
        let file = write(root.join("holiday.jpg"), "photo");

        let planned = Relocator::new().plan(&file, &root).unwrap();

        assert_eq!(planned, root.join("Images/holiday.jpg"));
        assert!(!root.join("Images").exists(), "plan must not create folders");
        assert!(file.exists(), "plan must not move the file");

        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn relocate_reports_where_the_file_landed() {
        let root = temp_dir();
        write(root.join("Documents/notes.txt"), "original");
        let incoming = write(root.join("notes.txt"), "incoming");

        let landed = Relocator::new().relocate(&incoming, &root).unwrap();

        assert_eq!(landed, root.join("Documents/notes (1).txt"));

        fs::remove_dir_all(&root).unwrap();
    }
}
