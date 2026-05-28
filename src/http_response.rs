use std::collections::HashMap;

pub(crate) struct HttpResponse {
    pub http_version: String,
    pub status: String,
    pub headers: HashMap<String, String>,
    pub body: Option<Vec<u8>>,
}

impl Default for HttpResponse {
    fn default() -> Self {
        Self {
            http_version: "HTTP/1.1".to_string(),
            status: "200 OK".to_string(),
            headers: HashMap::new(),
            body: None,
        }
    }
}

impl HttpResponse {
    pub fn get_response(&self) -> Vec<u8> {
        // for more complicated resposnes.
        let headers = self
            .headers
            .iter()
            .map(|(k, v)| format!("{}: {}", k, v))
            .collect::<Vec<_>>()
            .join("\r\n");

        let head = if headers.is_empty() {
            format!("{} {}\r\n\r\n", self.http_version, self.status)
        } else {
            format!(
                "{} {}\r\n{}\r\n\r\n",
                self.http_version, self.status, headers
            )
        };

        let mut response = head.into_bytes();
        if let Some(body) = &self.body {
            response.extend(body);
        }
        response
    }
}
