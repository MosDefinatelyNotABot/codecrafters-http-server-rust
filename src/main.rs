mod file_utils;
mod http_request;
mod http_response;
mod path_splitter;
mod route_handlers;

use file_utils::{DIR_PATH, get_dir_path};
use http_request::HttpRequest;
use path_splitter::path_spilter;
use route_handlers::ROUTES;

use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};

use std::env;

#[tokio::main]
async fn main() {
    println!("Logs from your program will appear here!");

    // set directory path
    let args: Vec<String> = env::args().collect();
    let raw_path = args
        .windows(2)
        .find(|w| w[0] == "--directory")
        .map(|w| w[1].clone())
        .unwrap_or_else(|| "not set".to_string());
    if let Some(path) = get_dir_path(&args) {
        DIR_PATH.set(path).unwrap();
    }

    println!("Directory Path set to: {}", raw_path);

    let listener = TcpListener::bind("127.0.0.1:4221").await.unwrap();

    loop {
        let (stream, _) = listener.accept().await.unwrap();
        tokio::spawn(async move {
            handle_connection(stream).await;
        });
    }
}

async fn handle_connection(stream: TcpStream) {
    println!("Accepted connection from: {}", stream.peer_addr().unwrap());

    let (reader, mut writer) = tokio::io::split(stream);
    let mut reader = BufReader::new(reader);
    let mut request_buffer = String::new();

    loop {
        let mut line = String::new();
        reader
            .read_line(&mut line)
            .await
            .expect("Error reading request. :(");
        if line == "\r\n" || line.is_empty() {
            request_buffer.push_str(&line);
            break;
        }
        request_buffer.push_str(&line);
    }

    // read body if Content-Length header is present
    let content_length: usize = request_buffer
        .lines()
        .find_map(|l| l.strip_prefix("Content-Length: "))
        .and_then(|v| v.trim().parse().ok())
        .unwrap_or(0);

    if content_length > 0 {
        let mut body_buf = vec![0u8; content_length];
        reader
            .read_exact(&mut body_buf)
            .await
            .expect("Error reading body. :(");
        request_buffer.push_str(&String::from_utf8_lossy(&body_buf));
    }

    let request: HttpRequest = HttpRequest::parse_request(&request_buffer);

    match request._target_path {
        Some(ref path) => {
            let (base_path, _path_chunks) = path_spilter(path).unwrap_or_default();
            let method = request._method.as_deref().unwrap_or("GET").to_string();

            let response = if ROUTES.contains_key(&(base_path.to_owned(), method.to_owned())) {
                ROUTES
                    .get(&(base_path, method))
                    .expect("Could not find handler")(&request)
            } else {
                ROUTES
                    .get(&("/error".to_string(), "GET".to_string()))
                    .expect("Could not find error handler")(&request)
            };

            writer
                .write_all(format!("{}\r\n", response).as_bytes())
                .await
                .expect("Error writing response. :(");
        }
        None => {
            let response = ROUTES
                .get(&("/root".to_string(), "GET".to_string()))
                .expect("Could not find root handler")(&request);
            writer
                .write_all(format!("{}\r\n", response).as_bytes())
                .await
                .expect("Error writing response. :(");
        }
    }
}
