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

#[cfg(test)]
mod tests {
    use super::*;
    use colored::Colorize;

    #[test]
    fn color_helpers_match_colored_output() {
        assert_eq!(red("hello"), "hello".red().to_string());
        assert_eq!(green("world"), "world".green().to_string());
        assert_eq!(yellow("warn"), "warn".yellow().to_string());
        assert_eq!(blue("info"), "info".blue().to_string());
    }
}
