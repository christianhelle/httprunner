use crate::colors;
use clap::Parser;

const LONG_VERSION: &str = concat!(
    env!("VERSION"),
    "\nGit tag: ",
    env!("GIT_TAG"),
    "\nGit commit: ",
    env!("GIT_COMMIT"),
    "\nBuild date: ",
    env!("BUILD_DATE")
);

#[derive(Parser)]
#[command(name = "httprunner")]
#[command(about = "HTTP File Runner - Execute HTTP requests from .http files", long_about = None)]
#[command(version = env!("VERSION"))]
#[command(long_version = LONG_VERSION)]
pub struct Cli {
    /// One or more .http files to process
    #[arg(value_name = "FILE")]
    pub files: Vec<String>,

    /// Show detailed HTTP request and response information
    #[arg(short, long)]
    pub verbose: bool,

    /// Log output to a file (defaults to 'log' if no filename is specified)
    #[arg(long, value_name = "FILENAME")]
    pub log: Option<Option<String>>,

    /// Specify environment name to load variables from http-client.env.json
    #[arg(long, value_name = "ENVIRONMENT")]
    pub env: Option<String>,

    /// Recursively discover and process all .http files from current directory
    #[arg(long)]
    pub discover: bool,

    /// Update httprunner to the latest version
    #[arg(long)]
    pub upgrade: bool,
}

impl Cli {
    pub fn get_log_filename(&self) -> Option<String> {
        match &self.log {
            Some(Some(filename)) => Some(filename.clone()),
            Some(None) => Some("log".to_string()),
            None => None,
        }
    }
}

#[allow(dead_code)]
pub fn show_version() {
    println!(
        "{} HTTP File Runner {} version {}",
        colors::blue(""),
        "",
        colors::green(env!("VERSION"))
    );
    println!("Git tag: {}{}", colors::yellow(""), env!("GIT_TAG"));
    println!("Git commit: {}{}", colors::yellow(""), env!("GIT_COMMIT"));
    println!("Build date: {}{}", colors::yellow(""), env!("BUILD_DATE"));
}

pub fn show_donation_banner() {
    println!("\n╭──────────────────────────────💝 Support────────────────────────────────╮");
    println!("│ 💖 Enjoying httprunner? Consider supporting the project!               │");
    println!("│                                                                        │");
    println!("│ 🎯 Sponsor: https://github.com/sponsors/christianhelle                 │");
    println!("│ ☕ Buy me a coffee: https://www.buymeacoffee.com/christianhelle        │");
    println!("│                                                                        │");
    println!("│ 🐛 Found an issue? https://github.com/christianhelle/httprunner/issues │");
    println!("╰────────────────────────────────────────────────────────────────────────╯");
}
