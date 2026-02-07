use super::formatter::{ReportSummary, format_local_datetime};
use super::writer::write_report_with_extension;
use crate::types::ProcessorResults;

pub fn generate_html(results: &ProcessorResults) -> Result<String, std::io::Error> {
    let mut html = String::new();

    append_html_header(&mut html);
    append_overall_summary(&mut html, results);
    append_file_results(&mut html, results);
    append_html_footer(&mut html);

    write_report_with_extension(html, "html")
}

fn append_html_header(html: &mut String) {
    html.push_str("<!DOCTYPE html>\n");
    html.push_str("<html lang=\"en\">\n");
    html.push_str("<head>\n");
    html.push_str("    <meta charset=\"UTF-8\">\n");
    html.push_str(
        "    <meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">\n",
    );
    html.push_str("    <title>HTTP File Runner - Test Report</title>\n");
    html.push_str("    <style>\n");
    html.push_str(include_str!("report_style.css"));
    html.push_str("    </style>\n");
    html.push_str("</head>\n");
    html.push_str("<body>\n");
    html.push_str("    <div class=\"container\">\n");
    html.push_str("        <header>\n");
    html.push_str("            <h1>🚀 HTTP File Runner - Test Report</h1>\n");
    html.push_str(&format!(
        "            <p class=\"timestamp\">Generated: {}</p>\n",
        format_local_datetime()
    ));
    html.push_str("        </header>\n");
}

fn append_overall_summary(html: &mut String, results: &ProcessorResults) {
    let summary = ReportSummary::from_results(results);

    html.push_str("        <section class=\"summary\">\n");
    html.push_str("            <h2>Overall Summary</h2>\n");
    html.push_str("            <div class=\"stats-grid\">\n");
    html.push_str(&format!(
        "                <div class=\"stat-card\"><div class=\"stat-label\">Total Requests</div><div class=\"stat-value\">{}</div></div>\n",
        summary.total_requests
    ));
    html.push_str(&format!(
        "                <div class=\"stat-card success\"><div class=\"stat-label\">Passed</div><div class=\"stat-value\">✅ {}</div></div>\n",
        summary.total_success
    ));
    html.push_str(&format!(
        "                <div class=\"stat-card failed\"><div class=\"stat-label\">Failed</div><div class=\"stat-value\">❌ {}</div></div>\n",
        summary.total_failed
    ));
    html.push_str(&format!(
        "                <div class=\"stat-card skipped\"><div class=\"stat-label\">Skipped</div><div class=\"stat-value\">⏭️ {}</div></div>\n",
        summary.total_skipped
    ));
    html.push_str(&format!(
        "                <div class=\"stat-card\"><div class=\"stat-label\">Success Rate</div><div class=\"stat-value\">{:.1}%</div></div>\n",
        summary.success_rate
    ));
    html.push_str("            </div>\n");
    html.push_str("        </section>\n");
}

fn append_file_results(html: &mut String, results: &ProcessorResults) {
    for file_results in &results.files {
        html.push_str("        <section class=\"file-section\">\n");
        html.push_str(&format!(
            "            <h2>📄 File: <code>{}</code></h2>\n",
            escape_html(&file_results.filename)
        ));
        html.push_str(&format!(
            "            <p class=\"file-stats\">Passed: {} | Failed: {} | Skipped: {}</p>\n",
            file_results.success_count, file_results.failed_count, file_results.skipped_count
        ));

        for context in &file_results.result_contexts {
            append_request_section(html, context);
        }

        html.push_str("        </section>\n");
    }
}

fn append_request_section(html: &mut String, context: &crate::types::RequestContext) {
    html.push_str("            <div class=\"request-card\">\n");
    html.push_str(&format!(
        "                <h3>{}</h3>\n",
        escape_html(&context.name)
    ));

    append_request_details(html, context);
    append_response_details(html, context);

    html.push_str("            </div>\n");
}

fn append_request_details(html: &mut String, context: &crate::types::RequestContext) {
    html.push_str("                <div class=\"details-section\">\n");
    html.push_str("                    <h4>Request Details</h4>\n");
    html.push_str("                    <ul class=\"details-list\">\n");
    html.push_str(&format!(
        "                        <li><strong>Method:</strong> <code>{}</code></li>\n",
        context.request.method
    ));
    html.push_str(&format!(
        "                        <li><strong>URL:</strong> <code>{}</code></li>\n",
        escape_html(&context.request.url)
    ));

    if let Some(timeout) = context.request.timeout {
        html.push_str(&format!(
            "                        <li><strong>Timeout:</strong> {}ms</li>\n",
            timeout
        ));
    }
    if let Some(conn_timeout) = context.request.connection_timeout {
        html.push_str(&format!(
            "                        <li><strong>Connection Timeout:</strong> {}ms</li>\n",
            conn_timeout
        ));
    }
    if let Some(ref depends_on) = context.request.depends_on {
        html.push_str(&format!(
            "                        <li><strong>Depends On:</strong> <code>{}</code></li>\n",
            escape_html(depends_on)
        ));
    }

    html.push_str("                    </ul>\n");

    append_request_headers(html, &context.request.headers);
    append_request_body(html, &context.request.body);
    append_conditions(html, &context.request.conditions);

    html.push_str("                </div>\n");
}

