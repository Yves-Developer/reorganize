use std::path::PathBuf;

pub fn resolve_duplicate(
    destpath: &mut PathBuf,
    filestem: &str,
    extension: &str,
    currpath: &PathBuf,
) {
    let mut counter = 0;

    while destpath.exists() {
        counter += 1;

        *destpath = currpath.join(format!("{filestem} ({}).{extension}", counter));
    }
}
