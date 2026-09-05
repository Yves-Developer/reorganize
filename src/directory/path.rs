// path
use std::env;
pub fn get_directory_path(directory: &str) -> String {
    let home_path = env::var("USERPROFILE").expect("Could not find home directory");

    format!("{home_path}\\{directory}")
}
