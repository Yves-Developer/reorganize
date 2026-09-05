use colored::Colorize;
use inquire::Select;

pub fn select_directory() -> Result<String, String> {
    let options = vec!["Downloads", "Documents", "Desktop", "Custom path"];
    let selection = Select::new("What do you want to organize?", options).prompt();

    match selection {
        Ok(directory) => Ok(directory.to_string()),
        Err(_) => {
            println!();
            println!("{} {}", "×".red(), "Selection cancelled.".red());

            Err("Selection cancelled.".to_string())
        }
    }
}
