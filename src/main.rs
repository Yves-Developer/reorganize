mod directory;
mod organizer;

use std::path::Path;
use std::time::Duration;

use colored::Colorize;
use indicatif::{ProgressBar, ProgressStyle};
use inquire::Confirm;

use organizer::relocator::relocate_file;

use directory::path::get_directory_path;
use directory::scanner::scan_directory;
use directory::selector::select_directory;

fn main() {
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
  ╚═╝  ╚══════╝ ╚═╝  ╚═╝ ╚═╝  ╚═╝ ╚═╝  ╚═╝╚═╝  ╚═══╝╚═╝  ╚═══╝╚═╝╚══════╝╚══════╝
    "#
        .cyan()
    );

    println!(
        "{}",
        "                    Organize your files effortlessly.".dimmed()
    );

    println!();

    println!("{}", "  REORGANIZE".bold().cyan());
    println!("{}", "  Organize your files effortlessly.".dimmed());
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
    // Custom Path Check
    // =========================

    if directory == "Custom path" {
        println!();
        println!(
            "{} {}",
            "×".red(),
            "Custom path is not implemented yet.".red()
        );

        return;
    }

    // =========================
    // Confirmation
    // =========================

    let confirmed = Confirm::new(&format!("Do you want to organize \"{}\"?", directory))
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

    // =========================
    // Resolve Directory Path
    // =========================

    let path = get_directory_path(&directory);
    let rootpath = Path::new(&path);

    println!(
        "{} {}",
        "✓".green(),
        format!("Scanning {}...", directory).bold()
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
        format!("Found {} files to organize.", total).bold()
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

    for entry in files {
        let curr_path = entry.path();

        relocate_file(&curr_path, rootpath);

        progress.inc(1);
    }

    // =========================
    // Finish Progress
    // =========================

    progress.finish_with_message("Organization complete!");

    // =========================
    // Final Success State
    // =========================

    println!();

    println!("{}", "  -----------------------------".dimmed());

    println!(
        "  {} {}",
        "✓".green(),
        format!("{} files processed.", total).green().bold()
    );

    println!(
        "  {} {}",
        "✓".green(),
        "Organization complete!".green().bold()
    );

    println!("{}", "  -----------------------------".dimmed());

    println!();
}
