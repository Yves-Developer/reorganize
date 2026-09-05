use inquire::{Select, Text};

/// The menu entry that asks the user to type a path instead of picking one.
pub const CUSTOM_PATH: &str = "Custom path";

pub fn select_directory() -> Result<String, String> {
    let options = vec!["Downloads", "Documents", "Desktop", CUSTOM_PATH];
    let selection = Select::new("What do you want to organize?", options).prompt();

    match selection {
        Ok(directory) => Ok(directory.to_string()),
        Err(_) => Err("Selection cancelled.".to_string()),
    }
}

pub fn prompt_custom_path() -> Result<String, String> {
    let entered = Text::new("Enter the full path of the folder to organize:").prompt();

    match entered {
        Ok(path) => {
            let path = path.trim().trim_matches('"').to_string();

            if path.is_empty() {
                return Err("No path entered.".to_string());
            }

            Ok(path)
        }

        Err(_) => Err("Custom path cancelled.".to_string()),
    }
}
