mod http_request;

use http_request::{HttpRequest, parse_request};
use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;

fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    let _valid_routes = HashMap::<String, String>::new();
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
                    Some(_) => stream
                        .write_all(b"HTTP/1.1 404 Not Found\r\n\r\n")
                        .expect("Error writing response. :("),
                    None => stream
                        .write_all(b"HTTP/1.1 200 OK\r\n\r\n")
                        .expect("Error writing response. :("),
                };
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}
