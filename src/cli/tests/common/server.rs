use anyhow::{Context, Result};
use serde_json::{Map, Value, json};
use std::collections::HashMap;
use std::io::Cursor;
use std::net::TcpListener;
use std::sync::{
    Arc, Mutex,
    mpsc::{self, Receiver, Sender},
};
use std::thread::{self, JoinHandle};
use std::time::Duration;
use tiny_http::{Header, Method, Request, Response, Server, StatusCode};

type HttpResponse = Response<Cursor<Vec<u8>>>;

#[derive(Default)]
struct ServerState {
    next_user_id: u64,
    users: HashMap<String, Value>,
}

pub struct TestServer {
    base_url: String,
    shutdown_tx: Option<Sender<()>>,
    handle: Option<JoinHandle<()>>,
}

impl TestServer {
    pub fn start() -> Result<Self> {
        let listener =
            TcpListener::bind("127.0.0.1:0").context("bind local listener for test server")?;
        let address = listener
            .local_addr()
            .context("read local address for test server")?;
        let server = Server::from_listener(listener, None)
            .map_err(anyhow::Error::msg)
            .context("create tiny_http server")?;
        let base_url = format!("http://{}", address);
        let state = Arc::new(Mutex::new(ServerState::default()));
        let (shutdown_tx, shutdown_rx) = mpsc::channel();
        let server_base_url = base_url.clone();
        let handle = thread::spawn(move || run_server(server, state, shutdown_rx, server_base_url));

        Ok(Self {
            base_url,
            shutdown_tx: Some(shutdown_tx),
            handle: Some(handle),
        })
    }

    pub fn base_url(&self) -> &str {
        &self.base_url
    }
}

impl Drop for TestServer {
    fn drop(&mut self) {
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(());
        }

        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}

fn run_server(
    server: Server,
    state: Arc<Mutex<ServerState>>,
    shutdown_rx: Receiver<()>,
    base_url: String,
) {
    loop {
        if shutdown_rx.try_recv().is_ok() {
            break;
        }

        match server.recv_timeout(Duration::from_millis(25)) {
            Ok(Some(request)) => handle_request(request, &base_url, &state),
            Ok(None) => {}
            Err(_) => break,
        }
    }
}

fn handle_request(mut request: Request, base_url: &str, state: &Arc<Mutex<ServerState>>) {
    let request_url = request.url().to_string();
    let (path, query) = split_url(&request_url);
    let headers = read_headers(&request);
    let body = read_body(&mut request);
    let response = route_request(
        request.method(),
        &path,
        query.as_deref(),
        &headers,
        &body,
        base_url,
        state,
    );

    let _ = request.respond(response);
}

