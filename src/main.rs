mod ai;
mod cli;
mod directory;
mod inspect;
mod naming;
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

use ai::ollama::Ollama;
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
        Command::AiStatus => run_ai_status(),
        Command::Organize { dry_run, ai, path } => organize(dry_run, ai, path),
    }
}

fn run_ai_status() {
    let ollama = Ollama::from_env();

    println!();
    println!("  {} {}", "Server:".dimmed(), ollama.host());
    println!("  {} {}", "Model: ".dimmed(), ollama.model());
    println!();

    let installed = match ollama.installed_models() {
        Ok(installed) => installed,

        Err(error) => {
            println!("  {} {}", "×".red(), "No local model server reached.".red().bold());
            println!("      {}", format!("({error})").dimmed());
            println!();
            println!(
                "  {}",
                "Install Ollama and run `ollama serve`, then pull a model:".dimmed()
            );
            println!("  {}", format!("    ollama pull {}", ollama.model()).dimmed());
            println!();
            println!(
                "  {}",
                "reorganize works without it; AI features stay switched off.".dimmed()
            );
            println!();

            return;
        }
    };

    println!("  {} {}", "✓".green(), "Server reachable.".green().bold());

    if installed.is_empty() {
        println!("  {} {}", "×".red(), "No models pulled yet.".yellow());
    } else {
        println!("  {} {}", "✓".green(), format!("{} installed:", installed.len()).bold());

        for name in &installed {
            println!("      {}", name.cyan());
        }
    }

    println!();

    if ollama.has_model(&installed) {
        println!(
            "  {} {}",
            "✓".green(),
            format!("\"{}\" is ready to use.", ollama.model()).green().bold()
        );
    } else {
        println!(
            "  {} {}",
            "×".red(),
            format!("\"{}\" is not installed.", ollama.model()).red().bold()
        );
        println!("  {}", format!("    ollama pull {}", ollama.model()).dimmed());
    }

    println!();
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

/// Turns a menu choice into a display label and a concrete path.
fn resolve_selection(directory: String) -> (String, String) {
    if directory == CUSTOM_PATH {
        match prompt_custom_path() {
            Ok(path) => (path.clone(), path),

            Err(error) => {
                println!();
                println!("{} {}", "×".red(), error.red());

                std::process::exit(0);
            }
        }
    } else {
        let path = get_directory_path(&directory);

        (directory, path)
    }
}

fn organize(dry_run: bool, ai: bool, given_path: Option<String>) {
    let path_was_given = given_path.is_some();
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

    // A folder given on the command line skips the menu entirely, which is
    // how you reach somewhere nested like Downloads\Invoices4.
    let (label, path) = match given_path {
        Some(given) => (given.clone(), given),

        None => {
            let directory = match select_directory() {
                Ok(directory) => directory,
                Err(error) => {
                    println!();
                    println!("{} {}", "×".red(), error.red());
                    return;
                }
            };

            resolve_selection(directory)
        }
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

    // Naming the folder on the command line already answers this, and a dry
    // run does not touch anything, so neither case needs asking.
    if !path_was_given && !dry_run {
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
    // Build the Plan
    // =========================

    let mut relocator = Relocator::new();
    let mut planned: Vec<PlannedMove> = Vec::new();
    let mut failures: Vec<(PathBuf, std::io::Error)> = Vec::new();

    let ollama = if ai { Some(Ollama::from_env()) } else { None };

    if let Some(ollama) = &ollama {
        // Say so up front: this is the slow path, and a missing server means
        // every file quietly falls back to the extension classifier.
        match ollama.installed_models() {
            Ok(installed) if ollama.has_model(&installed) => {
                println!(
                    "{} {}",
                    "✓".green(),
                    format!("Asking {} about each file...", ollama.model()).bold()
                );
            }

            _ => {
                println!(
                    "{} {}",
                    "×".red(),
                    "No usable local model; falling back to extensions.".yellow()
                );
                println!("  {}", "Run `reorganize ai` for details.".dimmed());
            }
        }

        println!();
    }

    let progress = ProgressBar::new(total);

    progress.set_style(
        ProgressStyle::with_template("  {spinner} {msg} [{bar:40.cyan/blue}] {pos}/{len}")
            .unwrap()
            .progress_chars("=>-"),
    );

    progress.enable_steady_tick(Duration::from_millis(100));
    progress.set_message(if ai { "Reading files..." } else { "Planning..." });

    for entry in &files {
        let from = entry.path();

        let outcome = match &ollama {
            Some(ollama) => {
                let facts = inspect::inspect(&from);
                let proposal = ai::organizer::propose(ollama, &facts);

                relocator
                    .plan_into(
                        &from,
                        rootpath,
                        &proposal.folder,
                        proposal.new_stem.as_deref(),
                    )
                    .map(|to| (to, Some(proposal)))
            }

            None => relocator.plan(&from, rootpath).map(|to| (to, None)),
        };

        match outcome {
            Ok((to, proposal)) => planned.push(PlannedMove { from, to, proposal }),
            Err(error) => failures.push((from, error)),
        }

        progress.inc(1);
    }

    progress.finish_and_clear();

    // =========================
    // Show the Plan
    // =========================

    // Renames are harder to reverse in your head than moves, so an AI run
    // always shows its plan, even when it is about to act on it.
    if dry_run || ai {
        for move_ in &planned {
            let destination = move_
                .to
                .strip_prefix(rootpath)
                .unwrap_or(&move_.to)
                .to_string_lossy()
                .into_owned();

            let renamed = move_.to.file_name() != move_.from.file_name();

            let suffix = if renamed {
                "  (renamed)".magenta().to_string()
            } else {
                String::new()
            };

            println!(
                "      {} {} {}{}",
                display_name(&move_.from).yellow(),
                "→".dimmed(),
                destination.cyan(),
                suffix
            );

            if let Some(proposal) = &move_.proposal {
                if proposal.fell_back {
                    println!(
                        "          {}",
                        format!("fell back: {}", proposal.reason).dimmed()
                    );
                } else if !proposal.reason.is_empty() {
                    println!("          {}", proposal.reason.dimmed());
                }
            }
        }

        for (path, error) in &failures {
            println!(
                "      {} {}",
                display_name(path).red(),
                format!("({error})").dimmed()
            );
        }

        println!();
    }

    if dry_run {
        println!("{}", "  -----------------------------".dimmed());
        println!(
            "  {} {}",
            "✓".green(),
            format!(
                "{} {} would be organized.",
                planned.len(),
                file_word(planned.len() as u64)
            )
            .green()
            .bold()
        );
        println!("  {}", "Dry run — nothing was moved.".yellow().bold());
        println!("{}", "  -----------------------------".dimmed());
        println!();

        return;
    }

    // =========================
    // Confirm the Plan
    // =========================

    if ai {
        let renames = planned
            .iter()
            .filter(|move_| move_.to.file_name() != move_.from.file_name())
            .count();

        let question = if renames > 0 {
            format!(
                "Apply this plan? {} {} will be renamed.",
                renames,
                file_word(renames as u64)
            )
        } else {
            "Apply this plan?".to_string()
        };

        match Confirm::new(&question).with_default(false).prompt() {
            Ok(true) => {}

            _ => {
                println!();
                println!("{} {}", "×".red(), "Nothing was moved.".yellow());

                return;
            }
        }

        println!();
    }

    if planned.is_empty() {
        println!("{} {}", "×".red(), "Nothing to move.".yellow());

        return;
    }

    // =========================
    // Execute
    // =========================

    let mut log = match undo::RunLog::create() {
        Ok(log) => log,

        Err(error) => {
            println!();
            println!("{} Could not start the run log: {}", "×".red(), error);
            println!("{}", "  Nothing was moved.".yellow());

            return;
        }
    };

    let progress = ProgressBar::new(planned.len() as u64);

    progress.set_style(
        ProgressStyle::with_template("  {spinner} {msg} [{bar:40.cyan/blue}] {pos}/{len}")
            .unwrap()
            .progress_chars("=>-"),
    );

    progress.enable_steady_tick(Duration::from_millis(100));
    progress.set_message("Organizing files...");

    let mut moved = 0u64;

    for move_ in planned {
        match relocator.perform(&move_.from, move_.to) {
            Ok(destination) => {
                moved += 1;

                if let Err(error) = log.record(&move_.from, &destination) {
                    progress.suspend(|| {
                        println!("{} Could not record a move: {}", "×".red(), error);
                    });
                }
            }

            Err(error) => failures.push((move_.from, error)),
        }

        progress.inc(1);
    }

    if failures.is_empty() {
        progress.finish_with_message("All files organized.");
    } else {
        progress.finish_with_message("Finished with some errors.");
    }

    let log_path = log.finish();

    // =========================
    // Final State
    // =========================

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
            format!("{} could not be moved:", failures.len()).red().bold()
        );

        for (path, error) in &failures {
            println!(
                "      {} {}",
                display_name(path).yellow(),
                format!("({error})").dimmed()
            );
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

/// One move decided before anything is written to disk.
struct PlannedMove {
    from: PathBuf,
    to: PathBuf,
    proposal: Option<ai::organizer::Proposal>,
}
