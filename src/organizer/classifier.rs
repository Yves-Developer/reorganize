// classifier module

use crate::organizer::category::Category;

pub fn classify_file(extension: Option<&str>) -> Category {
    match extension {
        // Images
        Some(
            "jpg" | "jpeg" | "jpe" | "png" | "gif" | "webp" | "avif" | "bmp" | "dib" | "tif"
            | "tiff" | "svg" | "ico" | "heic" | "heif" | "raw" | "cr2" | "cr3" | "nef" | "nrw"
            | "arw" | "dng" | "orf" | "rw2" | "pef" | "sr2" | "raf",
        ) => Category::Images,

        // Videos
        Some(
            "mp4" | "mkv" | "avi" | "mov" | "wmv" | "flv" | "webm" | "m4v" | "3gp" | "3g2" | "mpeg"
            | "mpg" | "mpe" | "mpv" | "m2v" | "mts" | "m2ts" | "vob" | "ogv" | "asf" | "rm"
            | "rmvb" | "divx" | "f4v" | "f4p" | "f4a" | "f4b",
        ) => Category::Videos,

        // Audio
        Some(
            "mp3" | "wav" | "flac" | "aac" | "m4a" | "m4b" | "ogg" | "oga" | "opus" | "wma"
            | "aiff" | "aif" | "aifc" | "alac" | "ape" | "amr" | "mid" | "midi" | "mka" | "ac3"
            | "dts" | "caf" | "au" | "ra" | "ram",
        ) => Category::Audio,

        // Documents
        Some(
            "pdf" | "doc" | "docx" | "docm" | "dot" | "dotx" | "dotm" | "odt" | "ott" | "rtf"
            | "txt" | "md" | "markdown" | "tex" | "pages" | "wpd" | "wps" | "csv" | "tsv" | "xls"
            | "xlsx" | "xlsm" | "xlsb" | "xlt" | "xltx" | "xltm" | "ods" | "ots" | "ppt" | "pptx"
            | "pptm" | "pps" | "ppsx" | "odp" | "otp" | "key",
        ) => Category::Documents,

        // Archives
        Some("zip" | "rar" | "7z" | "tar" | "gz" | "bz2" | "xz" | "zst" | "tgz" | "tbz2") => {
            Category::Archives
        }

        // Applications
        Some(
            "exe" | "msi" | "msix" | "msixbundle" | "appx" | "appxbundle" | "com" | "bat" | "cmd"
            | "scr",
        ) => Category::Applications,

        // Code
        Some(
            "rs" | "js" | "jsx" | "ts" | "tsx" | "py" | "java" | "c" | "h" | "cpp" | "hpp" | "cs"
            | "go" | "rb" | "php" | "swift" | "kt" | "kts" | "dart" | "lua" | "r" | "sql" | "html"
            | "htm" | "css" | "scss" | "sass" | "less" | "vue" | "svelte" | "astro" | "json"
            | "xml" | "yaml" | "yml" | "toml" | "ini" | "env" | "sh" | "ps1" | "fish",
        ) => Category::Code,

        // Fonts
        Some("ttf" | "otf" | "woff" | "woff2" | "eot") => Category::Fonts,

        // Disk Images
        Some("iso" | "img" | "dmg" | "vhd" | "vhdx" | "vmdk" | "qcow" | "qcow2") => Category::DiskImages,

        // Shortcuts
        Some("lnk" | "url" | "webloc") => Category::Shortcuts,

        // E-books
        Some("epub" | "mobi" | "azw" | "azw3" | "fb2" | "djvu") => Category::EBooks,

        // Android Builds
        Some("apk" | "aab") => Category::AndroidBuilds,

        // iOS Builds
        Some("ipa") => Category::IosBuilds,

        // Anything else, including files without an extension
        _ => Category::Other,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_each_known_category() {
        assert_eq!(classify_file(Some("png")), Category::Images);
        assert_eq!(classify_file(Some("mkv")), Category::Videos);
        assert_eq!(classify_file(Some("flac")), Category::Audio);
        assert_eq!(classify_file(Some("pdf")), Category::Documents);
        assert_eq!(classify_file(Some("zip")), Category::Archives);
        assert_eq!(classify_file(Some("exe")), Category::Applications);
        assert_eq!(classify_file(Some("rs")), Category::Code);
        assert_eq!(classify_file(Some("ttf")), Category::Fonts);
        assert_eq!(classify_file(Some("iso")), Category::DiskImages);
        assert_eq!(classify_file(Some("lnk")), Category::Shortcuts);
        assert_eq!(classify_file(Some("epub")), Category::EBooks);
        assert_eq!(classify_file(Some("apk")), Category::AndroidBuilds);
        assert_eq!(classify_file(Some("ipa")), Category::IosBuilds);
    }

    #[test]
    fn unknown_extension_falls_back_to_other() {
        assert_eq!(classify_file(Some("qwerty")), Category::Other);
    }

    #[test]
    fn missing_extension_falls_back_to_other() {
        assert_eq!(classify_file(None), Category::Other);
    }

    #[test]
    fn empty_extension_falls_back_to_other() {
        assert_eq!(classify_file(Some("")), Category::Other);
    }

    // classify_file matches lowercase arms only; the caller is responsible
    // for lowercasing. This pins that contract so it cannot drift silently.
    #[test]
    fn matching_is_case_sensitive_by_design() {
        assert_eq!(classify_file(Some("PNG")), Category::Other);
        assert_eq!(classify_file(Some("png")), Category::Images);
    }
}
