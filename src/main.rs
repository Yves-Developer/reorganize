mod cli;
mod directory;
mod organizer;
mod undo;

use std::path::{Path, PathBuf};
use std::time::Duration;

use colored::Colorize;
use indicatif::{ProgressBar, ProgressStyle};
use inquire::Confirm;

use organizer::relocator::Relocator;

use directory::path::get_directory_path;
use directory::scanner::scan_directory;
use directory::selector::{CUSTOM_PATH, prompt_custom_path, select_directory};

use cli::{Command, USAGE, parse_args};

fn main() {
    let command = match parse_args(std::env::args().skip(1)) {
        Ok(command) => command,

        Err(error) => {
            println!("{} {}", "×".red(), error.red());
            println!();
            println!("{USAGE}");

            return;
        }
    };

    match command {
        Command::Help => println!("{USAGE}"),
        Command::Undo => run_undo(),
        Command::Organize { dry_run } => organize(dry_run),
    }
}

fn run_undo() {
    let log = match undo::latest_log() {
        Ok(Some(log)) => log,

        Ok(None) => {
            println!("{} {}", "×".red(), "No previous run to undo.".yellow());

            return;
        }

        Err(error) => {
            println!("{} Could not read the run history: {}", "×".red(), error);

            return;
        }
    };

    let reversal = match undo::revert(&log) {
        Ok(reversal) => reversal,

        Err(error) => {
            println!("{} Could not undo the last run: {}", "×".red(), error);

            return;
        }
    };

    println!();

    println!(
        "  {} {}",
        "✓".green(),
        format!("{} {} put back.", reversal.restored, file_word(reversal.restored as u64))
            .green()
            .bold()
    );

    if reversal.skipped.is_empty() {
        if let Err(error) = std::fs::remove_file(&log) {
            println!("  {} Could not clear the run log: {}", "×".red(), error);
        }
    } else {
        println!(
            "  {} {}",
            "×".red(),
            format!("{} skipped:", reversal.skipped.len()).red().bold()
        );

        for (path, reason) in &reversal.skipped {
            let name = display_name(path);

            println!("      {} {}", name.yellow(), format!("({reason})").dimmed());
        }

        println!();
        println!(
            "  {}",
            format!("Run log kept at {}", log.display()).dimmed()
        );
    }

    println!();
}

/// "1 file" / "2 files".
fn file_word(count: u64) -> &'static str {
    if count == 1 { "file" } else { "files" }
}

fn display_name(path: &Path) -> String {
    path.file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_else(|| path.to_string_lossy().into_owned())
}

