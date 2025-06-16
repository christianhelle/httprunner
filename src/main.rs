mod types;
mod cli;
mod parser;
mod runner;
mod discovery;
mod log;
mod processor;

use anyhow::Result;
use cli::CliOptions;
use discovery::run_discovery_mode;
use processor::process_http_files;

#[tokio::main]
async fn main() -> Result<()> {
    let options = CliOptions::parse()?;
    
    if options.discover_mode {
        let discovered_files = run_discovery_mode()?;
        if !discovered_files.is_empty() {
            process_http_files(
                &discovered_files, 
                options.verbose, 
                options.log_file.as_deref()
            ).await?;
        }
    } else {
        process_http_files(
            &options.files, 
            options.verbose, 
            options.log_file.as_deref()
        ).await?;
    }
    
    Ok(())
}