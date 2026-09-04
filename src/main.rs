use std::io;
use std::env;
use std::fs;

fn fix_dupplicate(
    destpath: &mut String,
    filestem: &str,
    extension: &str,
    currpath: &str,
) {
    let mut counter = 0;

    while std::path::Path::new(destpath).exists() {
        counter += 1;

        *destpath =
            format!("{currpath}\\{filestem} ({}).{extension}", counter);
    }
}

fn main() {
    // starting..
    let directory;

    println!("Enter Directory name to organize: ");
    let mut input = String::new();

    // read line using stdin
    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read line");

    // trim empty space
    directory = input.trim();

    if !directory.is_empty() {
        println!("Organising your {directory} folder...");

        let home_path =
            env::var("USERPROFILE").expect("Could not find home directory");

        println!("HomePath: {}", home_path);

        let path = format!("{home_path}\\{directory}");

        let entries = match fs::read_dir(&path) {
            Ok(entries) => entries,
            Err(_) => {
                println!("Directory does not exist.");
                return;
            }
        };

        for entry in entries {
            let entry = entry.expect("Failed to read entry");
            let path_name = entry.path();

            if path_name.is_file() {
                if let Some(filename) = path_name.file_name() {
                    if let Some(extension) = path_name.extension() {
                        let extension = extension.to_string_lossy().to_lowercase();

                        match extension.as_str() {
                            // Images
                            "jpg" | "jpeg" | "jpe" | "png" | "gif" | "webp"
                            | "avif" | "bmp" | "dib" | "tif" | "tiff" | "svg"
                            | "ico" | "heic" | "heif" | "raw" | "cr2" | "cr3"
                            | "nef" | "nrw" | "arw" | "dng" | "orf" | "rw2"
                            | "pef" | "sr2" | "raf" => {
                                println!(
                                    "FILE: {0} | {1} → Image",
                                    filename.to_string_lossy(),
                                    extension
                                );

                                let imagepath = format!("{path}\\Images");
                                let mut imagedestination =
                                    format!("{imagepath}\\{}", filename.to_string_lossy());

                                let filestem = path_name
                                    .file_stem()
                                    .unwrap()
                                    .to_string_lossy();

                                fix_dupplicate(
                                    &mut imagedestination,
                                    &filestem,
                                    &extension,
                                    &imagepath,
                                );

                                fs::create_dir_all(imagepath)
                                    .expect("Failed to create Images directory");

                                fs::rename(path_name, imagedestination)
                                    .expect("Failed to move file");
                            }

                            // Videos
                            "mp4" | "mkv" | "avi" | "mov" | "wmv" | "flv"
                            | "webm" | "m4v" | "3gp" | "3g2" | "mpeg" | "mpg"
                            | "mpe" | "mpv" | "m2v" | "mts" | "m2ts" | "vob"
                            | "ogv" | "asf" | "rm" | "rmvb" | "divx" | "f4v"
                            | "f4p" | "f4a" | "f4b" => {
                                println!(
                                    "FILE: {0} | {1} → Video",
                                    filename.to_string_lossy(),
                                    extension
                                );

                                let videopath = format!("{path}\\Videos");
                                let mut videodestination =
                                    format!("{videopath}\\{}", filename.to_string_lossy());

                                let filestem = path_name
                                    .file_stem()
                                    .unwrap()
                                    .to_string_lossy();

                                fix_dupplicate(
                                    &mut videodestination,
                                    &filestem,
                                    &extension,
                                    &videopath,
                                );

                                fs::create_dir_all(videopath)
                                    .expect("Failed to create Video directory");

                                fs::rename(path_name, videodestination)
                                    .expect("Failed to move file");
                            }

                            // Audio
                            "mp3" | "wav" | "flac" | "aac" | "m4a" | "m4b"
                            | "ogg" | "oga" | "opus" | "wma" | "aiff" | "aif"
                            | "aifc" | "alac" | "ape" | "amr" | "mid" | "midi"
                            | "mka" | "ac3" | "dts" | "caf" | "au" | "ra"
                            | "ram" => {
                                println!(
                                    "FILE: {0} | {1} → Audio",
                                    filename.to_string_lossy(),
                                    extension
                                );

                                let audiopath = format!("{path}\\Audios");
                                let mut audiodestination =
                                    format!("{audiopath}\\{}", filename.to_string_lossy());

                                let filestem = path_name
                                    .file_stem()
                                    .unwrap()
                                    .to_string_lossy();

                                fix_dupplicate(
                                    &mut audiodestination,
                                    &filestem,
                                    &extension,
                                    &audiopath,
                                );

                                fs::create_dir_all(audiopath)
                                    .expect("Failed to create Audio directory");

                                fs::rename(path_name, audiodestination)
                                    .expect("Failed to move file");
                            }

                            // Documents
                            "pdf" | "doc" | "docx" | "docm" | "dot" | "dotx"
                            | "dotm" | "odt" | "ott" | "rtf" | "txt" | "md"
                            | "markdown" | "tex" | "pages" | "wpd" | "wps"
                            | "csv" | "tsv" | "xls" | "xlsx" | "xlsm" | "xlsb"
                            | "xlt" | "xltx" | "xltm" | "ods" | "ots" | "ppt"
                            | "pptx" | "pptm" | "pps" | "ppsx" | "odp" | "otp"
                            | "key" => {
                                println!(
                                    "FILE: {0} | {1} → Document",
                                    filename.to_string_lossy(),
                                    extension
                                );

                                let docpath = format!("{path}\\Documents");
                                let mut docdestination =
                                    format!("{docpath}\\{}", filename.to_string_lossy());

                                let filestem = path_name
                                    .file_stem()
                                    .unwrap()
                                    .to_string_lossy();

                                fix_dupplicate(
                                    &mut docdestination,
                                    &filestem,
                                    &extension,
                                    &docpath,
                                );

                                fs::create_dir_all(docpath)
                                    .expect("Failed to create Documents directory");

                                fs::rename(path_name, docdestination)
                                    .expect("Failed to move file");
                            }

                            // Archives
                            "zip" | "rar" | "7z" | "tar" | "gz" | "bz2"
                            | "xz" | "zst" | "tgz" | "tbz2" => {
                                println!(
                                    "FILE: {0} | {1} → Archive",
                                    filename.to_string_lossy(),
                                    extension
                                );

                                let folder = format!("{path}\\Archives");
                                let mut destination =
                                    format!("{folder}\\{}", filename.to_string_lossy());

                                let filestem = path_name
                                    .file_stem()
                                    .unwrap()
                                    .to_string_lossy();

                                fix_dupplicate(
                                    &mut destination,
                                    &filestem,
                                    &extension,
                                    &folder,
                                );

                                fs::create_dir_all(folder)
                                    .expect("Failed to create Archives directory");

                                fs::rename(path_name, destination)
                                    .expect("Failed to move file");
                            }

                            // Applications
                            "exe" | "msi" | "msix" | "msixbundle" | "appx"
                            | "appxbundle" | "com" | "bat" | "cmd" | "scr" => {
                                println!(
                                    "FILE: {0} | {1} → Application",
                                    filename.to_string_lossy(),
                                    extension
                                );

                                let folder = format!("{path}\\Applications");
                                let mut destination =
                                    format!("{folder}\\{}", filename.to_string_lossy());

                                let filestem = path_name
                                    .file_stem()
                                    .unwrap()
                                    .to_string_lossy();

                                fix_dupplicate(
                                    &mut destination,
                                    &filestem,
                                    &extension,
                                    &folder,
                                );

                                fs::create_dir_all(folder)
                                    .expect("Failed to create Applications directory");

                                fs::rename(path_name, destination)
                                    .expect("Failed to move file");
                            }

                            // Code / Programming
                            "rs" | "js" | "jsx" | "ts" | "tsx" | "py" | "java"
                            | "c" | "h" | "cpp" | "hpp" | "cs" | "go" | "rb"
                            | "php" | "swift" | "kt" | "kts" | "dart" | "lua"
                            | "r" | "sql" | "html" | "htm" | "css" | "scss"
                            | "sass" | "less" | "vue" | "svelte" | "astro"
                            | "json" | "xml" | "yaml" | "yml" | "toml" | "ini"
                            | "env" | "sh" | "ps1" | "fish" => {
                                println!(
                                    "FILE: {0} | {1} → Code",
                                    filename.to_string_lossy(),
                                    extension
                                );

                                let folder = format!("{path}\\Code");
                                let mut destination =
                                    format!("{folder}\\{}", filename.to_string_lossy());

                                let filestem = path_name
                                    .file_stem()
                                    .unwrap()
                                    .to_string_lossy();

                                fix_dupplicate(
                                    &mut destination,
                                    &filestem,
                                    &extension,
                                    &folder,
                                );

                                fs::create_dir_all(folder)
                                    .expect("Failed to create Code directory");

                                fs::rename(path_name, destination)
                                    .expect("Failed to move file");
                            }

                            // Fonts
                            "ttf" | "otf" | "woff" | "woff2" | "eot" => {
                                println!(
                                    "FILE: {0} | {1} → Font",
                                    filename.to_string_lossy(),
                                    extension
                                );

                                let folder = format!("{path}\\Fonts");
                                let mut destination =
                                    format!("{folder}\\{}", filename.to_string_lossy());

                                let filestem = path_name
                                    .file_stem()
                                    .unwrap()
                                    .to_string_lossy();

                                fix_dupplicate(
                                    &mut destination,
                                    &filestem,
                                    &extension,
                                    &folder,
                                );

                                fs::create_dir_all(folder)
                                    .expect("Failed to create Fonts directory");

                                fs::rename(path_name, destination)
                                    .expect("Failed to move file");
                            }

                            // Disk Images
                            "iso" | "img" | "dmg" | "vhd" | "vhdx" | "vmdk"
                            | "qcow" | "qcow2" => {
                                println!(
                                    "FILE: {0} | {1} → Disk Image",
                                    filename.to_string_lossy(),
                                    extension
                                );

                                let folder = format!("{path}\\Disk Images");
                                let mut destination =
                                    format!("{folder}\\{}", filename.to_string_lossy());

                                let filestem = path_name
                                    .file_stem()
                                    .unwrap()
                                    .to_string_lossy();

                                fix_dupplicate(
                                    &mut destination,
                                    &filestem,
                                    &extension,
                                    &folder,
                                );

                                fs::create_dir_all(folder)
                                    .expect("Failed to create Disk Images directory");

                                fs::rename(path_name, destination)
                                    .expect("Failed to move file");
                            }

                            // Shortcuts
                            "lnk" | "url" | "webloc" => {
                                println!(
                                    "FILE: {0} | {1} → Shortcut",
                                    filename.to_string_lossy(),
                                    extension
                                );

                                let folder = format!("{path}\\Shortcuts");
                                let mut destination =
                                    format!("{folder}\\{}", filename.to_string_lossy());

                                let filestem = path_name
                                    .file_stem()
                                    .unwrap()
                                    .to_string_lossy();

                                fix_dupplicate(
                                    &mut destination,
                                    &filestem,
                                    &extension,
                                    &folder,
                                );

                                fs::create_dir_all(folder)
                                    .expect("Failed to create Shortcuts directory");

                                fs::rename(path_name, destination)
                                    .expect("Failed to move file");
                            }

                            // E-books
                            "epub" | "mobi" | "azw" | "azw3" | "fb2" | "djvu" => {
                                println!(
                                    "FILE: {0} | {1} → E-book",
                                    filename.to_string_lossy(),
                                    extension
                                );

                                let folder = format!("{path}\\E-books");
                                let mut destination =
                                    format!("{folder}\\{}", filename.to_string_lossy());

                                let filestem = path_name
                                    .file_stem()
                                    .unwrap()
                                    .to_string_lossy();

                                fix_dupplicate(
                                    &mut destination,
                                    &filestem,
                                    &extension,
                                    &folder,
                                );

                                fs::create_dir_all(folder)
                                    .expect("Failed to create E-books directory");

                                fs::rename(path_name, destination)
                                    .expect("Failed to move file");
                            }

                            // Android Builds
                            "apk" | "aab" => {
                                println!(
                                    "FILE: {0} | {1} → Android Build",
                                    filename.to_string_lossy(),
                                    extension
                                );

                                let folder = format!("{path}\\Android Builds");
                                let mut destination =
                                    format!("{folder}\\{}", filename.to_string_lossy());

                                let filestem = path_name
                                    .file_stem()
                                    .unwrap()
                                    .to_string_lossy();

                                fix_dupplicate(
                                    &mut destination,
                                    &filestem,
                                    &extension,
                                    &folder,
                                );

                                fs::create_dir_all(folder)
                                    .expect("Failed to create Android Builds directory");

                                fs::rename(path_name, destination)
                                    .expect("Failed to move file");
                            }

                            // iOS Builds
                            "ipa" => {
                                println!(
                                    "FILE: {0} | {1} → iOS Build",
                                    filename.to_string_lossy(),
                                    extension
                                );

                                let folder = format!("{path}\\iOS Builds");
                                let mut destination =
                                    format!("{folder}\\{}", filename.to_string_lossy());

                                let filestem = path_name
                                    .file_stem()
                                    .unwrap()
                                    .to_string_lossy();

                                fix_dupplicate(
                                    &mut destination,
                                    &filestem,
                                    &extension,
                                    &folder,
                                );

                                fs::create_dir_all(folder)
                                    .expect("Failed to create iOS Builds directory");

                                fs::rename(path_name, destination)
                                    .expect("Failed to move file");
                            }

                            // Other
                            _ => {
                                println!(
                                    "FILE: {0} | {1} → Other",
                                    filename.to_string_lossy(),
                                    extension
                                );
                            }
                        }
                    } else {
                        println!(
                            "{} --> No extension",
                            filename.to_string_lossy()
                        );
                    }
                }
            } else if path_name.is_dir() {
                continue;
            }
        }
    } else {
        println!("directory shouldn't be empty!");
    }
}