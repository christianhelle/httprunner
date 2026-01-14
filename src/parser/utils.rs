pub fn is_http_request_line(line: &str) -> bool {
    line.contains("HTTP/")
        || line.starts_with("GET ")
        || line.starts_with("POST ")
        || line.starts_with("PUT ")
        || line.starts_with("DELETE ")
        || line.starts_with("PATCH ")
        || line.starts_with("HEAD ")
        || line.starts_with("OPTIONS ")
        || line.starts_with("TRACE ")
        || line.starts_with("CONNECT ")
}
