// Scanner
use std::fs;

pub fn scan_directory(path: &str) -> Result<Vec<fs::DirEntry>, std::io::Error> {
    let entries = fs::read_dir(path)?;

    let entries: Vec<fs::DirEntry> = entries.filter_map(|entry| entry.ok()).collect();

    Ok(entries)
}
