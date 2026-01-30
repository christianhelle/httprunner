mod cli;
mod upgrade;

use anyhow::Result;
use clap::{CommandFactory, Parser};
use httprunner_lib::types::ProcessorResults;
use httprunner_lib::{colors, discovery, export, logging, processor, report};

use crate::cli::ReportFormat;
use crate::report::{generate_html, generate_markdown};

fn main() -> anyhow::Result<()> {
    let cli_args = cli::Cli::parse();
    if cli_args.upgrade {
        return upgrade::run_upgrade();
    }

    let result = run(&cli_args);

    show_support_key();
    if !cli_args.no_banner {
        cli::show_donation_banner();
    }

    if result.is_err() {
        std::process::exit(1);
    }

    Ok(())
}

fn run(cli_args: &cli::Cli) -> Result<()> {
    let files = load_files(cli_args)?;
    let results = process_http_files(cli_args, files)?;
    generate_report(cli_args, &results)?;
    export_results(cli_args, &results)?;
    Ok(())
}

fn load_files(cli_args: &cli::Cli) -> Result<Vec<String>> {
    let files = if cli_args.discover {
        let discovered = discovery::run_discovery_mode()?;
        if discovered.is_empty() {
            return Ok(Vec::<String>::new());
        }
        discovered
    } else if cli_args.files.is_empty() {
        let mut cmd = cli::Cli::command();
        cmd.print_help()?;
        std::process::exit(0);
    } else {
        cli_args.files.clone()
    };
    Ok(files)
}

fn process_http_files(cli_args: &cli::Cli, files: Vec<String>) -> Result<ProcessorResults> {
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
    Ok(results)
}

fn generate_report(cli_args: &cli::Cli, results: &ProcessorResults) -> Result<()> {
    if let Some(format) = cli_args.report {
        let result = match format {
            ReportFormat::Markdown => generate_markdown(results),
            ReportFormat::Html => generate_html(results),
        };

        match result {
            Ok(filename) => println!("{} Report generated: {}", colors::green("✅"), filename),
            Err(e) => {
                eprintln!("{} Failed to generate report: {}", colors::red("❌"), e);
                anyhow::bail!("Failed to generate report: {}", e);
            }
        }
    }
    Ok(())
}

fn export_results(cli_args: &cli::Cli, results: &ProcessorResults) -> Result<()> {
    if !cli_args.export {
        return Ok(());
    }

    match export::export_results(results, cli_args.pretty_json) {
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
            anyhow::bail!("Failed to export results: {}", e);
        }
    }
    Ok(())
}

fn show_support_key() {
    match logging::get_support_key() {
        Ok(support_key) => {
            println!(
                "\n{} Support Key: {} (use this key when seeking support)",
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
}
