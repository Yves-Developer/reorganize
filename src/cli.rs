// Argument parsing.
//
// Hand-rolled rather than pulling in a parser crate: there are three options
// and no subcommand tree yet. If the AI work adds `learn`, `--explain` and
// friends, this is the seam to swap for clap.

pub enum Command {
    Organize { dry_run: bool },
    Undo,
    Help,
}

pub const USAGE: &str = "Usage:
  reorganize              Organize a folder
  reorganize --dry-run    Show what would move, without moving anything
  reorganize undo         Reverse the most recent run

Options:
  -n, --dry-run           Preview only
  -h, --help              Show this message";

pub fn parse_args<I>(args: I) -> Result<Command, String>
where
    I: IntoIterator<Item = String>,
{
    let mut dry_run = false;
    let mut undo = false;

    for arg in args {
        match arg.as_str() {
            "-h" | "--help" => return Ok(Command::Help),
            "-n" | "--dry-run" => dry_run = true,
            "undo" => undo = true,
            other => return Err(format!("Unknown argument: {other}")),
        }
    }

    if undo && dry_run {
        return Err("undo cannot be combined with --dry-run.".to_string());
    }

    if undo {
        return Ok(Command::Undo);
    }

    Ok(Command::Organize { dry_run })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(args: &[&str]) -> Result<Command, String> {
        parse_args(args.iter().map(|arg| arg.to_string()))
    }

    #[test]
    fn no_arguments_organizes_for_real() {
        assert!(matches!(
            parse(&[]),
            Ok(Command::Organize { dry_run: false })
        ));
    }

    #[test]
    fn dry_run_is_recognised_in_both_forms() {
        assert!(matches!(
            parse(&["--dry-run"]),
            Ok(Command::Organize { dry_run: true })
        ));
        assert!(matches!(
            parse(&["-n"]),
            Ok(Command::Organize { dry_run: true })
        ));
    }

    #[test]
    fn undo_is_recognised() {
        assert!(matches!(parse(&["undo"]), Ok(Command::Undo)));
    }

    #[test]
    fn help_wins_over_other_arguments() {
        assert!(matches!(parse(&["--dry-run", "--help"]), Ok(Command::Help)));
    }

    // Silently ignoring a typo like `--dry-runn` would move files for real
    // when the user asked for a preview.
    #[test]
    fn unknown_arguments_are_rejected() {
        assert!(parse(&["--dry-runn"]).is_err());
        assert!(parse(&["--force"]).is_err());
    }

    #[test]
    fn undo_and_dry_run_together_are_rejected() {
        assert!(parse(&["undo", "--dry-run"]).is_err());
    }
}
