// Run logs, so a reorganize run can be reversed.
//
// One tab-separated line per move: `original<TAB>destination`. Windows forbids
// tab (and every other control character) in file names, so no escaping is
// needed and a path can never be split across columns.

use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

/// `%LOCALAPPDATA%\reorganize\runs` — outside any folder being organized, so
/// the log can never be picked up and filed away by a later run.
pub fn log_directory() -> io::Result<PathBuf> {
    let base = std::env::var("LOCALAPPDATA")
        .map_err(|_| io::Error::new(io::ErrorKind::NotFound, "LOCALAPPDATA is not set"))?;

    Ok(Path::new(&base).join("reorganize").join("runs"))
}

pub struct RunLog {
    path: PathBuf,
    writer: io::BufWriter<fs::File>,
    entries: usize,
}

impl RunLog {
    pub fn create() -> io::Result<RunLog> {
        RunLog::create_in(log_directory()?)
    }

    /// Split out from `create` so tests can log somewhere other than the
    /// user's real application data directory.
    pub fn create_in(directory: PathBuf) -> io::Result<RunLog> {
        fs::create_dir_all(&directory)?;

        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| io::Error::other("system clock is before the unix epoch"))?
            .as_millis();

        // Milliseconds stay 13 digits until well past 2200, so the fixed width
        // keeps a plain filename sort in chronological order. The pid keeps two
        // runs started in the same millisecond apart.
        let path = directory.join(format!("{stamp}-{}.tsv", std::process::id()));
        let file = fs::File::create(&path)?;

        Ok(RunLog {
            path,
            writer: io::BufWriter::new(file),
            entries: 0,
        })
    }

    pub fn record(&mut self, from: &Path, to: &Path) -> io::Result<()> {
        writeln!(self.writer, "{}\t{}", from.display(), to.display())?;
        self.entries += 1;

        Ok(())
    }

    /// Flushes the log, discarding it if the run moved nothing.
    pub fn finish(mut self) -> io::Result<Option<PathBuf>> {
        self.writer.flush()?;

        if self.entries == 0 {
            fs::remove_file(&self.path)?;

            return Ok(None);
        }

        Ok(Some(self.path))
    }
}

pub fn latest_log() -> io::Result<Option<PathBuf>> {
    let directory = log_directory()?;

    if !directory.is_dir() {
        return Ok(None);
    }

    let mut logs: Vec<PathBuf> = fs::read_dir(&directory)?
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|extension| extension == "tsv"))
        .collect();

    logs.sort();

    Ok(logs.pop())
}

pub fn parse_log(contents: &str) -> Vec<(PathBuf, PathBuf)> {
    contents
        .lines()
        .filter_map(|line| {
            let (from, to) = line.split_once('\t')?;

            if from.is_empty() || to.is_empty() {
                return None;
            }

            Some((PathBuf::from(from), PathBuf::from(to)))
        })
        .collect()
}

pub struct Reversal {
    pub restored: usize,
    pub skipped: Vec<(PathBuf, String)>,
}

