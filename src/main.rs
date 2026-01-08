mod assertions;
mod cli;
mod colors;
mod conditions;
mod discovery;
mod environment;
mod error;
mod functions;
mod logging;
mod parser;
mod processor;
mod report;
mod runner;
mod types;
mod upgrade;
mod variables;

use clap::{CommandFactory, Parser};

use crate::report::generate_markdown;

fn main() -> error::Result<()> {
    let cli_args = cli::Cli::parse();

    if cli_args.upgrade {
        return upgrade::run_upgrade();
    }

    let files = if cli_args.discover {
        let discovered = discovery::run_discovery_mode()?;
        if discovered.is_empty() {
            return Ok(());
        }
        discovered
    } else if cli_args.files.is_empty() {
        // Show help when no arguments are provided
        let mut cmd = cli::Cli::command();
        cmd.print_help()?;
        std::process::exit(0);
    } else {
        cli_args.files.clone()
    };

    let results = processor::process_http_files(
        &files,
        cli_args.verbose,
        cli_args.get_log_filename().as_deref(),
        cli_args.env.as_deref(),
        cli_args.insecure,
        cli_args.pretty_json,
    )?;

    if results.success {
        println!(
            "{} All discovered files processed successfully",
            colors::green("✅")
        );
    } else {
        println!(
            "{} Some discovered files failed to process",
            colors::red("❌")
        );
    }

    if !cli_args.no_banner {
        cli::show_donation_banner();
    }

    if cli_args.report {
        match generate_markdown(&results) {
            Ok(filename) => println!("{} Report generated: {}", colors::green("✅"), filename),
            Err(e) => {
                eprintln!("{} Failed to generate report: {}", colors::red("❌"), e);
                std::process::exit(2);
            }
        }
    }

    if !results.success {
        std::process::exit(1);
    }

    Ok(())
}
