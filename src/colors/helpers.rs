use colored::Colorize;

pub fn red(text: &str) -> String {
    text.red().to_string()
}

pub fn green(text: &str) -> String {
    text.green().to_string()
}

pub fn yellow(text: &str) -> String {
    text.yellow().to_string()
}

pub fn blue(text: &str) -> String {
    text.blue().to_string()
}
