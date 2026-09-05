// classifier module

pub fn classify_file(extension: Option<&str>) -> &str {
    match extension {
        // Images
        Some(
            "jpg" | "jpeg" | "jpe" | "png" | "gif" | "webp" | "avif" | "bmp" | "dib" | "tif"
            | "tiff" | "svg" | "ico" | "heic" | "heif" | "raw" | "cr2" | "cr3" | "nef" | "nrw"
            | "arw" | "dng" | "orf" | "rw2" | "pef" | "sr2" | "raf",
        ) => "Images",

        // Videos
        Some(
            "mp4" | "mkv" | "avi" | "mov" | "wmv" | "flv" | "webm" | "m4v" | "3gp" | "3g2" | "mpeg"
            | "mpg" | "mpe" | "mpv" | "m2v" | "mts" | "m2ts" | "vob" | "ogv" | "asf" | "rm"
            | "rmvb" | "divx" | "f4v" | "f4p" | "f4a" | "f4b",
        ) => "Videos",

        // Audio
        Some(
            "mp3" | "wav" | "flac" | "aac" | "m4a" | "m4b" | "ogg" | "oga" | "opus" | "wma"
            | "aiff" | "aif" | "aifc" | "alac" | "ape" | "amr" | "mid" | "midi" | "mka" | "ac3"
            | "dts" | "caf" | "au" | "ra" | "ram",
        ) => "Audio",

        // Documents
        Some(
            "pdf" | "doc" | "docx" | "docm" | "dot" | "dotx" | "dotm" | "odt" | "ott" | "rtf"
            | "txt" | "md" | "markdown" | "tex" | "pages" | "wpd" | "wps" | "csv" | "tsv" | "xls"
            | "xlsx" | "xlsm" | "xlsb" | "xlt" | "xltx" | "xltm" | "ods" | "ots" | "ppt" | "pptx"
            | "pptm" | "pps" | "ppsx" | "odp" | "otp" | "key",
        ) => "Documents",

        // Archives
        Some("zip" | "rar" | "7z" | "tar" | "gz" | "bz2" | "xz" | "zst" | "tgz" | "tbz2") => {
            "Archives"
        }

        // Applications
        Some(
            "exe" | "msi" | "msix" | "msixbundle" | "appx" | "appxbundle" | "com" | "bat" | "cmd"
            | "scr",
        ) => "Applications",

        // Code
        Some(
            "rs" | "js" | "jsx" | "ts" | "tsx" | "py" | "java" | "c" | "h" | "cpp" | "hpp" | "cs"
            | "go" | "rb" | "php" | "swift" | "kt" | "kts" | "dart" | "lua" | "r" | "sql" | "html"
            | "htm" | "css" | "scss" | "sass" | "less" | "vue" | "svelte" | "astro" | "json"
            | "xml" | "yaml" | "yml" | "toml" | "ini" | "env" | "sh" | "ps1" | "fish",
        ) => "Code",

        // Fonts
        Some("ttf" | "otf" | "woff" | "woff2" | "eot") => "Fonts",

        // Disk Images
        Some("iso" | "img" | "dmg" | "vhd" | "vhdx" | "vmdk" | "qcow" | "qcow2") => "Disk Images",

        // Shortcuts
        Some("lnk" | "url" | "webloc") => "Shortcuts",

        // E-books
        Some("epub" | "mobi" | "azw" | "azw3" | "fb2" | "djvu") => "E-books",

        // Android Builds
        Some("apk" | "aab") => "Android Builds",

        // iOS Builds
        Some("ipa") => "iOS Builds",

        // Anything else, including files without an extension
        _ => "Other",
    }
}
