mod http_request;
mod http_response;
mod path_splitter;
mod route_handlers;

use http_request::{HttpRequest, parse_request};
use path_splitter::path_spilter;
use route_handlers::{
    RequestHandler, echo_handler, error_handler, root_handler, user_agent_handler,
};

use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;

fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    // route mapper. maps a path to a handler function.
    let valid_routes: HashMap<String, RequestHandler> = [
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

                // println!(
                //     "request: [\"{}\", \"{}\", \"{}\", \"{}\", \"{}\"]",
                //     request.method.as_deref().unwrap_or(""),
                //     request.target_path.as_deref().unwrap_or(""),
                //     request._http_version.as_deref().unwrap_or(""),
                //     request.headers.as_deref().unwrap_or(""),
                //     request.body.as_deref().unwrap_or(""),
                // );

                match request._target_path {
                    Some(ref path) => {
                        // check if path is valid
                        let (base_path, _path_chunks) = path_spilter(path).unwrap_or_default();
                        println!("base_path: {}, path_chunks: {:?}", base_path, _path_chunks);

                        if valid_routes.contains_key(&base_path) {
                            // path is valid
                            let response = valid_routes
                                .get(&base_path)
                                .expect("Could not find handler function")(
                                &request
                            );

                            stream
                                .write_all(format!("{}\r\n", response).as_bytes())
                                .expect("Error writing response. :(");
                        } else {
                            // path is not valid
                            let err_response = valid_routes
                                .get("/error")
                                .expect("Could not find handler function")(
                                &request
                            );
                            stream
                                .write_all(format!("{}\r\n", err_response).as_bytes())
                                .expect("Error writing response. :(");
                        }
                    }
                    None => {
                        // no target path specified.
                        let health_check_resposne = valid_routes[&"/root".to_string()](&request);
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
