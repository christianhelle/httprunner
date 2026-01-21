mod cli;
mod upgrade;

use clap::{CommandFactory, Parser};
use httprunner_lib::{colors, discovery, processor, report};

use crate::cli::ReportFormat;
use crate::report::{generate_html, generate_markdown};

fn main() -> anyhow::Result<()> {
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

    if let Some(format) = cli_args.report {
        let result = match format {
            ReportFormat::Markdown => generate_markdown(&results),
            ReportFormat::Html => generate_html(&results),
        };

        match result {
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
