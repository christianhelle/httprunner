use crate::types::HttpResult;
use crate::parser::parse_http_file;
use crate::runner::execute_http_request;
use crate::log::Log;
use anyhow::Result;
use colored::*;

pub async fn process_http_files(
    files: &[String], 
    verbose: bool, 
    log_filename: Option<&str>
) -> Result<()> {
    let mut log = Log::new(log_filename)?;
    
    let mut total_success_count = 0u32;
    let mut total_request_count = 0u32;
    let mut files_processed = 0u32;
    
    for http_file in files {
        log.writeln(&format!("{} HTTP File Runner - Processing file: {}{}", 
                            "🚀".blue(), http_file, "".normal()));
        log.writeln(&"=".repeat(50));
        
        let requests = match parse_http_file(http_file) {
            Ok(requests) => requests,
            Err(e) => {
                if e.to_string().contains("No such file") {
                    log.writeln(&format!("{} Error: File '{}' not found{}", 
                                        "❌".red(), http_file, "".normal()));
                } else {
                    log.writeln(&format!("{} Error parsing file: {}{}", 
                                        "❌".red(), e, "".normal()));
                }
                continue;
            }
        };
        
        if requests.is_empty() {
            log.writeln(&format!("{} No HTTP requests found in file{}", 
                                "⚠️".yellow(), "".normal()));
            continue;
        }
        
        let mut success_count = 0u32;
        
        for request in &requests {
            let result = match execute_http_request(request, verbose).await {
                Ok(result) => result,
                Err(e) => {
                    log.writeln(&format!("{} {} {} - Error: {}{}", 
                                        "❌".red(), request.method, request.url, e, "".normal()));
                    continue;
                }
            };
            
            if result.success {
                success_count += 1;
                log.writeln(&format!("{} {} {} - Status: {} - {}ms{}", 
                                    "✅".green(), request.method, request.url, 
                                    result.status_code, result.duration_ms, "".normal()));
            } else {
                if let Some(ref msg) = result.error_message {
                    log.writeln(&format!("{} {} {} - Status: {} - {}ms - Error: {}{}", 
                                        "❌".red(), request.method, request.url, 
                                        result.status_code, result.duration_ms, msg, "".normal()));
                } else {
                    log.writeln(&format!("{} {} {} - Status: {} - {}ms{}", 
                                        "❌".red(), request.method, request.url, 
                                        result.status_code, result.duration_ms, "".normal()));
                }
            }
            
            if verbose {
                print_verbose_response(&mut log, &result);
            }
        }
        
        total_success_count += success_count;
        total_request_count += requests.len() as u32;
        files_processed += 1;
        
        log.writeln("");
        log.writeln(&"=".repeat(50));
        log.writeln(&format!("File Summary: {}/{} requests succeeded", 
                            success_count, requests.len()));
        log.writeln("");
    }
    
    if files_processed > 1 {
        log.writeln(&format!("{} Overall Summary:", "🎯".bright_blue()));
        log.writeln(&format!("Files processed: {}", files_processed));
        log.writeln(&format!("Total requests: {}/{}", total_success_count, total_request_count));
    }
    
    Ok(())
}

fn print_verbose_response(log: &mut Log, result: &HttpResult) {
    if let Some(ref headers) = result.response_headers {
        log.writeln(&format!("  {}Response Headers:{}", "📋".cyan(), "".normal()));
        for header in headers {
            log.writeln(&format!("    {}: {}", header.name, header.value));
        }
    }
    
    if let Some(ref body) = result.response_body {
        log.writeln(&format!("  {}Response Body:{}", "📄".cyan(), "".normal()));
        // Limit body output to first 500 characters to avoid overwhelming output
        if body.len() > 500 {
            log.writeln(&format!("    {}...", &body[..500]));
            log.writeln(&format!("    {}(truncated - {} total characters){}", 
                                "".dimmed(), body.len(), "".normal()));
        } else {
            log.writeln(&format!("    {}", body));
        }
    }
    log.writeln("");
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[tokio::test]
    #[ignore] // Skip network tests in CI/sandbox environments
    async fn test_process_http_files() {
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path();
        
        // Create a test .http file
        let test_file = temp_path.join("test.http");
        fs::write(&test_file, "GET https://httpbin.org/status/200").unwrap();
        
        let files = vec![test_file.to_string_lossy().to_string()];
        let result = process_http_files(&files, false, None).await;
        
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_process_nonexistent_file() {
        let files = vec!["nonexistent.http".to_string()];
        let result = process_http_files(&files, false, None).await;
        
        // Should not error out, but continue processing
        assert!(result.is_ok());
    }
}