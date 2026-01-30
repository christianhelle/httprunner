mod cli;
mod upgrade;

use clap::{CommandFactory, Parser};
use httprunner_lib::{colors, discovery, export, logging, processor, report};

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

    if cli_args.export {
        match export::export_results(&results, cli_args.pretty_json) {
            Ok(export_results) => {
                println!(
                    "{} Exported requests and responses to files",
                    colors::green("✅")
                );
                for filename in export_results.file_names {
                    println!("   {} Exported {}", colors::green("✅"), filename);
                }
                for filename in export_results.failed_file_names {
                    println!("   {} Failed to export {}", colors::red("❌"), filename);
                }
            }
            Err(e) => {
                eprintln!("{} Failed to export results: {}", colors::red("❌"), e);
                std::process::exit(3);
            }
        }
    }

    match logging::get_support_key() {
        Ok(support_key) => {
            println!(
                "{} Support Key: {} (use this key when seeking support)",
                colors::blue("ℹ️"),
                support_key.short_key
            );
        }
        Err(e) => {
            eprintln!(
                "{} Failed to get or generate support key: {}",
                colors::red("❌"),
                e
            );
        }
    }

    if !cli_args.no_banner {
        cli::show_donation_banner();
    }

    if !results.success {
        std::process::exit(1);
    }

    Ok(())
}
