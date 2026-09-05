// Argument parsing.
//
// Hand-rolled rather than pulling in a parser crate: there are three options
// and no subcommand tree yet. If the AI work adds `learn`, `--explain` and
// friends, this is the seam to swap for clap.

pub enum Command {
    Organize {
        dry_run: bool,
        ai: bool,
        path: Option<String>,
    },
    Undo,
    AiStatus,
    Help,
}

pub const USAGE: &str = "Usage:
  reorganize              Organize a folder (pick one interactively)
  reorganize <folder>     Organize that folder directly, at any depth
  reorganize --dry-run    Show what would move, without moving anything
  reorganize undo         Reverse the most recent run
  reorganize ai           Check whether a local model is available

Options:
      --ai                Let a local model choose folders and file names
  -n, --dry-run           Preview only
  -h, --help              Show this message";

pub fn parse_args<I>(args: I) -> Result<Command, String>
where
    I: IntoIterator<Item = String>,
{
    let mut dry_run = false;
    let mut undo = false;
    let mut ai_status = false;
    let mut ai_mode = false;
    let mut path: Option<String> = None;

    for arg in args {
        match arg.as_str() {
            "-h" | "--help" => return Ok(Command::Help),
            "-n" | "--dry-run" => dry_run = true,
            "undo" => undo = true,
            "ai" => ai_status = true,
            "--ai" => ai_mode = true,
            // A leading dash is always a flag, so an unknown one is a typo
            // rather than a folder called "--dry-runn".
            other if other.starts_with('-') => {
                return Err(format!("Unknown option: {other}"));
            }

            other => {
                if path.is_some() {
                    return Err("Only one folder can be given.".to_string());
                }

                path = Some(other.to_string());
            }
        }
    }

    if path.is_some() && (undo || ai_status) {
        return Err("a folder can only be given when organizing.".to_string());
    }

    if ai_mode && (undo || ai_status) {
        return Err("--ai only applies when organizing.".to_string());
    }

    if undo && ai_status {
        return Err("undo and ai cannot be combined.".to_string());
    }

    if (undo || ai_status) && dry_run {
        return Err("--dry-run only applies when organizing.".to_string());
    }

    if ai_status {
        return Ok(Command::AiStatus);
    }

    if undo {
        return Ok(Command::Undo);
    }

    Ok(Command::Organize {
        dry_run,
        ai: ai_mode,
        path,
    })
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
            Ok(Command::Organize {
                dry_run: false,
                ai: false,
                path: None
            })
        ));
    }

    #[test]
    fn dry_run_is_recognised_in_both_forms() {
        assert!(matches!(
            parse(&["--dry-run"]),
            Ok(Command::Organize {
                dry_run: true,
                ai: false,
                path: None
            })
        ));
        assert!(matches!(
            parse(&["-n"]),
            Ok(Command::Organize {
                dry_run: true,
                ai: false,
                path: None
            })
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

    #[test]
    fn ai_mode_is_a_flag_not_a_subcommand() {
        assert!(matches!(
            parse(&["--ai"]),
            Ok(Command::Organize {
                dry_run: false,
                ai: true,
                path: None
            })
        ));
        assert!(matches!(
            parse(&["--ai", "--dry-run"]),
            Ok(Command::Organize {
                dry_run: true,
                ai: true,
                path: None
            })
        ));
    }

    #[test]
    fn ai_flag_and_ai_subcommand_do_not_mix() {
        assert!(parse(&["--ai", "ai"]).is_err());
        assert!(parse(&["--ai", "undo"]).is_err());
    }

    #[test]
    fn ai_status_is_recognised() {
        assert!(matches!(parse(&["ai"]), Ok(Command::AiStatus)));
    }

    #[test]
    fn ai_rejects_meaningless_combinations() {
        assert!(parse(&["ai", "--dry-run"]).is_err());
        assert!(parse(&["ai", "undo"]).is_err());
    }
}
