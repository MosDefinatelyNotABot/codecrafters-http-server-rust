use std::{collections::HashMap, fs, sync::LazyLock};

use crate::{
    file_utils::DIR_PATH, http_request::HttpRequest, http_response, path_splitter::path_spilter,
};

pub(crate) type RequestHandler = Box<dyn Fn(&HttpRequest) -> String + Send + Sync>;

// route mapper. maps a path to a handler function.
pub(crate) static ROUTES: LazyLock<HashMap<String, RequestHandler>> = LazyLock::new(|| {
    [
        (
            "/root".to_string(),
            Box::new(root_handler) as RequestHandler,
        ),
        (
            "/error".to_string(),
            Box::new(error_handler) as RequestHandler,
        ),
        (
            "/echo".to_string(),
            Box::new(echo_handler) as RequestHandler,
        ),
        (
            "/user-agent".to_string(),
            Box::new(user_agent_handler) as RequestHandler,
        ),
        (
            "/files".to_string(),
            Box::new(file_handler) as RequestHandler,
        ),
    ]
    .into_iter()
    .collect()
});

pub(crate) fn root_handler(request: &HttpRequest) -> String {
    println!("[root_handler] request target: {:?}", request._target_path);
    let response = "HTTP/1.1 200 OK\r\n\r\nHealthy".to_string();
    println!("[root_handler] returning 200 OK");
    response
}

pub(crate) fn error_handler(request: &HttpRequest) -> String {
    println!("[error_handler] request target: {:?}", request._target_path);
    "HTTP/1.1 404 Not Found\r\n\r\n".to_string()
}

pub(crate) fn echo_handler(request: &HttpRequest) -> String {
    println!("[echo_handler] request target: {:?}", request._target_path);
    let (_base_path, path_chunks) = path_spilter(
        request
            ._target_path
            .as_ref()
            .expect("No path specified. :("),
    )
    .unwrap_or_default();

    let body = path_chunks.first().cloned().unwrap_or_default();
    println!("[echo_handler] echoing body: {:?}", body);

    let http_response = http_response::HttpResponse {
        http_version: "HTTP/1.1".to_string(),
        status: "200 OK".to_string(),
        headers: vec![
            ("Content-Type".to_string(), "text/plain".to_string()),
            ("Content-Length".to_string(), body.len().to_string()),
        ],
        body: Some(body),
    };

    http_response.get_response()
}

pub(crate) fn user_agent_handler(request: &HttpRequest) -> String {
    println!(
        "[user_agent_handler] request target: {:?}",
        request._target_path
    );
    println!("[user_agent_handler] headers: {:?}", request._headers);

    if let Some(user_agent) = request._headers.get("User-Agent") {
        println!("[user_agent_handler] User-Agent: {:?}", user_agent);

        let http_response = http_response::HttpResponse {
            http_version: "HTTP/1.1".to_string(),
            status: "200 OK".to_string(),
            headers: vec![
                ("Content-Type".to_string(), "text/plain".to_string()),
                ("Content-Length".to_string(), user_agent.len().to_string()),
            ],
            body: Some(user_agent.to_owned()),
        };

        http_response.get_response()
    } else {
        println!("[user_agent_handler] User-Agent header not found, returning 404");
        error_handler(request)
    }
}

pub(crate) fn file_handler(request: &HttpRequest) -> String {
    println!("[file_handler] request target: {:?}", request._target_path);
    if let Some(dir_path) = DIR_PATH.get() {
        println!("[file_handler] serving from dir: {:?}", dir_path);
        let (_base_path, path_chunks) = path_spilter(
            request
                ._target_path
                .as_ref()
                .expect("No path specified. :("),
        )
        .unwrap_or_default();

        println!("[file_handler] path_chunks: {:?}", path_chunks);

        let requested_fname = dir_path.join(path_chunks.first().expect("No requested file. :("));

        println!(
            "[file_handler] full path: {:?}, is_file: {}",
            requested_fname,
            requested_fname.is_file()
        );

        if requested_fname.is_file() {
            let bytes = fs::read(&requested_fname).expect("Error reading file. :(");
            let content_length = bytes.len();
            println!("[file_handler] read {} bytes", content_length);

            let http_response = http_response::HttpResponse {
                http_version: "HTTP/1.1".to_string(),
                status: "200 OK".to_string(),
                headers: vec![
                    (
                        "Content-Type".to_string(),
                        "application/octet-stream".to_string(),
                    ),
                    ("Content-Length".to_string(), content_length.to_string()),
                ],
                body: Some(String::from_utf8_lossy(&bytes).into_owned()),
            };

            http_response.get_response()
        } else {
            println!("[file_handler] file not found, returning 404");
            error_handler(request)
        }
    } else {
        println!("[file_handler] DIR_PATH not set, returning 404");
        error_handler(request)
    }
}
