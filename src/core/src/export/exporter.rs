use std::{
    fs::{File, OpenOptions},
    io::Write,
    time::{SystemTime, UNIX_EPOCH},
};

use crate::processor::format_json_if_valid;
use crate::types::{Header, ProcessorResults};

enum ExportType {
    Request,
    Response,
}

pub struct ExportResults {
    pub file_names: Vec<String>,
    pub failed_file_names: Vec<String>,
}

pub fn export_results(
    results: &ProcessorResults,
    pretty_json: bool,
) -> Result<ExportResults, std::io::Error> {
    let mut file_names = Vec::new();
    let mut failed_file_names = Vec::new();
    let timestamp = get_timestamp();
    for file_results in &results.files {
        for test_results in &file_results.result_contexts {
            match export_request(timestamp, test_results, pretty_json) {
                Ok(file_name) => {
                    file_names.push(file_name);
                }
                Err(err) => {
                    failed_file_names.push(format!("{}: {}", test_results.name, err));
                }
            }
            match export_response(timestamp, test_results, pretty_json) {
                Ok(file_name) => {
                    file_names.push(file_name);
                }
                Err(err) => {
                    failed_file_names.push(format!("{}: {}", test_results.name, err));
                }
            }
        }
    }

    Ok(ExportResults {
        file_names,
        failed_file_names,
    })
}

fn export_response(
    timestamp: u64,
    test_results: &crate::types::RequestContext,
    pretty_json: bool,
) -> Result<String, std::io::Error> {
    let base_filename = format!("{}_response", &test_results.name);
    let file = ExportFile::new(base_filename, timestamp)?;
    write_http_request_response(&test_results, ExportType::Response, file.file, pretty_json)?;
    Ok(file.file_name)
}

fn export_request(
    timestamp: u64,
    test_results: &crate::types::RequestContext,
    pretty_json: bool,
) -> Result<String, std::io::Error> {
    let base_filename = format!("{}_request", &test_results.name);
    let file = ExportFile::new(base_filename, timestamp)?;
    write_http_request_response(&test_results, ExportType::Request, file.file, pretty_json)?;
    Ok(file.file_name)
}

fn get_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

fn write_http_request_response(
    test_results: &&crate::types::RequestContext,
    export_type: ExportType,
    file: File,
    pretty_json: bool,
) -> Result<(), std::io::Error> {
    match export_type {
        ExportType::Request => write_http_request(test_results, file, pretty_json),
        ExportType::Response => write_http_response(test_results, file, pretty_json),
    }
}

fn write_http_response(
    test_results: &&crate::types::RequestContext,
    mut file: File,
    pretty_json: bool,
) -> Result<(), std::io::Error> {
    if let Some(result) = &test_results.result {
        let status_line = format!("HTTP/1.1 {}\r\n", result.status_code);
        file.write_all(status_line.as_bytes())?;
        write_http_headers(test_results, ExportType::Response, &file)?;
        if let Some(response_body) = &result.response_body {
            if pretty_json {
                let body = format!("{}\r\n", format_json_if_valid(response_body));
                file.write_all(body.as_bytes())?;
            } else {
                let body = format!("{}\r\n", response_body);
                file.write_all(body.as_bytes())?;
            }
        }
    }
    Ok(())
}

fn write_http_request(
    test_results: &&crate::types::RequestContext,
    mut file: File,
    pretty_json: bool,
) -> Result<(), std::io::Error> {
    write_http_headers(test_results, ExportType::Request, &file)?;
    if let Some(request_body) = &test_results.request.body {
        if pretty_json {
            let body = format!("{}\r\n", format_json_if_valid(request_body));
            file.write_all(body.as_bytes())?;
        } else {
            let body = format!("{}\r\n", request_body);
            file.write_all(body.as_bytes())?;
        }
    }
    Ok(())
}

fn write_http_headers(
    test_results: &&crate::types::RequestContext,
    export_type: ExportType,
    mut file: &File,
) -> Result<(), std::io::Error> {
    let headers = match export_type {
        ExportType::Request => {
            let header = format!(
                "{} {}\r\n",
                test_results.request.method, test_results.request.url
            );
            file.write_all(header.as_bytes())?;
            test_results.request.headers.clone()
        }
        ExportType::Response => match &test_results.result {
            Some(result) => {
                let mut headers = Vec::<Header>::new();
                if let Some(response_headers) = &result.response_headers {
                    response_headers.iter().for_each(|(k, v)| {
                        headers.push(Header {
                            name: k.to_string(),
                            value: v.to_string(),
                        });
                    });
                }
                headers
            }
            None => Vec::new(),
        },
    };
    for header in headers.iter() {
        let header_line = format!("{}: {}\r\n", header.name, header.value);
        file.write_all(header_line.as_bytes())?;
    }
    file.write_all("\r\n".as_bytes())?;
    Ok(())
}

struct ExportFile {
    file_name: String,
    file: File,
}

impl ExportFile {
    fn new(base_filename: String, timestamp: u64) -> Result<Self, std::io::Error> {
        let log_filename = get_filename(base_filename, timestamp);
        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&log_filename)?;

        Ok(ExportFile {
            file_name: log_filename,
            file,
        })
    }
}

fn get_filename(base_filename: String, timestamp: u64) -> String {
    let log_filename = format!("{}_{}.log", base_filename, timestamp,);
    log_filename
}
