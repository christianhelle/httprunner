use super::response_processor::{
    build_error_result, build_success_result, build_temp_result_for_assertions, extract_headers,
    should_capture_response,
};
use crate::assertions;
use crate::types::{HttpRequest, HttpResult};
use reqwest::Client;
use std::collections::HashMap;

#[cfg(not(target_arch = "wasm32"))]
use std::time::Instant;
#[cfg(target_arch = "wasm32")]
use web_time::Instant;

pub async fn execute_http_request_async(
    request: &HttpRequest,
    verbose: bool,
    insecure: bool,
) -> Result<HttpResult> {
    let client = build_client_async(request, insecure)?;
    let req_builder = build_request_async(&client, request)?;

    let start_time = Instant::now();

    let response = match req_builder.send().await {
        Ok(resp) => resp,
        Err(e) => {
            let duration_ms = start_time.elapsed().as_millis() as u64;
            let error_message = format!("Request failed: {}", e);
            return Ok(build_error_result(request, &error_message, duration_ms));
        }
    };

    let status_code = response.status().as_u16();
    let mut success = response.status().is_success();

    let (response_headers, response_body) =
        capture_response_details_async(request, verbose, response).await?;

    let duration_ms = start_time.elapsed().as_millis() as u64;

    let assertion_results = if !request.assertions.is_empty() {
        let temp_result = build_temp_result_for_assertions(
            request,
            status_code,
            success,
            duration_ms,
            response_headers.clone(),
            response_body.clone(),
        );

        let results = assertions::evaluate_assertions(&request.assertions, &temp_result);
        let all_passed = results.iter().all(|r| r.passed);
        success = all_passed;
        results
    } else {
        Vec::new()
    };

    Ok(build_success_result(
        request,
        status_code,
        success,
        duration_ms,
        response_headers,
        response_body,
        assertion_results,
    ))
}

fn build_client_async(_request: &HttpRequest, insecure: bool) -> Result<Client> {
    #[allow(unused_mut)]
    let mut client_builder = Client::builder();

    #[cfg(not(target_arch = "wasm32"))]
    {
        let connection_timeout = request.connection_timeout.unwrap_or(30_000);
        let read_timeout = request.timeout.unwrap_or(60_000);

        client_builder = client_builder
            .connect_timeout(std::time::Duration::from_millis(connection_timeout))
            .timeout(std::time::Duration::from_millis(read_timeout));

        if insecure {
            client_builder = client_builder
                .danger_accept_invalid_hostnames(true)
                .danger_accept_invalid_certs(true);
        }
    }

    #[cfg(target_arch = "wasm32")]
    let _ = insecure;

    Ok(client_builder.build()?)
}

fn build_request_async(client: &Client, request: &HttpRequest) -> Result<reqwest::RequestBuilder> {
    let method = request.method.to_uppercase();
    let method = reqwest::Method::from_bytes(method.as_bytes())?;

    let mut req_builder = client.request(method, &request.url);

    for header in &request.headers {
        req_builder = req_builder.header(&header.name, &header.value);
    }

    if let Some(ref body) = request.body {
        req_builder = req_builder.body(body.clone());
    }

    Ok(req_builder)
}

async fn capture_response_details_async(
    request: &HttpRequest,
    verbose: bool,
    response: reqwest::Response,
) -> Result<(Option<HashMap<String, String>>, Option<String>)> {
    if should_capture_response(request, verbose) {
        let headers = Some(extract_headers(response.headers()));
        let body = response.text().await.ok();
        Ok((headers, body))
    } else {
        Ok((None, None))
    }
}
