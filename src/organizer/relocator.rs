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

    if let Some(extension) = extension.as_deref() {
        resolve_duplicate(&mut destinationpath, &filestem, extension, &newpath);
    }
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
