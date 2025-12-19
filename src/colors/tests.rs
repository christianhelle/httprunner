use super::*;
use colored::Colorize;

#[test]
fn color_helpers_match_colored_output() {
    assert_eq!(red("hello"), "hello".red().to_string());
    assert_eq!(green("world"), "world".green().to_string());
    assert_eq!(yellow("warn"), "warn".yellow().to_string());
    assert_eq!(blue("info"), "info".blue().to_string());
}
