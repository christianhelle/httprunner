use crate::types::{AssertionType, ProcessorResults};
use chrono::Local;
use std::fs;
use std::io::Write;

fn escape_markdown(s: &str) -> String {
    s.replace('|', "\\|")
}

pub fn generate_markdown(results: ProcessorResults) -> Result<String, std::io::Error> {
    let timestamp = Local::now().format("%Y%m%d-%H%M%S");
    let filename = format!("report-{}.md", timestamp);
    
    let mut report = String::new();
    
    // Header
    report.push_str("# HTTP File Runner - Test Report\n\n");
    report.push_str(&format!("**Generated:** {}\n\n", Local::now().format("%Y-%m-%d %H:%M:%S")));
    
    // Overall summary
    let total_success: u32 = results.files.iter().map(|f| f.success_count).sum();
    let total_failed: u32 = results.files.iter().map(|f| f.failed_count).sum();
    let total_skipped: u32 = results.files.iter().map(|f| f.skipepd_count).sum();
    let total_requests = total_success + total_failed + total_skipped;
    
    report.push_str("## Overall Summary\n\n");
    report.push_str(&format!("- **Total Requests:** {}\n", total_requests));
    report.push_str(&format!("- **Passed:** ✅ {}\n", total_success));
    report.push_str(&format!("- **Failed:** ❌ {}\n", total_failed));
    report.push_str(&format!("- **Skipped:** ⏭️ {}\n", total_skipped));
    report.push_str(&format!("- **Success Rate:** {:.1}%\n\n", 
        if total_requests > 0 { (total_success as f64 / total_requests as f64) * 100.0 } else { 0.0 }));
    
    // Process each file
    for file_results in &results.files {
        report.push_str("---\n\n");
        report.push_str(&format!("## File: `{}`\n\n", escape_markdown(&file_results.filename)));
        
        report.push_str(&format!("- **Passed:** {} | **Failed:** {} | **Skipped:** {}\n\n",
            file_results.success_count, 
            file_results.failed_count, 
            file_results.skipepd_count));
        
        // Process each request
        for context in &file_results.result_contexts {
            let request_name = context.name.clone();
            report.push_str(&format!("### Request: {}\n\n", escape_markdown(&request_name)));
            
            // Request details
            report.push_str("#### Request Details\n\n");
            report.push_str(&format!("- **Method:** `{}`\n", context.request.method));
            report.push_str(&format!("- **URL:** `{}`\n", escape_markdown(&context.request.url)));
            
            if let Some(timeout) = context.request.timeout {
                report.push_str(&format!("- **Timeout:** {}ms\n", timeout));
            }
            if let Some(conn_timeout) = context.request.connection_timeout {
                report.push_str(&format!("- **Connection Timeout:** {}ms\n", conn_timeout));
            }
            if let Some(ref depends_on) = context.request.depends_on {
                report.push_str(&format!("- **Depends On:** `{}`\n", escape_markdown(depends_on)));
            }
            
            // Headers
            if !context.request.headers.is_empty() {
                report.push_str("\n**Headers:**\n\n");
                report.push_str("| Header | Value |\n");
                report.push_str("|--------|-------|\n");
                for header in &context.request.headers {
                    report.push_str(&format!("| {} | {} |\n", 
                        escape_markdown(&header.name), 
                        escape_markdown(&header.value)));
                }
                report.push_str("\n");
            }
            
            // Request body
            if let Some(ref body) = context.request.body {
                report.push_str("**Request Body:**\n\n");
                report.push_str("```\n");
                report.push_str(body);
                report.push_str("\n```\n\n");
            }
            
            // Conditions
            if !context.request.conditions.is_empty() {
                report.push_str("**Conditions:**\n\n");
                for condition in &context.request.conditions {
                    let directive = if condition.negate { "@if-not" } else { "@if" };
                    let cond_type = format!("{:?}", condition.condition_type);
                    report.push_str(&format!("- {} `{}.response.{}` == `{}`\n", 
                        directive,
                        escape_markdown(&condition.request_name),
                        cond_type,
                        escape_markdown(&condition.expected_value)));
                }
                report.push_str("\n");
            }
            
            // Result details
            if let Some(ref result) = context.result {
                report.push_str("#### Response Details\n\n");
                
                let status_icon = if result.success { "✅" } else { "❌" };
                report.push_str(&format!("- **Status:** {} {}\n", status_icon, result.status_code));
                report.push_str(&format!("- **Duration:** {}ms\n", result.duration_ms));
                
                if let Some(ref error_msg) = result.error_message {
                    report.push_str(&format!("- **Error:** {}\n", escape_markdown(error_msg)));
                }
                
                // Response headers
                if let Some(ref headers) = result.response_headers {
                    if !headers.is_empty() {
                        report.push_str("\n**Response Headers:**\n\n");
                        report.push_str("| Header | Value |\n");
                        report.push_str("|--------|-------|\n");
                        for (name, value) in headers {
                            report.push_str(&format!("| {} | {} |\n", 
                                escape_markdown(name), 
                                escape_markdown(value)));
                        }
                        report.push_str("\n");
                    }
                }
                
                // Response body
                if let Some(ref body) = result.response_body {
                    report.push_str("**Response Body:**\n\n");
                    report.push_str("```\n");
                    report.push_str(body);
                    report.push_str("\n```\n\n");
                }
                
                // Assertions
                if !result.assertion_results.is_empty() {
                    report.push_str("#### Assertion Results\n\n");
                    report.push_str("| Type | Expected | Actual | Result |\n");
                    report.push_str("|------|----------|--------|--------|\n");
                    
                    for assertion_result in &result.assertion_results {
                        let assertion_type_str = match assertion_result.assertion.assertion_type {
                            AssertionType::Status => "Status Code",
                            AssertionType::Body => "Response Body",
                            AssertionType::Headers => "Response Headers",
                        };
                        
                        let result_icon = if assertion_result.passed { "✅" } else { "❌" };
                        let actual_val = assertion_result.actual_value
                            .as_ref()
                            .map(|v| escape_markdown(v))
                            .unwrap_or_else(|| "N/A".to_string());
                        
                        report.push_str(&format!("| {} | {} | {} | {} |\n",
                            assertion_type_str,
                            escape_markdown(&assertion_result.assertion.expected_value),
                            actual_val,
                            result_icon));
                    }
                    report.push_str("\n");
                }
            } else {
                report.push_str("#### Response Details\n\n");
                report.push_str("⏭️ **Request was skipped**\n\n");
            }
        }
    }
    
    // Write to file
    let mut file = fs::File::create(&filename)?;
    file.write_all(report.as_bytes())?;
    
    Ok(filename)
}
