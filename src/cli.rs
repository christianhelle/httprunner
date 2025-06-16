use clap::{Arg, ArgMatches, Command};
use anyhow::Result;

#[derive(Debug, Clone)]
pub struct CliOptions {
    pub discover_mode: bool,
    pub verbose: bool,
    pub log_file: Option<String>,
    pub files: Vec<String>,
}

impl CliOptions {
    pub fn parse() -> Result<Self> {
        let matches = Command::new("httprunner")
            .version("0.1.0")
            .about("A command-line tool for executing HTTP requests from .http files")
            .arg(
                Arg::new("files")
                    .help("One or more .http files to process")
                    .value_name("HTTP_FILE")
                    .action(clap::ArgAction::Append)
                    .required_unless_present("discover")
            )
            .arg(
                Arg::new("discover")
                    .long("discover")
                    .help("Recursively discover and process all .http files from current directory")
                    .action(clap::ArgAction::SetTrue)
            )
            .arg(
                Arg::new("verbose")
                    .long("verbose")
                    .help("Show detailed HTTP request and response information")
                    .action(clap::ArgAction::SetTrue)
            )
            .arg(
                Arg::new("log")
                    .long("log")
                    .help("Log output to file with optional filename")
                    .value_name("FILENAME")
                    .action(clap::ArgAction::Set)
                    .num_args(0..=1)
                    .default_missing_value("log")
            )
            .get_matches();

        Ok(Self::from_matches(&matches))
    }

    fn from_matches(matches: &ArgMatches) -> Self {
        let discover_mode = matches.get_flag("discover");
        let verbose = matches.get_flag("verbose");
        let log_file = matches.get_one::<String>("log").cloned();
        
        let files = if discover_mode {
            Vec::new()
        } else {
            matches.get_many::<String>("files")
                .unwrap_or_default()
                .cloned()
                .collect()
        };

        Self {
            discover_mode,
            verbose,
            log_file,
            files,
        }
    }
}