/// Moves every file in `log` back where it came from, newest move first.
///
/// A move is skipped rather than forced when the file is no longer where the
/// log says it was left, or when something already occupies the original path.
pub fn revert(log: &Path) -> io::Result<Reversal> {
    let contents = fs::read_to_string(log)?;

    let mut restored = 0;
    let mut skipped = Vec::new();

    for (from, to) in parse_log(&contents).into_iter().rev() {
        if !to.exists() {
            skipped.push((to, "no longer at the destination".to_string()));
            continue;
        }

        if from.exists() {
            skipped.push((from, "something is already at the original path".to_string()));
            continue;
        }

        if let Some(parent) = from.parent()
            && let Err(error) = fs::create_dir_all(parent)
        {
            skipped.push((from, error.to_string()));
            continue;
        }

        match fs::rename(&to, &from) {
            Ok(()) => restored += 1,
            Err(error) => skipped.push((to, error.to_string())),
        }
    }

    Ok(Reversal { restored, skipped })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    static COUNTER: AtomicUsize = AtomicUsize::new(0);

    fn temp_dir() -> PathBuf {
        let unique = COUNTER.fetch_add(1, Ordering::Relaxed);

        let dir = std::env::temp_dir().join(format!(
            "reorganize-undo-{}-{}",
            std::process::id(),
            unique
        ));

        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();

        dir
    }

    fn write_file(path: PathBuf, contents: &str) -> PathBuf {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }

        fs::write(&path, contents).unwrap();

        path
    }

    #[test]
    fn parses_tab_separated_moves() {
        let moves = parse_log("a.txt\tDocuments\\a.txt\nb.jpg\tImages\\b.jpg\n");

        assert_eq!(moves.len(), 2);
        assert_eq!(moves[0].0, PathBuf::from("a.txt"));
        assert_eq!(moves[0].1, PathBuf::from("Documents\\a.txt"));
    }

    #[test]
    fn ignores_blank_and_malformed_lines() {
        let moves = parse_log("\n\nnot-a-move\na\tb\n\t\n");

        assert_eq!(moves.len(), 1);
        assert_eq!(moves[0], (PathBuf::from("a"), PathBuf::from("b")));
    }

    // Spaces are the common case on Windows; tabs are illegal in file names,
    // which is why a plain TSV needs no escaping.
    #[test]
    fn paths_with_spaces_survive_a_round_trip() {
        let moves = parse_log("my file.txt\tDocuments\\my file.txt\n");

        assert_eq!(moves.len(), 1);
        assert_eq!(moves[0].1, PathBuf::from("Documents\\my file.txt"));
    }

    #[test]
    fn reverting_puts_files_back() {
        let root = temp_dir();

        let original = root.join("holiday.jpg");
        let moved = write_file(root.join("Images/holiday.jpg"), "photo");

        let log = write_file(
            root.join("run.tsv"),
            &format!("{}\t{}\n", original.display(), moved.display()),
        );

        let reversal = revert(&log).unwrap();

        assert_eq!(reversal.restored, 1);
        assert!(reversal.skipped.is_empty());
        assert_eq!(fs::read_to_string(&original).unwrap(), "photo");
        assert!(!moved.exists());

        fs::remove_dir_all(&root).unwrap();
    }

    // Undo must never clobber a file the user created after the run.
    #[test]
    fn reverting_skips_when_the_original_path_is_occupied() {
        let root = temp_dir();

        let original = write_file(root.join("holiday.jpg"), "a different file");
        let moved = write_file(root.join("Images/holiday.jpg"), "photo");

        let log = write_file(
            root.join("run.tsv"),
            &format!("{}\t{}\n", original.display(), moved.display()),
        );

        let reversal = revert(&log).unwrap();

        assert_eq!(reversal.restored, 0);
        assert_eq!(reversal.skipped.len(), 1);
        assert_eq!(fs::read_to_string(&original).unwrap(), "a different file");
        assert_eq!(fs::read_to_string(&moved).unwrap(), "photo");

        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn reverting_skips_a_file_that_has_since_been_deleted() {
        let root = temp_dir();

        let log = write_file(
            root.join("run.tsv"),
            &format!(
                "{}\t{}\n",
                root.join("gone.txt").display(),
                root.join("Documents/gone.txt").display()
            ),
        );

        let reversal = revert(&log).unwrap();

        assert_eq!(reversal.restored, 0);
        assert_eq!(reversal.skipped.len(), 1);

        fs::remove_dir_all(&root).unwrap();
    }

    // Moves are undone newest-first, so a file that was renamed out of the way
    // is restored before the file that displaced it.
    #[test]
    fn reverting_restores_duplicates_in_reverse_order() {
        let root = temp_dir();

        let first_original = root.join("notes.txt");
        let first_moved = write_file(root.join("Documents/notes.txt"), "first");

        let second_original = root.join("sub/notes.txt");
        let second_moved = write_file(root.join("Documents/notes (1).txt"), "second");

        let log = write_file(
            root.join("run.tsv"),
            &format!(
                "{}\t{}\n{}\t{}\n",
                first_original.display(),
                first_moved.display(),
                second_original.display(),
                second_moved.display()
            ),
        );

        let reversal = revert(&log).unwrap();

        assert_eq!(reversal.restored, 2);
        assert_eq!(fs::read_to_string(&first_original).unwrap(), "first");
        assert_eq!(fs::read_to_string(&second_original).unwrap(), "second");

        fs::remove_dir_all(&root).unwrap();
    }

    // The whole point of the log: a real run must be fully reversible,
    // including the files that were renamed to avoid a collision.
    #[test]
    fn a_run_can_be_undone_completely() {
        use crate::organizer::relocator::Relocator;

        let root = temp_dir();
        let logs = temp_dir();

        write_file(root.join("holiday.jpg"), "photo");
        write_file(root.join("notes.txt"), "notes");
        write_file(root.join("LICENSE"), "license");
        write_file(root.join("Documents/notes.txt"), "a pre-existing file");

        let sources: Vec<PathBuf> = ["holiday.jpg", "notes.txt", "LICENSE"]
            .iter()
            .map(|name| root.join(name))
            .collect();

        let mut relocator = Relocator::new();
        let mut log = RunLog::create_in(logs.clone()).unwrap();

        for source in &sources {
            let destination = relocator.relocate(source, &root).unwrap();
            log.record(source, &destination).unwrap();
        }

        let log_path = log.finish().unwrap().expect("run moved files");

        // Everything left the root, and the pre-existing file was not touched.
        for source in &sources {
            assert!(!source.exists(), "{} should have moved", source.display());
        }
        assert_eq!(
            fs::read_to_string(root.join("Documents/notes.txt")).unwrap(),
            "a pre-existing file"
        );
        assert_eq!(
            fs::read_to_string(root.join("Documents/notes (1).txt")).unwrap(),
            "notes"
        );

        let reversal = revert(&log_path).unwrap();

        assert_eq!(reversal.restored, 3);
        assert!(reversal.skipped.is_empty(), "{:?}", reversal.skipped);

        assert_eq!(fs::read_to_string(root.join("holiday.jpg")).unwrap(), "photo");
        assert_eq!(fs::read_to_string(root.join("notes.txt")).unwrap(), "notes");
        assert_eq!(fs::read_to_string(root.join("LICENSE")).unwrap(), "license");
        assert_eq!(
            fs::read_to_string(root.join("Documents/notes.txt")).unwrap(),
            "a pre-existing file"
        );

        fs::remove_dir_all(&root).unwrap();
        fs::remove_dir_all(&logs).unwrap();
    }

    #[test]
    fn a_run_that_moved_nothing_leaves_no_log() {
        let logs = temp_dir();

        let log = RunLog::create_in(logs.clone()).unwrap();

        assert!(log.finish().unwrap().is_none());

        let remaining: Vec<_> = fs::read_dir(&logs).unwrap().collect();
        assert!(remaining.is_empty(), "an empty run should clean up after itself");

        fs::remove_dir_all(&logs).unwrap();
    }
}
