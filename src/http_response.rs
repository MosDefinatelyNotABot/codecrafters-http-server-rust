pub(crate) struct HttpResponse {
    pub http_version: String,
    pub status: String,
    pub headers: Vec<(String, String)>,
    pub body: Option<String>,
}

impl Default for HttpResponse {
    fn default() -> Self {
        Self {
            http_version: "HTTP/1.1".to_string(),
            status: "200 OK".to_string(),
            headers: Vec::new(),
            body: None,
        }
    }
}

impl HttpResponse {
    pub fn get_response(&self) -> String {
        // for more complicated resposnes.
        let headers = self
            .headers
            .iter()
            .map(|(k, v)| format!("{}: {}", k, v))
            .collect::<Vec<_>>()
            .join("\r\n");
        format!(
            "{} {}\r\n{}\r\n\r\n{}",
            self.http_version,
            self.status,
            headers,
            self.body.as_deref().unwrap_or_default()
        )
    }
}
