mod http_request;
mod http_response;
mod path_splitter;

use http_request::{HttpRequest, parse_request};
use path_splitter::path_spilter;

use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;

use crate::http_response::HttpResponse;

fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    // route mapper. maps a path to a handler function.
    let valid_routes: HashMap<String, Box<dyn Fn(&String) -> String>> = [
        (
            "/root".to_string(),
            Box::new(root_handler) as Box<dyn Fn(&String) -> String>,
        ),
        (
            "/echo".to_string(),
            Box::new(echo_handler) as Box<dyn Fn(&String) -> String>,
        ),
        (
            "/error".to_string(),
            Box::new(error_handler) as Box<dyn Fn(&String) -> String>,
        ),
    ]
    .into_iter()
    .collect();

    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                println!("accepted new connection");

                let mut reader = BufReader::new(&mut stream);
                let mut request_buffer = String::new();

                loop {
                    let mut line = String::new();
                    reader
                        .read_line(&mut line)
                        .expect("Error reading request. :(");
                    if line == "\r\n" || line.is_empty() {
                        break;
                    }
                    request_buffer.push_str(&line);
                }

                let request: HttpRequest = parse_request(&request_buffer);

                println!(
                    "request: [\"{}\", \"{}\", \"{}\", \"{}\", \"{}\"]",
                    request.method.as_deref().unwrap_or(""),
                    request.target_path.as_deref().unwrap_or(""),
                    request._http_version.as_deref().unwrap_or(""),
                    request.headers.as_deref().unwrap_or(""),
                    request.body.as_deref().unwrap_or(""),
                );

                match request.target_path {
                    Some(path) => {
                        // check if path is valid
                        let (base_path, _path_chunks) = path_spilter(&path).unwrap_or_default();
                        println!("base_path: {}, path_chunks: {:?}", base_path, _path_chunks);

                        if valid_routes.contains_key(&base_path) {
                            // path is valid
                            let response = valid_routes
                                .get(&base_path)
                                .expect("Could not find handler function")(
                                &path
                            );

                            stream
                                .write_all(format!("{}\r\n", response).as_bytes())
                                .expect("Error writing response. :(");
                        } else {
                            // path is not valid
                            let err_response =
                                valid_routes[&"/error".to_string()](&"/error".to_string());
                            stream
                                .write_all(format!("{}\r\n", err_response).as_bytes())
                                .expect("Error writing response. :(");
                        }
                    }
                    None => {
                        // no target path specified.
                        //
                        let health_check_resposne =
                            valid_routes[&"/root".to_string()](&"/root".to_string());
                        stream
                            .write_all(format!("{}\r\n", health_check_resposne).as_bytes())
                            .expect("Error writing response. :(");
                    }
                };
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}

fn echo_handler(path: &String) -> String {
    let (_base_path, path_chunks) = path_spilter(path).unwrap_or_default();

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

fn error_handler(_: &String) -> String {
    "HTTP/1.1 404 Not Found\r\n\r\n".to_string()
}

fn root_handler(_: &String) -> String {
    // doubles as a health check
    "HTTP/1.1 200 OK\r\n\r\nHealthy".to_string()
}
