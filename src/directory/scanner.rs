// Scanner
use std::fs;

pub fn scan_directory(path: &str) -> Result<Vec<fs::DirEntry>, std::io::Error> {
    let entries = match fs::read_dir(path) {
        Ok(entries) => entries,
        Err(error) => return Err(error),
    };

    let entries: Vec<fs::DirEntry> = entries.filter_map(|entry| entry.ok()).collect();

    Ok(entries)
}
