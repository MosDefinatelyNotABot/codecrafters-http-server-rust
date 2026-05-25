use crate::{http_request::HttpRequest, http_response::HttpResponse, path_splitter::path_spilter};

pub(crate) type RequestHandler = Box<dyn Fn(&HttpRequest) -> String + Send + Sync>;

pub(crate) fn root_handler(_: &HttpRequest) -> String {
    // doubles as a health check
    "HTTP/1.1 200 OK\r\n\r\nHealthy".to_string()
}

pub(crate) fn error_handler(_: &HttpRequest) -> String {
    "HTTP/1.1 404 Not Found\r\n\r\n".to_string()
}

pub(crate) fn echo_handler(request: &HttpRequest) -> String {
    let (_base_path, path_chunks) = path_spilter(
        request
            ._target_path
            .as_ref()
            .expect("No path specified. :("),
    )
    .unwrap_or_default();

    let body = path_chunks.first().cloned().unwrap_or_default();
    let mut http_response = HttpResponse::default();

    let headers = vec![
        ("Content-Type".to_string(), "text/plain".to_string()),
        ("Content-Length".to_string(), body.len().to_string()),
    ];
    http_response.headers = headers;
    http_response.body = Some(body);

    http_response.get_response()
}

pub(crate) fn user_agent_handler(request: &HttpRequest) -> String {
    // if let Some(user_agent) = &request._headers.get("User-Agent") {
    //     let mut http_response = HttpResponse::default();

    if let Some(user_agent) = request._headers.get("User-Agent") {
        let mut http_response = HttpResponse::default();

        let headers = vec![
            ("Content-Type".to_string(), "text/plain".to_string()),
            ("Content-Length".to_string(), user_agent.len().to_string()),
        ];
        http_response.headers = headers;
        http_response.body = Some(user_agent.to_owned());

        http_response.get_response()
    } else {
        error_handler(request)
    }
}
