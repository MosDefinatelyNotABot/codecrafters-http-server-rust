enum HttpHeader {
    HttpVersion(String),
    StatusCode(u16, String),
}

enum HttpResponse {
    Header(HttpHeader),
    Body(String),
}

pub(crate) fn HttpOK() -> HttpResponse {
    HttpResponse::Header(HttpHeader::StatusCode(200, "OK".to_string()))
}
