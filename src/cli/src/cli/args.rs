use clap::{Parser, ValueEnum};

const LONG_VERSION: &str = concat!(
    env!("VERSION"),
    "\ngit tag: ",
    env!("GIT_TAG"),
    "\ngit commit: ",
    env!("GIT_COMMIT"),
    "\nbuild date: ",
    env!("BUILD_DATE")
);

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum ReportFormat {
    Markdown,
    Html,
}

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

    /// Generate summary report (default=markdown)
    #[arg(long, value_name = "FORMAT", num_args = 0..=1, default_missing_value = "markdown")]
    pub report: Option<ReportFormat>,

    /// Export requests and responses to files
    #[arg(long)]
    pub export: bool,

    /// Disable anonymous telemetry data collection
    #[arg(long)]
    pub no_telemetry: bool,

    /// Delay between requests in milliseconds (default: 0)
    #[arg(long, value_name = "MILLISECONDS", default_value = "0")]
    pub delay: u64,
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
