use clap::Parser;

const LONG_VERSION: &str = concat!(
    env!("VERSION"),
    "\ngit tag: ",
    env!("GIT_TAG"),
    "\ngit commit: ",
    env!("GIT_COMMIT"),
    "\nbuild date: ",
    env!("BUILD_DATE")
);

#[derive(Parser)]
#[command(name = "httprunner")]
#[command(about = "HTTP File Runner - Execute HTTP requests from .http files", long_about = None)]
#[command(version = LONG_VERSION)]
#[command(long_version = LONG_VERSION)]
pub struct Cli {
    /// One or more .http files to process
    #[arg(
        value_name = "FILE",
        num_args = 0..,
        conflicts_with_all = ["discover", "upgrade"]
    )]
    pub files: Vec<String>,

    /// Show detailed HTTP request and response information
    #[arg(short, long)]
    pub verbose: bool,
    /// Log output to a file (defaults to 'log' if no filename is specified)
    #[arg(long, value_name = "FILENAME", num_args = 0..=1)]
    pub log: Option<Option<String>>,

    /// Specify environment name to load variables from http-client.env.json
    #[arg(long, value_name = "ENVIRONMENT")]
    pub env: Option<String>,

    /// Allow insecure HTTPS connections (accept invalid certificates and hostnames)
    #[arg(long)]
    pub insecure: bool,

    /// Recursively discover and process all .http files from current directory
    #[arg(long)]
    pub discover: bool,

    /// Update httprunner to the latest version
    #[arg(long)]
    pub upgrade: bool,

    /// Do not show the donation banner
    #[arg(long)]
    pub no_banner: bool,

    /// Pretty-print JSON payloads in verbose output
    #[arg(long)]
    pub pretty_json: bool,
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

#[cfg(test)]
mod tests {
    use super::*;

    fn cli_with_log(log: Option<Option<&str>>) -> Cli {
        Cli {
            files: vec![],
            verbose: false,
            log: log.map(|opt| opt.map(|s| s.to_string())),
            env: None,
            insecure: false,
            discover: false,
            upgrade: false,
            no_banner: false,
            pretty_json: false,
        }
    }

    #[test]
    fn get_log_filename_returns_explicit_value() {
        let cli = cli_with_log(Some(Some("custom")));
        assert_eq!(cli.get_log_filename().as_deref(), Some("custom"));
    }

    #[test]
    fn get_log_filename_defaults_to_log_name() {
        let cli = cli_with_log(Some(None));
        assert_eq!(cli.get_log_filename().as_deref(), Some("log"));
    }

    #[test]
    fn get_log_filename_none_when_flag_missing() {
        let cli = cli_with_log(None);
        assert!(cli.get_log_filename().is_none());
    }
}
