use std::io;
use std::io::Write;
use std::env;
use std::fs;
use colored::Colorize;

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
    
    println!("{}", r#"
  ██████╗ ███████╗ ██████╗ ██████╗  ██████╗  █████╗ ███╗   ██╗██╗███████╗███████╗
  ██╔══██╗██╔════╝██╔═══██╗██╔══██╗██╔════╝ ██╔══██╗████╗  ██║██║╚══███╔╝██╔════╝
  ██████╔╝█████╗  ██║   ██║██████╔╝██║  ███╗███████║██╔██╗ ██║██║  ███╔╝ █████╗
  ██╔══██╗██╔══╝  ██║   ██║██╔══██╗██║   ██║██╔══██║██║╚██╗██║██║ ███╔╝  ██╔══╝
  ██║  ██║███████╗╚██████╔╝██║  ██║╚██████╔╝██║  ██║██║ ╚████║██║███████╗███████╗
  ╚═╝  ╚══════╝ ╚═════╝ ╚═╝  ╚═╝ ╚═════╝ ╚═╝  ╚═╝╚═╝  ╚═══╝╚═╝╚══════╝╚══════╝
    "#.cyan());

    println!("{}", "                    Organize your files effortlessly.".dimmed());
    println!();

    // CLI Header
    println!();
    println!("{}", "  REORGANIZE".bold().cyan());
    println!("{}", "  Organize your files effortlessly.".dimmed());
    println!();

    // Directory prompt
    print!("{} ", "?".cyan().bold());
    print!("{}", "Directory to organize: ".bold());

    io::stdout().flush().unwrap();

    let mut input = String::new();

    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read line");

    let directory = input.trim();

    if directory.is_empty() {
        println!();
        println!(
            "{} {}",
            "✖".red().bold(),
            "Directory shouldn't be empty!".red()
        );
        return;
    }

    println!();

    println!(
        "{} {}",
        "✔".green().bold(),
        format!("Scanning {}...", directory).bold()
    );

    let home_path =
        env::var("USERPROFILE").expect("Could not find home directory");

    let path = format!("{home_path}\\{directory}");

    let entries = match fs::read_dir(&path) {
        Ok(entries) => entries,
        Err(_) => {
            println!(
                "{} {}",
                "✖".red().bold(),
                "Directory does not exist.".red()
            );
            return;
        }
    };

    println!();

    println!("{}", "  Organizing files...".bold());
    println!();

    for entry in entries {
        let entry = entry.expect("Failed to read entry");
        let path_name = entry.path();

        if path_name.is_file() {
            if let Some(filename) = path_name.file_name() {
                if let Some(extension) = path_name.extension() {

                    let extension =
                        extension.to_string_lossy().to_lowercase();

                    match extension.as_str() {

                        // Images
                        "jpg" | "jpeg" | "jpe" | "png" | "gif" | "webp"
                        | "avif" | "bmp" | "dib" | "tif" | "tiff" | "svg"
                        | "ico" | "heic" | "heif" | "raw" | "cr2" | "cr3"
                        | "nef" | "nrw" | "arw" | "dng" | "orf" | "rw2"
                        | "pef" | "sr2" | "raf" => {

                            println!(
                                "  {} {} {}",
                                "→".dimmed(),
                                filename.to_string_lossy(),
                                "Image".cyan()
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
                                "  {} {} {}",
                                "→".dimmed(),
                                filename.to_string_lossy(),
                                "Video".magenta()
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
                                "  {} {} {}",
                                "→".dimmed(),
                                filename.to_string_lossy(),
                                "Audio".yellow()
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
                                "  {} {} {}",
                                "→".dimmed(),
                                filename.to_string_lossy(),
                                "Document".blue()
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
                                "  {} {} {}",
                                "→".dimmed(),
                                filename.to_string_lossy(),
                                "Archive".red()
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
                                "  {} {} {}",
                                "→".dimmed(),
                                filename.to_string_lossy(),
                                "Application".green()
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

                        // Code
                        "rs" | "js" | "jsx" | "ts" | "tsx" | "py"
                        | "java" | "c" | "h" | "cpp" | "hpp" | "cs"
                        | "go" | "rb" | "php" | "swift" | "kt" | "kts"
                        | "dart" | "lua" | "r" | "sql" | "html" | "htm"
                        | "css" | "scss" | "sass" | "less" | "vue"
                        | "svelte" | "astro" | "json" | "xml" | "yaml"
                        | "yml" | "toml" | "ini" | "env" | "sh"
                        | "ps1" | "fish" => {

                            println!(
                                "  {} {} {}",
                                "→".dimmed(),
                                filename.to_string_lossy(),
                                "Code".cyan()
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
                                "  {} {} {}",
                                "→".dimmed(),
                                filename.to_string_lossy(),
                                "Font".cyan()
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
                        "iso" | "img" | "dmg" | "vhd" | "vhdx"
                        | "vmdk" | "qcow" | "qcow2" => {

                            println!(
                                "  {} {} {}",
                                "→".dimmed(),
                                filename.to_string_lossy(),
                                "Disk Image".yellow()
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
                                "  {} {} {}",
                                "→".dimmed(),
                                filename.to_string_lossy(),
                                "Shortcut".blue()
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
                                "  {} {} {}",
                                "→".dimmed(),
                                filename.to_string_lossy(),
                                "E-book".magenta()
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
                                "  {} {} {}",
                                "→".dimmed(),
                                filename.to_string_lossy(),
                                "Android Build".green()
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
                                "  {} {} {}",
                                "→".dimmed(),
                                filename.to_string_lossy(),
                                "iOS Build".blue()
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
                                "  {} {} {}",
                                "→".dimmed(),
                                filename.to_string_lossy(),
                                "Other".dimmed()
                            );
                        }
                    }
                } else {
                    println!(
                        "  {} {} {}",
                        "→".dimmed(),
                        filename.to_string_lossy(),
                        "No extension".dimmed()
                    );
                }
            }
        }
    }

    println!();
    println!("{}", "  ─────────────────────────────".dimmed());
    println!(
        "  {} {}",
        "✔".green().bold(),
        "Organization complete!".green().bold()
    );
    println!("{}", "  ─────────────────────────────".dimmed());
    println!();
}