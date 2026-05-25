mod http_request;
mod http_response;
mod path_splitter;
mod route_handlers;

use http_request::HttpRequest;
use path_splitter::path_spilter;
use route_handlers::{
    RequestHandler, echo_handler, error_handler, root_handler, user_agent_handler,
};

use std::collections::HashMap;
use std::sync::Arc;

use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};

#[tokio::main]
async fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    // route mapper. maps a path to a handler function.
    let routes: Arc<HashMap<String, RequestHandler>> = Arc::new(
        [
            (
                "/root".to_string(),
                Box::new(root_handler) as RequestHandler,
            ),
            (
                "/error".to_string(),
                Box::new(error_handler) as RequestHandler,
            ),
            (
                "/echo".to_string(),
                Box::new(echo_handler) as RequestHandler,
            ),
            (
                "/user-agent".to_string(),
                Box::new(user_agent_handler) as RequestHandler,
            ),
        ]
        .into_iter()
        .collect(),
    );

    let listener = TcpListener::bind("127.0.0.1:4221").await.unwrap();

    loop {
        let (stream, _) = listener.accept().await.unwrap();
        let routes = Arc::clone(&routes);
        tokio::spawn(async move {
            handle_connection(stream, routes).await;
        });
    }
}

async fn handle_connection(stream: TcpStream, routes: Arc<HashMap<String, RequestHandler>>) {
    // run for each connection to server
    let (reader, mut writer) = tokio::io::split(stream);
    let mut reader = BufReader::new(reader);
    let mut request_buffer = String::new();

    // read request line by line. Asynchronously
    loop {
        let mut line = String::new();
        reader
            .read_line(&mut line)
            .await
            .expect("Error reading request. :(");
        if line == "\r\n" || line.is_empty() {
            break;
        }
        request_buffer.push_str(&line);
    }

    // parse request into HttpRequest struct
    let request = HttpRequest::parse_request(&request_buffer);

    match request._target_path {
        Some(ref path) => {
            // check if path is valid
            let (base_path, _path_chunks) = path_spilter(path).unwrap_or_default();
            // println!("base_path: {}, path_chunks: {:?}", base_path, _path_chunks);

            if routes.contains_key(&base_path) {
                // path is valid
                let response =
                    routes
                        .get(&base_path)
                        .expect("Could not find handler function")(&request);

                writer
                    .write_all(format!("{}\r\n", response).as_bytes())
                    .await
                    .expect("Error writing response. :(");
            } else {
                // path is not valid
                let err_response =
                    routes
                        .get("/error")
                        .expect("Could not find handler function")(&request);
                writer
                    .write_all(format!("{}\r\n", err_response).as_bytes())
                    .await
                    .expect("Error writing response. :(");
            }
        }
        None => {
            // no target path specified.
            let health_check_resposne = routes[&"/root".to_string()](&request);
            writer
                .write_all(format!("{}\r\n", health_check_resposne).as_bytes())
                .await
                .expect("Error writing response. :(");
        }
    };
}
