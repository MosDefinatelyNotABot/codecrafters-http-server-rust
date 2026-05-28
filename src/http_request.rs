use std::{collections::HashMap, pin::Pin};

use tokio::io::{AsyncBufRead, AsyncBufReadExt, AsyncReadExt};

pub(crate) struct HttpRequest {
    pub _method: Option<String>,
    pub _target_path: Option<String>,
    pub _http_version: Option<String>,
    pub _headers: HashMap<String, String>,
    pub _body: Option<String>,
}

/// Reads the request from the socket, returning (request_line, headers, body).
pub(crate) async fn fetch_request_str(
    mut reader: Pin<&mut impl AsyncBufRead>,
) -> (String, String, Option<String>) {
    let mut request_line = String::new();
    let mut headers_buffer = String::new();

    // read request line
    reader
        .as_mut()
        .read_line(&mut request_line)
        .await
        .expect("Error reading request line. :(");

    // read headers until blank line
    loop {
        let mut line = String::new();
        reader
            .as_mut()
            .read_line(&mut line)
            .await
            .expect("Error reading headers. :(");
        if line == "\r\n" || line.is_empty() {
            break;
        }
        headers_buffer.push_str(&line);
    }

    // read body if Content-Length header is present
    let content_length: usize = headers_buffer
        .lines()
        .find_map(|l| l.strip_prefix("Content-Length: "))
        .and_then(|v| v.trim().parse().ok())
        .unwrap_or(0);

    let body = if content_length > 0 {
        let mut body_buf = vec![0u8; content_length];
        reader
            .as_mut()
            .read_exact(&mut body_buf)
            .await
            .expect("Error reading body. :(");
        Some(String::from_utf8_lossy(&body_buf).into_owned())
    } else {
        None
    };

    (request_line, headers_buffer, body)
}

/// Struct representing an HTTP request.
impl HttpRequest {
    /// Parses the output of `fetch_request_str` into an `HttpRequest`.
    pub(crate) fn parse_request(
        req_line: &str,
        headers_str: &str,
        body: Option<String>,
    ) -> HttpRequest {
        // parse request line
        let parts: Vec<&str> = req_line.trim_end().split(' ').collect();
        let method = parts.first().map(|s| s.to_string());
        let target_path = parts
            .get(1)
            .filter(|s| s.starts_with('/') && s.len() > 1)
            .map(|s| s.to_string());
        let http_version = parts.get(2).map(|s| s.to_string());

        // parse headers
        let headers: HashMap<String, String> = headers_str
            .lines()
            .filter_map(|line| line.split_once(": "))
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();

        HttpRequest {
            _method: method,
            _target_path: target_path,
            _http_version: http_version,
            _headers: headers,
            _body: body,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_well_formed_request() {
        let req = HttpRequest::parse_request(
            "GET /index.html HTTP/1.1",
            "Host: localhost\r\nUser-Agent: foobar/1.2.3",
            Some("Hello body".to_string()),
        );

        assert_eq!(req._method.as_deref(), Some("GET"));
        assert_eq!(req._target_path.as_deref(), Some("/index.html"));
        assert_eq!(req._http_version.as_deref(), Some("HTTP/1.1"));
        assert_eq!(
            req._headers.get("Host").map(|s| s.as_str()),
            Some("localhost")
        );
        assert_eq!(
            req._headers.get("User-Agent").map(|s| s.as_str()),
            Some("foobar/1.2.3")
        );
        assert_eq!(req._body.as_deref(), Some("Hello body"));
    }

    #[test]
    fn parses_multiple_headers() {
        let req = HttpRequest::parse_request(
            "GET /index.html HTTP/1.1",
            "Host: localhost\r\nUser-Agent: foobar/1.2.3\r\nAccept: */*",
            None,
        );

        assert_eq!(
            req._headers.get("Host").map(|s| s.as_str()),
            Some("localhost")
        );
        assert_eq!(
            req._headers.get("User-Agent").map(|s| s.as_str()),
            Some("foobar/1.2.3")
        );
        assert_eq!(req._headers.get("Accept").map(|s| s.as_str()), Some("*/*"));
    }

    #[test]
    fn parses_request_line_only() {
        let req = HttpRequest::parse_request("POST /submit HTTP/1.1", "", None);

        assert_eq!(req._method.as_deref(), Some("POST"));
        assert_eq!(req._target_path.as_deref(), Some("/submit"));
        assert_eq!(req._http_version.as_deref(), Some("HTTP/1.1"));
        assert!(req._headers.is_empty());
        assert!(req._body.is_none());
    }

    #[test]
    fn handles_malformed_request_line() {
        let req = HttpRequest::parse_request("BADREQUEST", "Host: localhost", None);

        assert_eq!(req._method.as_deref(), Some("BADREQUEST"));
        assert!(req._target_path.is_none());
        assert!(req._http_version.is_none());
    }

    #[test]
    fn handles_empty_input() {
        let req = HttpRequest::parse_request("", "", None);

        assert_eq!(req._method.as_deref(), Some(""));
        assert!(req._target_path.is_none());
        assert!(req._http_version.is_none());
        assert!(req._headers.is_empty());
        assert!(req._body.is_none());
    }

    #[test]
    fn root_path_yields_no_target_path() {
        let req = HttpRequest::parse_request("GET / HTTP/1.1", "Host: localhost", None);

        assert_eq!(req._method.as_deref(), Some("GET"));
        assert!(req._target_path.is_none());
        assert_eq!(req._http_version.as_deref(), Some("HTTP/1.1"));
    }

    #[test]
    fn path_without_leading_slash_yields_no_target_path() {
        let req = HttpRequest::parse_request("GET noslash HTTP/1.1", "", None);

        assert_eq!(req._method.as_deref(), Some("GET"));
        assert!(req._target_path.is_none());
    }
}
