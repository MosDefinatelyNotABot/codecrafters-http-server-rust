mod compression_utils;
mod file_utils;
mod http_request;
mod http_response;
mod path_splitter;
mod route_handlers;

use file_utils::{DIR_PATH, get_dir_path};
use http_request::{HttpRequest, fetch_request_str};
use path_splitter::path_spilter;
use route_handlers::ROUTES;

use tokio::io::{AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};

use std::env;
use std::pin::Pin;

use crate::http_response::HttpResponse;

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
    // handles a single client connection
    let client_addr = stream.peer_addr().unwrap();
    println!("[main] Accepted connection from: {}", client_addr);

    let (reader, mut writer) = tokio::io::split(stream);
    let mut reader = BufReader::new(reader);

    loop {
        // read request into strings
        let (request_line, headers_buffer, body): (String, String, Option<String>) =
            fetch_request_str(Pin::new(&mut reader)).await;

        // empty request line means client closed the connection
        if request_line.is_empty() {
            println!("[main] Client closed the connection from: {}", client_addr);
            break;
        }

        // parse request
        let request: HttpRequest = HttpRequest::parse_request(&request_line, &headers_buffer, body);

        println!("[main] Request headers: {:?}", request._headers);
        println!("[main] Request headers: {:?}", request._target_path);

        let close_after_response = request
            ._headers
            .get("Connection")
            .is_some_and(|v| v.eq_ignore_ascii_case("close"));

        // get response
        let mut http_response: HttpResponse = match request._target_path {
            Some(ref path) => {
                let (base_path, path_args) = path_spilter(path).unwrap_or_default();
                println!(
                    "[main] base: {:?} --- path_args: {:?}",
                    base_path, path_args
                );
                let method = request._method.as_deref().unwrap_or("GET").to_string();
                if ROUTES.contains_key(&(base_path.to_owned(), method.to_owned())) {
                    ROUTES
                        .get(&(base_path, method))
                        .expect("Could not find handler")(&request)
                } else {
                    ROUTES
                        .get(&("/error".to_string(), "GET".to_string()))
                        .expect("Could not find error handler")(&request)
                }
            }
            None => ROUTES
                .get(&("/root".to_string(), "GET".to_string()))
                .expect("Could not find root handler")(&request),
        };

        // compress body if necessary
        let client_compression_support: Vec<&str> = request
            ._headers
            .get("Accept-Encoding")
            .map(|v| v.as_str())
            .unwrap_or("")
            .split(", ")
            .collect::<Vec<&str>>();

        let compression_method: Option<&str> = client_compression_support
            .iter()
            .find(|m| compression_utils::COMPRESSION_METHODS.contains_key(**m))
            .cloned();

        // println!("[main] compression method: {:?}", compression_method);

        if let Some(compression_method) = compression_method {
            // println!("[main] compressing body");

            http_response.body = Some(compression_utils::compress_data(
                http_response.body.as_deref().unwrap_or(&[]),
                Some(compression_method),
            ));

            // add content encoding header
            http_response.headers.insert(
                "Content-Encoding".to_string(),
                compression_method.to_owned(),
            );

            // update content length header
            let compressed_content_length: usize = http_response
                .body
                .as_ref()
                .expect("compressed body is None")
                .len();

            http_response.headers.insert(
                "Content-Length".to_string(),
                compressed_content_length.to_string(),
            );
        }

        println!("[main] response headers: {:?}", http_response.headers);

        if close_after_response {
            // close the connection with client
            println!(
                "[main] Closing connection after response from: {}",
                &client_addr
            );
            http_response
                .headers
                .insert("Connection".to_string(), "close".to_string());

            // send response
            writer
                .write_all(http_response.get_response().as_slice())
                .await
                .expect("Error writing response. :(");

            break;
        } else {
            // send response and keep connection open
            writer
                .write_all(http_response.get_response().as_slice())
                .await
                .expect("Error writing response. :(");
        }
    }
}
