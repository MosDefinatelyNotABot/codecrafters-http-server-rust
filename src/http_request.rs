use std::collections::HashMap;

pub(crate) struct HttpRequest {
    pub _method: Option<String>,
    pub _target_path: Option<String>,
    pub _http_version: Option<String>,
    pub _headers: HashMap<String, String>,
    pub _body: Option<String>,
}

impl HttpRequest {
    pub(crate) fn parse_request(request: &str) -> HttpRequest {
        let chunks = request.split("\r\n");

        let mut method = None;
        let mut target_path = None;
        let mut http_version = None;
        let mut headers: HashMap<String, String> = HashMap::new();
        let mut body = None;
        let mut in_headers = true;

        for (idx, ch) in chunks.enumerate() {
            if idx == 0 {
                let parts: Vec<&str> = ch.split(' ').collect();
                method = parts.first().map(|s| s.to_string());
                target_path = if parts
                    .get(1)
                    .is_some_and(|s| s.starts_with("/") && !s.strip_prefix("/").unwrap().is_empty())
                {
                    Some(parts.get(1).unwrap().to_string())
                } else {
                    None
                };
                http_version = parts.get(2).map(|s| s.to_string());
            } else if in_headers {
                if ch.is_empty() {
                    in_headers = false;
                } else if let Some((key, value)) = ch.split_once(": ") {
                    headers.insert(key.to_string(), value.to_string());
                }
            } else {
                body = Some(ch.to_string());
                break;
            }
        }

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
        let raw = "GET /index.html HTTP/1.1\r\nHost: localhost\r\nUser-Agent: foobar/1.2.3\r\n\r\nHello body";
        let req = parse_request(raw);

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
        let raw = "GET /index.html HTTP/1.1\r\nHost: localhost\r\nUser-Agent: foobar/1.2.3\r\nAccept: */*\r\n\r\n";
        let req = parse_request(raw);

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
        let raw = "POST /submit HTTP/1.1";
        let req = parse_request(raw);

        assert_eq!(req._method.as_deref(), Some("POST"));
        assert_eq!(req._target_path.as_deref(), Some("/submit"));
        assert_eq!(req._http_version.as_deref(), Some("HTTP/1.1"));
        assert!(req._headers.is_empty());
        assert!(req._body.is_none());
    }

    #[test]
    fn handles_malformed_request_line() {
        let raw = "BADREQUEST\r\nHost: localhost";
        let req = parse_request(raw);

        assert_eq!(req._method.as_deref(), Some("BADREQUEST"));
        assert!(req._target_path.is_none());
        assert!(req._http_version.is_none());
    }

    #[test]
    fn handles_empty_input() {
        let req = parse_request("");

        assert_eq!(req._method.as_deref(), Some(""));
        assert!(req._target_path.is_none());
        assert!(req._http_version.is_none());
        assert!(req._headers.is_empty());
        assert!(req._body.is_none());
    }

    #[test]
    fn root_path_yields_no_target_path() {
        let raw = "GET / HTTP/1.1\r\nHost: localhost";
        let req = parse_request(raw);

        assert_eq!(req._method.as_deref(), Some("GET"));
        assert!(req._target_path.is_none());
        assert_eq!(req._http_version.as_deref(), Some("HTTP/1.1"));
    }

    #[test]
    fn path_without_leading_slash_yields_no_target_path() {
        let raw = "GET noslash HTTP/1.1";
        let req = parse_request(raw);

        assert_eq!(req._method.as_deref(), Some("GET"));
        assert!(req._target_path.is_none());
    }
}