fn append_request_headers(html: &mut String, headers: &[crate::types::Header]) {
    if !headers.is_empty() {
        html.push_str("                    <h5>Headers</h5>\n");
        html.push_str("                    <table class=\"data-table\">\n");
        html.push_str(
            "                        <thead><tr><th>Header</th><th>Value</th></tr></thead>\n",
        );
        html.push_str("                        <tbody>\n");
        for header in headers {
            html.push_str(&format!(
                "                            <tr><td>{}</td><td>{}</td></tr>\n",
                escape_html(&header.name),
                escape_html(&header.value)
            ));
        }
        html.push_str("                        </tbody>\n");
        html.push_str("                    </table>\n");
    }
}

fn append_request_body(html: &mut String, body: &Option<String>) {
    if let Some(body) = body {
        html.push_str("                    <h5>Request Body</h5>\n");
        html.push_str("                    <pre class=\"code-block\"><code>");
        html.push_str(&escape_html(body));
        html.push_str("</code></pre>\n");
    }
}

fn append_conditions(html: &mut String, conditions: &[crate::types::Condition]) {
    if !conditions.is_empty() {
        html.push_str("                    <h5>Conditions</h5>\n");
        html.push_str("                    <ul class=\"details-list\">\n");
        for condition in conditions {
            let directive = if condition.negate { "@if-not" } else { "@if" };
            let cond_type = format!("{:?}", condition.condition_type);
            html.push_str(&format!(
                "                        <li>{} <code>{}.response.{}</code> == <code>{}</code></li>\n",
                directive,
                escape_html(&condition.request_name),
                cond_type,
                escape_html(&condition.expected_value)
            ));
        }
        html.push_str("                    </ul>\n");
    }
}

fn append_response_details(html: &mut String, context: &crate::types::RequestContext) {
    html.push_str("                <div class=\"details-section\">\n");
    html.push_str("                    <h4>Response Details</h4>\n");

    if let Some(ref result) = context.result {
        let status_class = if result.success { "success" } else { "failed" };
        let status_icon = if result.success { "✅" } else { "❌" };

        html.push_str("                    <ul class=\"details-list\">\n");
        html.push_str(&format!(
            "                        <li><strong>Status:</strong> <span class=\"status {}\">{} {}</span></li>\n",
            status_class, status_icon, result.status_code
        ));
        html.push_str(&format!(
            "                        <li><strong>Duration:</strong> {}ms</li>\n",
            result.duration_ms
        ));

        if let Some(ref error_msg) = result.error_message {
            html.push_str(&format!(
                "                        <li class=\"error\"><strong>Error:</strong> {}</li>\n",
                escape_html(error_msg)
            ));
        }

        html.push_str("                    </ul>\n");

        append_response_headers(html, &result.response_headers);
        append_response_body(html, &result.response_body);
        append_assertions(html, &result.assertion_results);
    } else {
        html.push_str("                    <p class=\"skipped\">⏭️ Request was skipped</p>\n");
    }

    html.push_str("                </div>\n");
}

fn append_response_headers(
    html: &mut String,
    headers: &Option<std::collections::HashMap<String, String>>,
) {
    if let Some(headers) = headers
        && !headers.is_empty()
    {
        html.push_str("                    <h5>Response Headers</h5>\n");
        html.push_str("                    <table class=\"data-table\">\n");
        html.push_str(
            "                        <thead><tr><th>Header</th><th>Value</th></tr></thead>\n",
        );
        html.push_str("                        <tbody>\n");
        for (name, value) in headers {
            html.push_str(&format!(
                "                            <tr><td>{}</td><td>{}</td></tr>\n",
                escape_html(name),
                escape_html(value)
            ));
        }
        html.push_str("                        </tbody>\n");
        html.push_str("                    </table>\n");
    }
}

fn append_response_body(html: &mut String, body: &Option<String>) {
    if let Some(body) = body {
        html.push_str("                    <h5>Response Body</h5>\n");
        html.push_str("                    <pre class=\"code-block\"><code>");
        html.push_str(&escape_html(body));
        html.push_str("</code></pre>\n");
    }
}

fn append_assertions(html: &mut String, assertions: &[crate::types::AssertionResult]) {
    if !assertions.is_empty() {
        html.push_str("                    <h4>Assertion Results</h4>\n");
        html.push_str("                    <table class=\"data-table\">\n");
        html.push_str("                        <thead><tr><th>Type</th><th>Expected</th><th>Actual</th><th>Result</th></tr></thead>\n");
        html.push_str("                        <tbody>\n");

        for assertion_result in assertions {
            let assertion_type_str = assertion_result.assertion.assertion_type.to_string();

            let result_class= if assertion_result.passed {
                "success"
            } else {
                "failed"
            };
            let result_icon = if assertion_result.passed {
                "✅"
            } else {
                "❌"
            };
            let actual_val = assertion_result
                .actual_value
                .as_ref()
                .map(|v| escape_html(v))
                .unwrap_or_else(|| "N/A".to_string());

            html.push_str(&format!(
                "                            <tr><td>{}</td><td>{}</td><td>{}</td><td class=\"{}\">{}</td></tr>\n",
                assertion_type_str,
                escape_html(&assertion_result.assertion.expected_value),
                actual_val,
                result_class,
                result_icon
            ));
        }

        html.push_str("                        </tbody>\n");
        html.push_str("                    </table>\n");
    }
}

fn append_html_footer(html: &mut String) {
    html.push_str("    </div>\n");
    html.push_str("</body>\n");
    html.push_str("</html>\n");
}

fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}
