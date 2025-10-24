mod assertions;
mod cli;
mod colors;
mod discovery;
mod environment;
mod log;
mod parser;
mod processor;
mod request_variables;
mod runner;
mod types;
mod upgrade;

use clap::Parser;

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
        eprintln!("{} No .http files specified", colors::red("❌"));
        eprintln!("Use --help for usage information");
        std::process::exit(1);
    } else {
        cli_args.files.clone()
    };

    let result = processor::process_http_files(
        &files,
        cli_args.verbose,
        cli_args.get_log_filename().as_deref(),
        cli_args.env.as_deref(),
    )?;

    if result {
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

    cli::show_donation_banner();

    if !result {
        std::process::exit(1);
    }

    Ok(())
}
