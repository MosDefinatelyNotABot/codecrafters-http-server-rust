pub(crate) struct HttpRequest {
    pub method: Option<String>,
    pub target_path: Option<String>,
    pub _http_version: Option<String>,
    pub headers: Option<String>,
    pub body: Option<String>,
}

pub(crate) fn parse_request(request: &str) -> HttpRequest {
    // splits reqeust into status, header and body

    let chunks = request.split("\r\n");

    let mut method = None;
    let mut target_path = None;
    let mut http_version = None;
    let mut headers = None;
    let mut body = None;

    for (idx, ch) in chunks.enumerate() {
        match idx {
            0 => {
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
            }
            1 => {
                headers = Some(ch.to_string());
            }
            2 => {
                body = Some(ch.to_string());
            }
            _ => {
                break;
            }
        }
    }

    HttpRequest {
        method,
        target_path,
        _http_version: http_version,
        headers,
        body,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_well_formed_request() {
        let raw = "GET /index.html HTTP/1.1\r\nHost: localhost\r\nHello body";
        let req = parse_request(raw);

        assert_eq!(req.method.as_deref(), Some("GET"));
        assert_eq!(req.target_path.as_deref(), Some("/index.html"));
        assert_eq!(req._http_version.as_deref(), Some("HTTP/1.1"));
        assert_eq!(req.headers.as_deref(), Some("Host: localhost"));
        assert_eq!(req.body.as_deref(), Some("Hello body"));
    }

    #[test]
    fn parses_request_line_only() {
        let raw = "POST /submit HTTP/1.1";
        let req = parse_request(raw);

        assert_eq!(req.method.as_deref(), Some("POST"));
        assert_eq!(req.target_path.as_deref(), Some("/submit"));
        assert_eq!(req._http_version.as_deref(), Some("HTTP/1.1"));
        assert!(req.headers.is_none());
        assert!(req.body.is_none());
    }

    #[test]
    fn handles_malformed_request_line() {
        let raw = "BADREQUEST\r\nHost: localhost";
        let req = parse_request(raw);

        assert_eq!(req.method.as_deref(), Some("BADREQUEST"));
        assert!(req.target_path.is_none());
        assert!(req._http_version.is_none());
    }

    #[test]
    fn handles_empty_input() {
        let req = parse_request("");

        assert_eq!(req.method.as_deref(), Some(""));
        assert!(req.target_path.is_none());
        assert!(req._http_version.is_none());
        assert!(req.headers.is_none());
        assert!(req.body.is_none());
    }

    #[test]
    fn root_path_yields_no_target_path() {
        let raw = "GET / HTTP/1.1\r\nHost: localhost";
        let req = parse_request(raw);

        assert_eq!(req.method.as_deref(), Some("GET"));
        assert!(req.target_path.is_none());
        assert_eq!(req._http_version.as_deref(), Some("HTTP/1.1"));
    }

    #[test]
    fn path_without_leading_slash_yields_no_target_path() {
        let raw = "GET noslash HTTP/1.1";
        let req = parse_request(raw);

        assert_eq!(req.method.as_deref(), Some("GET"));
        assert!(req.target_path.is_none());
    }
}