fn route_request(
    method: &Method,
    path: &str,
    query: Option<&str>,
    headers: &Map<String, Value>,
    body: &str,
    base_url: &str,
    state: &Arc<Mutex<ServerState>>,
) -> HttpResponse {
    match (method, path) {
        (_, "/health") => json_response(200, json!({ "status": "ok" })),
        (_, "/headers") => json_response(200, json!({ "headers": Value::Object(headers.clone()) })),
        (_, "/json") => json_response(
            200,
            json!({
                "slideshow": {
                    "title": "Sample Slide Show",
                    "author": "httprunner"
                }
            }),
        ),
        (_, "/uuid") => json_response(
            200,
            json!({ "uuid": "123e4567-e89b-12d3-a456-426614174000" }),
        ),
        (_, "/zen") => text_response(200, "Keep it logically awesome."),
        (_, "/posts/1") => json_response(
            200,
            json!({
                "id": 1,
                "title": "Local post",
                "body": "This came from the smoke test server"
            }),
        ),
        (_, "/users/1") => json_response(
            200,
            json!({
                "id": 1,
                "name": "Local User",
                "email": "local@example.com"
            }),
        ),
        (_, "/repos") => {
            let mut response = json_response(
                200,
                json!([
                    { "id": 1, "name": "httprunner", "owner": "christianhelle" }
                ]),
            );
            response.add_header(make_header(
                "Link",
                &format!("<{base_url}/repos?page=2>; rel=\"next\""),
            ));
            response
        }
        (_, _) if path.starts_with("/status/") => {
            let status = path.trim_start_matches("/status/").parse().unwrap_or(500);
            json_response(status, json!({ "status": status }))
        }
        (_, _) if path.starts_with("/delay/") => {
            let delay_ms = path.trim_start_matches("/delay/").parse().unwrap_or(0);
            thread::sleep(Duration::from_millis(delay_ms));
            json_response(200, json!({ "delay_ms": delay_ms, "status": "ok" }))
        }
        (&Method::Get, "/get") => json_response(
            200,
            json!({
                "args": Value::Object(parse_query(query)),
                "headers": Value::Object(headers.clone()),
                "url": format!("{base_url}{path}"),
            }),
        ),
        (&Method::Post, "/post") | (&Method::Put, "/put") | (&Method::Delete, "/delete") => {
            json_response(
                200,
                json!({
                    "args": Value::Object(parse_query(query)),
                    "headers": Value::Object(headers.clone()),
                    "json": parse_json_body(body),
                    "method": method.to_string(),
                    "url": format!("{base_url}{path}"),
                }),
            )
        }
        (&Method::Post, "/api/users") => {
            let mut state = state.lock().expect("server state lock poisoned");
            state.next_user_id += 1;
            let user_id = format!("user-{}", state.next_user_id);
            let mut user = parse_json_body(body);

            if let Value::Object(map) = &mut user {
                map.insert("id".to_string(), Value::String(user_id.clone()));
            }

            state.users.insert(user_id, user.clone());
            json_response(201, user)
        }
        (&Method::Get, _) if path.starts_with("/api/users/") => {
            let user_id = path.trim_start_matches("/api/users/");
            let state = state.lock().expect("server state lock poisoned");
            match state.users.get(user_id) {
                Some(user) => json_response(200, user.clone()),
                None => json_response(404, json!({ "error": "not found" })),
            }
        }
        _ => json_response(404, json!({ "error": "not found" })),
    }
}

fn split_url(url: &str) -> (String, Option<String>) {
    match url.split_once('?') {
        Some((path, query)) => (path.to_string(), Some(query.to_string())),
        None => (url.to_string(), None),
    }
}

fn read_headers(request: &Request) -> Map<String, Value> {
    let mut headers = Map::new();
    for header in request.headers() {
        headers.insert(
            header.field.as_str().to_string(),
            Value::String(header.value.as_str().to_string()),
        );
    }
    headers
}

fn read_body(request: &mut Request) -> String {
    let mut body = String::new();
    let _ = request.as_reader().read_to_string(&mut body);
    body
}

fn parse_query(query: Option<&str>) -> Map<String, Value> {
    let mut args = Map::new();
    let Some(query) = query else {
        return args;
    };

    for pair in query.split('&') {
        if pair.is_empty() {
            continue;
        }

        let (key, value) = pair.split_once('=').unwrap_or((pair, ""));
        args.insert(key.to_string(), Value::String(value.to_string()));
    }

    args
}

fn parse_json_body(body: &str) -> Value {
    if body.trim().is_empty() {
        Value::Null
    } else {
        serde_json::from_str(body).unwrap_or_else(|_| Value::String(body.to_string()))
    }
}

fn json_response(status: u16, body: Value) -> HttpResponse {
    response(status, body.to_string().into_bytes(), "application/json")
}

fn text_response(status: u16, body: &str) -> HttpResponse {
    response(status, body.as_bytes().to_vec(), "text/plain")
}

fn response(status: u16, body: Vec<u8>, content_type: &str) -> HttpResponse {
    let mut response = Response::from_data(body).with_status_code(StatusCode(status));
    response.add_header(make_header("Content-Type", content_type));
    response
}

fn make_header(name: &str, value: &str) -> Header {
    Header::from_bytes(name.as_bytes(), value.as_bytes()).expect("valid static header")
}