fn organize(dry_run: bool) {
    // =========================
    // CLI Header
    // =========================

    println!(
        "{}",
        r#"
  ██████╗ ███████╗ ██████╗ ██████╗  ██████╗  █████╗ ███╗   ██╗██╗███████╗███████╗
  ██╔══██╗██╔════╝██╔═══██╗██╔══██╗██╔════╝ ██╔══██╗████╗  ██║██║╚══███╔╝██╔════╝
  ██████╔╝█████╗  ██║   ██║██████╔╝██║  ███╗███████║██╔██╗ ██║██║  ███╔╝ █████╗
  ██╔══██╗██╔══╝  ██║   ██║██╔══██╗██║   ██║██╔══██║██║╚██╗██║██║ ███╔╝  ██╔══╝
  ██║  ██║███████╗╚██████╔╝██║  ██║╚██████╔╝██║  ██║██║ ╚████║██║███████╗███████╗
  ╚═╝  ╚═╝╚══════╝ ╚═════╝ ╚═╝  ╚═╝ ╚═════╝ ╚═╝  ╚═╝╚═╝  ╚═══╝╚═╝╚══════╝╚══════╝
    "#
        .cyan()
    );

    println!(
        "{}",
        "              Organize your files effortlessly.".dimmed()
    );

    println!();

    // =========================
    // Directory Selection
    // =========================

    let directory = match select_directory() {
        Ok(directory) => directory,
        Err(error) => {
            println!();
            println!("{} {}", "×".red(), error.red());
            return;
        }
    };

    // =========================
    // Resolve Directory Path
    // =========================

    // "Custom path" is typed in full; the presets live under the home directory.
    let (label, path) = if directory == CUSTOM_PATH {
        match prompt_custom_path() {
            Ok(path) => (path.clone(), path),

            Err(error) => {
                println!();
                println!("{} {}", "×".red(), error.red());

                return;
            }
        }
    } else {
        (directory.clone(), get_directory_path(&directory))
    };

    let rootpath = Path::new(&path);

    if !rootpath.is_dir() {
        println!();
        println!(
            "{} {}",
            "×".red(),
            format!("\"{}\" is not an existing folder.", path).red()
        );

        return;
    }

    // =========================
    // Confirmation
    // =========================

    let confirmed = Confirm::new(&format!("Do you want to organize \"{}\"?", label))
        .with_default(false)
        .prompt();

    match confirmed {
        Ok(true) => {}

        Ok(false) => {
            println!();
            println!("{} {}", "×".red(), "Organization cancelled.".yellow());

            return;
        }

        Err(_) => {
            println!();
            println!("{} {}", "×".red(), "Confirmation cancelled.".red());

            return;
        }
    }

    println!();

    println!(
        "{} {}",
        "✓".green(),
        format!("Scanning {}...", label).bold()
    );

    // =========================
    // Scan Directory
    // =========================

    let entries = match scan_directory(&path) {
        Ok(entries) => entries,

        Err(error) => {
            println!();
            println!("{} Failed to scan directory: {}", "×".red(), error);

            return;
        }
    };

    // =========================
    // Keep Only Files
    // =========================

    let files: Vec<_> = entries
        .into_iter()
        .filter(|entry| entry.path().is_file())
        .collect();

    let total = files.len() as u64;

    println!(
        "{} {}",
        "✓".green(),
        format!("Found {} {} to organize.", total, file_word(total)).bold()
    );

    // =========================
    // Nothing To Organize
    // =========================

    if total == 0 {
        println!();
        println!(
            "{} {}",
            "✓".green(),
            "No files found. Nothing to organize.".yellow()
        );

        return;
    }

    println!();

    // =========================
    // Dry Run: Preview Only
    // =========================

    if dry_run {
        let mut relocator = Relocator::new();

        for entry in &files {
            let currpath = entry.path();

            match relocator.plan(&currpath, rootpath) {
                Ok(destpath) => {
                    let destination = destpath
                        .strip_prefix(rootpath)
                        .unwrap_or(&destpath)
                        .to_string_lossy()
                        .into_owned();

                    println!(
                        "      {} {} {}",
                        display_name(&currpath).yellow(),
                        "→".dimmed(),
                        destination.cyan()
                    );
                }

                Err(error) => {
                    println!(
                        "      {} {}",
                        display_name(&currpath).red(),
                        format!("({error})").dimmed()
                    );
                }
            }
        }

        println!();
        println!("{}", "  -----------------------------".dimmed());
        println!(
            "  {} {}",
            "✓".green(),
            format!("{} {} would be organized.", total, file_word(total)).green().bold()
        );
        println!(
            "  {}",
            "Dry run — nothing was moved.".yellow().bold()
        );
        println!("{}", "  -----------------------------".dimmed());
        println!();

        return;
    }

    // =========================
    // Progress Bar
    // =========================

    let progress = ProgressBar::new(total);

    progress.set_style(
        ProgressStyle::with_template("  {spinner} {msg} [{bar:40.cyan/blue}] {pos}/{len}")
            .unwrap()
            .progress_chars("=>-"),
    );

    progress.enable_steady_tick(Duration::from_millis(100));
    progress.set_message("Organizing files...");

    // =========================
    // Organize Files
    // =========================

    let mut failures: Vec<(PathBuf, std::io::Error)> = Vec::new();
    let mut relocator = Relocator::new();

    // Without a log the run cannot be undone, so a failure to open it stops
    // the run rather than moving files that could never be put back.
    let mut log = match undo::RunLog::create() {
        Ok(log) => log,

        Err(error) => {
            progress.finish_and_clear();

            println!();
            println!("{} Could not start the run log: {}", "×".red(), error);
            println!("{}", "  Nothing was moved.".yellow());

            return;
        }
    };

    for entry in files {
        let curr_path = entry.path();

        match relocator.relocate(&curr_path, rootpath) {
            Ok(destpath) => {
                if let Err(error) = log.record(&curr_path, &destpath) {
                    progress.suspend(|| {
                        println!("{} Could not record a move: {}", "×".red(), error);
                    });
                }
            }

            Err(error) => failures.push((curr_path, error)),
        }

        progress.inc(1);
    }

    let log_path = log.finish();

    // =========================
    // Finish Progress
    // =========================

    if failures.is_empty() {
        progress.finish_with_message("All files organized.");
    } else {
        progress.finish_with_message("Finished with some errors.");
    }

    // =========================
    // Final State
    // =========================

    let failed = failures.len() as u64;
    let moved = total - failed;

    println!();

    println!("{}", "  -----------------------------".dimmed());

    println!(
        "  {} {}",
        "✓".green(),
        format!("{} of {} {} organized.", moved, total, file_word(total))
            .green()
            .bold()
    );

    if failures.is_empty() {
        println!(
            "  {} {}",
            "✓".green(),
            "Organization complete!".green().bold()
        );
    } else {
        println!(
            "  {} {}",
            "×".red(),
            format!("{} could not be moved:", failed).red().bold()
        );

        for (path, error) in &failures {
            let name = path
                .file_name()
                .map(|name| name.to_string_lossy().into_owned())
                .unwrap_or_else(|| path.to_string_lossy().into_owned());

            println!("      {} {}", name.yellow(), format!("({error})").dimmed());
        }
    }

    if let Ok(Some(_)) = log_path {
        println!(
            "  {}",
            "Run `reorganize undo` to put everything back.".dimmed()
        );
    }

    println!("{}", "  -----------------------------".dimmed());

    println!();
}
