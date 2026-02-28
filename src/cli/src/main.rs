mod cli;
mod shutdown;
mod upgrade;

use crate::cli::ReportFormat;
use anyhow::Result;
use clap::{CommandFactory, Parser};
use httprunner_core::report::{generate_html, generate_markdown};
use httprunner_core::telemetry::{self, AppType, CliArgPatterns};
use httprunner_core::types::ProcessorResults;
use httprunner_core::{colors, discovery, export, logging, processor};

const VERSION: &str = env!("VERSION");
const INSTRUMENTATION_KEY: &str = "a7a07a35-4869-4fa2-b852-03f44b35f418";

fn main() -> anyhow::Result<()> {
    shutdown::enable_forceful_shutdown();

    let cli_args = cli::Cli::parse();
    if cli_args.files.is_empty() && !cli_args.discover && !cli_args.upgrade {
        let mut cmd = cli::Cli::command();
        cmd.print_help()?;
        return Ok(());
    }

    telemetry::init_without_persisted_state(
        AppType::CLI,
        VERSION,
        cli_args.no_telemetry,
        INSTRUMENTATION_KEY,
    );
    track_cli_usage(&cli_args);

    if cli_args.upgrade {
        let result = upgrade::run_upgrade();
        telemetry::flush();
        return result;
    }

    let result = run(&cli_args);

    if let Err(ref e) = result {
        telemetry::track_error(e.as_ref());
    }

    telemetry::flush();

    if result.is_err() {
        std::process::exit(1);
    }

    Ok(())
}

fn track_cli_usage(cli_args: &cli::Cli) {
    let patterns = CliArgPatterns {
        verbose: cli_args.verbose,
        log: cli_args.log.is_some(),
        env: cli_args.env.is_some(),
        insecure: cli_args.insecure,
        discover: cli_args.discover,
        no_banner: cli_args.no_banner,
        pretty_json: cli_args.pretty_json,
        report: cli_args.report.is_some(),
        report_format: cli_args.report.map(|f| format!("{:?}", f)),
        export: cli_args.export,
        export_json: cli_args.export_json,
        file_count: cli_args.files.len(),
        delay: cli_args.delay,
    };
    telemetry::track_cli_args(&patterns);
}

fn run(cli_args: &cli::Cli) -> Result<()> {
    let files = load_files(cli_args)?;
    if files.is_empty() {
        return Ok(());
    }
    let results = process_http_files(cli_args, files)?;
    generate_report(cli_args, &results)?;
    export_results(cli_args, &results)?;
    export_json_results(cli_args, &results)?;
    show_support_key(cli_args);
    if !cli_args.no_banner {
        cli::show_donation_banner();
    }
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
        return Ok(Vec::<String>::new());
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
        cli_args.delay,
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

fn export_json_results(cli_args: &cli::Cli, results: &ProcessorResults) -> Result<()> {
    if !cli_args.export_json {
        return Ok(());
    }

    match export::export_json(results) {
        Ok(filename) => {
            println!(
                "{} Exported JSON results to {}",
                colors::green("✅"),
                filename
            );
        }
        Err(e) => {
            eprintln!(
                "{} Failed to export JSON results: {}",
                colors::red("❌"),
                e
            );
            anyhow::bail!("Failed to export JSON results: {}", e);
        }
    }
    Ok(())
}

fn show_support_key(cli_args: &cli::Cli) {
    if cli_args.no_telemetry {
        return;
    }
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
