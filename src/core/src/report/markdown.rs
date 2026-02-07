use super::formatter::{ReportSummary, escape_markdown, format_local_datetime};
use super::writer::write_report;
use crate::types::ProcessorResults;

pub fn generate_markdown(results: &ProcessorResults) -> Result<String, std::io::Error> {
    let mut report = String::new();

    append_header(&mut report);
    append_overall_summary(&mut report, results);
    append_file_results(&mut report, results);

    write_report(report)
}

fn append_header(report: &mut String) {
    report.push_str("# HTTP File Runner - Test Report\n\n");
    report.push_str(&format!("**Generated:** {}\n\n", format_local_datetime()));
}

fn append_overall_summary(report: &mut String, results: &ProcessorResults) {
    let summary = ReportSummary::from_results(results);

    report.push_str("## Overall Summary\n\n");
    report.push_str(&format!("- **Total Requests:** {}\n", summary.total_requests));
    report.push_str(&format!("- **Passed:** ✅ {}\n", summary.total_success));
    report.push_str(&format!("- **Failed:** ❌ {}\n", summary.total_failed));
    report.push_str(&format!("- **Skipped:** ⏭️ {}\n", summary.total_skipped));
    report.push_str(&format!(
        "- **Success Rate:** {:.1}%\n\n",
        summary.success_rate
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
        && !headers.is_empty()
    {
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
            let assertion_type_str = assertion_result.assertion.assertion_type.to_string();

            let result_icon= if assertion_result.passed {
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
