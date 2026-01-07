use super::formatter::escape_markdown;
use super::writer::write_report;
use crate::types::{AssertionType, ProcessorResults};

pub fn generate_markdown(results: &ProcessorResults) -> Result<String, std::io::Error> {
    let mut report = String::new();

    append_header(&mut report);
    append_overall_summary(&mut report, results);
    append_file_results(&mut report, results);

    write_report(report)
}

fn append_header(report: &mut String) {
    report.push_str("# HTTP File Runner - Test Report\n\n");
    report.push_str(&format!(
        "**Generated:** {}\n\n",
        format_local_datetime()
    ));
}

fn format_local_datetime() -> String {
    use std::time::SystemTime;
    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .expect("System time before UNIX EPOCH");
    
    let secs = now.as_secs();
    let days = secs / 86400;
    let hours = (secs % 86400) / 3600;
    let minutes = (secs % 3600) / 60;
    let seconds = secs % 60;
    
    let (year, month, day) = days_to_ymd(days);
    
    format!("{:04}-{:02}-{:02} {:02}:{:02}:{:02}", 
            year, month, day, hours, minutes, seconds)
}

fn days_to_ymd(days: u64) -> (u64, u64, u64) {
    let mut year = 1970;
    let mut remaining_days = days;
    
    loop {
        let days_in_year = if is_leap_year(year) { 366 } else { 365 };
        if remaining_days < days_in_year {
            break;
        }
        remaining_days -= days_in_year;
        year += 1;
    }
    
    let days_in_months = if is_leap_year(year) {
        [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    } else {
        [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    };
    
    let mut month = 1;
    for &days_in_month in &days_in_months {
        if remaining_days < days_in_month as u64 {
            break;
        }
        remaining_days -= days_in_month as u64;
        month += 1;
    }
    
    (year, month, remaining_days + 1)
}

fn is_leap_year(year: u64) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}

fn append_overall_summary(report: &mut String, results: &ProcessorResults) {
    let total_success: u32 = results.files.iter().map(|f| f.success_count).sum();
    let total_failed: u32 = results.files.iter().map(|f| f.failed_count).sum();
    let total_skipped: u32 = results.files.iter().map(|f| f.skipped_count).sum();
    let total_requests = total_success + total_failed + total_skipped;

    report.push_str("## Overall Summary\n\n");
    report.push_str(&format!("- **Total Requests:** {}\n", total_requests));
    report.push_str(&format!("- **Passed:** ✅ {}\n", total_success));
    report.push_str(&format!("- **Failed:** ❌ {}\n", total_failed));
    report.push_str(&format!("- **Skipped:** ⏭️ {}\n", total_skipped));
    report.push_str(&format!(
        "- **Success Rate:** {:.1}%\n\n",
        if total_requests > 0 {
            (total_success as f64 / total_requests as f64) * 100.0
        } else {
            0.0
        }
    ));
}

fn append_file_results(report: &mut String, results: &ProcessorResults) {
    for file_results in &results.files {
        report.push_str("---\n\n");
        report.push_str(&format!(
            "## File: `{}`\n\n",
            escape_markdown(&file_results.filename)
        ));

        report.push_str(&format!(
            "- **Passed:** {} | **Failed:** {} | **Skipped:** {}\n\n",
            file_results.success_count, file_results.failed_count, file_results.skipped_count
        ));

        for context in &file_results.result_contexts {
            append_request_section(report, context);
        }
    }
}

fn append_request_section(report: &mut String, context: &crate::types::RequestContext) {
    report.push_str(&format!(
        "### Request: {}\n\n",
        escape_markdown(&context.name)
    ));

    append_request_details(report, context);
    append_response_details(report, context);
}

fn append_request_details(report: &mut String, context: &crate::types::RequestContext) {
    report.push_str("#### Request Details\n\n");
    report.push_str(&format!("- **Method:** `{}`\n", context.request.method));
    report.push_str(&format!(
        "- **URL:** `{}`\n",
        escape_markdown(&context.request.url)
    ));

    if let Some(timeout) = context.request.timeout {
        report.push_str(&format!("- **Timeout:** {}ms\n", timeout));
    }
    if let Some(conn_timeout) = context.request.connection_timeout {
        report.push_str(&format!("- **Connection Timeout:** {}ms\n", conn_timeout));
    }
    if let Some(ref depends_on) = context.request.depends_on {
        report.push_str(&format!(
            "- **Depends On:** `{}`\n",
            escape_markdown(depends_on)
        ));
    }

    append_request_headers(report, &context.request.headers);
    append_request_body(report, &context.request.body);
    append_conditions(report, &context.request.conditions);
}

fn append_request_headers(report: &mut String, headers: &[crate::types::Header]) {
    if !headers.is_empty() {
        report.push_str("\n**Headers:**\n\n");
        report.push_str("| Header | Value |\n");
        report.push_str("|--------|-------|\n");
        for header in headers {
            report.push_str(&format!(
                "| {} | {} |\n",
                escape_markdown(&header.name),
                escape_markdown(&header.value)
            ));
        }
        report.push('\n');
    }
}

fn append_request_body(report: &mut String, body: &Option<String>) {
    if let Some(body) = body {
        report.push_str("**Request Body:**\n\n");
        report.push_str("```\n");
        report.push_str(body);
        report.push_str("\n```\n\n");
    }
}

fn append_conditions(report: &mut String, conditions: &[crate::types::Condition]) {
    if !conditions.is_empty() {
        report.push_str("**Conditions:**\n\n");
        for condition in conditions {
            let directive = if condition.negate { "@if-not" } else { "@if" };
            let cond_type = format!("{:?}", condition.condition_type);
            report.push_str(&format!(
                "- {} `{}.response.{}` == `{}`\n",
                directive,
                escape_markdown(&condition.request_name),
                cond_type,
                escape_markdown(&condition.expected_value)
            ));
        }
        report.push('\n');
    }
}

fn append_response_details(report: &mut String, context: &crate::types::RequestContext) {
    if let Some(ref result) = context.result {
        report.push_str("#### Response Details\n\n");

        let status_icon = if result.success { "✅" } else { "❌" };
        report.push_str(&format!(
            "- **Status:** {} {}\n",
            status_icon, result.status_code
        ));
        report.push_str(&format!("- **Duration:** {}ms\n", result.duration_ms));

        if let Some(ref error_msg) = result.error_message {
            report.push_str(&format!("- **Error:** {}\n", escape_markdown(error_msg)));
        }

        append_response_headers(report, &result.response_headers);
        append_response_body(report, &result.response_body);
        append_assertions(report, &result.assertion_results);
    } else {
        report.push_str("#### Response Details\n\n");
        report.push_str("⏭️ **Request was skipped**\n\n");
    }
}

fn append_response_headers(
    report: &mut String,
    headers: &Option<std::collections::HashMap<String, String>>,
) {
    if let Some(headers) = headers
        && !headers.is_empty() {
            report.push_str("\n**Response Headers:**\n\n");
            report.push_str("| Header | Value |\n");
            report.push_str("|--------|-------|\n");
            for (name, value) in headers {
                report.push_str(&format!(
                    "| {} | {} |\n",
                    escape_markdown(name),
                    escape_markdown(value)
                ));
            }
            report.push('\n');
        }
}

fn append_response_body(report: &mut String, body: &Option<String>) {
    if let Some(body) = body {
        report.push_str("**Response Body:**\n\n");
        report.push_str("```\n");
        report.push_str(body);
        report.push_str("\n```\n\n");
    }
}

fn append_assertions(report: &mut String, assertions: &[crate::types::AssertionResult]) {
    if !assertions.is_empty() {
        report.push_str("#### Assertion Results\n\n");
        report.push_str("| Type | Expected | Actual | Result |\n");
        report.push_str("|------|----------|--------|--------|\n");

        for assertion_result in assertions {
            let assertion_type_str = match assertion_result.assertion.assertion_type {
                AssertionType::Status => "Status Code",
                AssertionType::Body => "Response Body",
                AssertionType::Headers => "Response Headers",
            };

            let result_icon = if assertion_result.passed {
                "✅"
            } else {
                "❌"
            };
            let actual_val = assertion_result
                .actual_value
                .as_ref()
                .map(|v| escape_markdown(v))
                .unwrap_or_else(|| "N/A".to_string());

            report.push_str(&format!(
                "| {} | {} | {} | {} |\n",
                assertion_type_str,
                escape_markdown(&assertion_result.assertion.expected_value),
                actual_val,
                result_icon
            ));
        }
        report.push('\n');
    }
}
