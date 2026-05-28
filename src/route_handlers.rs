use std::{collections::HashMap, fs, sync::LazyLock};

use crate::{
    file_utils::DIR_PATH,
    http_request::HttpRequest,
    http_response::{self, HttpResponse},
    path_splitter::path_spilter,
};

pub(crate) type RequestHandler = Box<dyn Fn(&HttpRequest) -> HttpResponse + Send + Sync>;

// route mapper. maps a path to a handler function.
pub(crate) static ROUTES: LazyLock<HashMap<(String, String), RequestHandler>> =
    LazyLock::new(|| {
        [
            (
                ("/root".to_string(), "GET".to_string()),
                Box::new(root_handler_get) as RequestHandler,
            ),
            (
                ("/error".to_string(), "GET".to_string()),
                Box::new(error_handler_get) as RequestHandler,
            ),
            (
                ("/echo".to_string(), "GET".to_string()),
                Box::new(echo_handler) as RequestHandler,
            ),
            (
                ("/user-agent".to_string(), "GET".to_string()),
                Box::new(user_agent_handler_get) as RequestHandler,
            ),
            (
                ("/files".to_string(), "GET".to_string()),
                Box::new(files_handler_get) as RequestHandler,
            ),
            (
                ("/files".to_string(), "POST".to_string()),
                Box::new(file_handler_post) as RequestHandler,
            ),
        ]
        .into_iter()
        .collect()
    });

fn root_handler_get(request: &HttpRequest) -> HttpResponse {
    println!(
        "[root_handler_get] root path handler returning healthy. request target: {:?}",
        request._target_path
    );

    HttpResponse {
        http_version: "HTTP/1.1".to_string(),
        status: "200 OK".to_string(),
        headers: vec![],
        body: Some("Healthy".to_string()),
    }
}

fn error_handler_get(request: &HttpRequest) -> HttpResponse {
    println!(
        "[error_handler_get] request target: {:?}",
        request._target_path
    );

    HttpResponse {
        http_version: "HTTP/1.1".to_string(),
        status: "404 Not Found".to_string(),
        headers: vec![],
        body: None,
    }
}

fn echo_handler(request: &HttpRequest) -> HttpResponse {
    println!("[echo_handler] request target: {:?}", request._target_path);
    let (_base_path, path_chunks) = path_spilter(
        request
            ._target_path
            .as_ref()
            .expect("No path specified. :("),
    )
    .unwrap_or_default();

    let body = path_chunks.first().cloned().unwrap_or_default();
    println!("[echo_handler] echoing body: {:?}", body);

    HttpResponse {
        http_version: "HTTP/1.1".to_string(),
        status: "200 OK".to_string(),
        headers: vec![
            ("Content-Type".to_string(), "text/plain".to_string()),
            ("Content-Length".to_string(), body.len().to_string()),
        ],
        body: Some(body),
    }
}

fn user_agent_handler_get(request: &HttpRequest) -> HttpResponse {
    println!(
        "[user_agent_handler_get] request target: {:?}",
        request._target_path
    );
    println!("[user_agent_handler_get] headers: {:?}", request._headers);

    if let Some(user_agent) = request._headers.get("User-Agent") {
        println!("[user_agent_handler_get] User-Agent: {:?}", user_agent);

        let http_response = http_response::HttpResponse {
            http_version: "HTTP/1.1".to_string(),
            status: "200 OK".to_string(),
            headers: vec![
                ("Content-Type".to_string(), "text/plain".to_string()),
                ("Content-Length".to_string(), user_agent.len().to_string()),
            ],
            body: Some(user_agent.to_owned()),
        };

        http_response
    } else {
        println!("[user_agent_handler_get] User-Agent header not found, returning 404");
        error_handler_get(request)
    }
}

fn files_handler_get(request: &HttpRequest) -> HttpResponse {
    println!(
        "[files_handler_get] request target: {:?}",
        request._target_path
    );

    // get path arguments
    let (_, path_args) = match request._target_path.as_ref() {
        Some(path) => match path_spilter(path) {
            Ok(result) => result,
            Err(_) => {
                println!("[files_handler_get] failed to parse path, returning 404");
                return error_handler_get(request);
            }
        },
        None => {
            println!("[files_handler_get] no path specified, returning 404");
            return error_handler_get(request);
        }
    };

    // check if dir path is set
    let dir_path = match DIR_PATH.get() {
        Some(result) => result,
        None => {
            println!("[files_handler_get] DIR_PATH not set, returning 404");
            return error_handler_get(request);
        }
    };

    println!("[files_handler_get] path_chunks: {:?}", path_args);
    let requested_fname = match path_args.first() {
        Some(fname) => dir_path.join(fname),
        None => {
            println!("[files_handler_get] no filename in path, returning 404");
            return error_handler_get(request);
        }
    };

    println!(
        "[files_handler_get] full path: {:?}, is_file: {}",
        requested_fname,
        requested_fname.is_file()
    );

    // Now send file the file
    if requested_fname.is_file() {
        let bytes = fs::read(&requested_fname).expect("Error reading file. :(");
        let content_length = bytes.len();
        println!("[files_handler_get] read {} bytes", content_length);

        let http_response = http_response::HttpResponse {
            http_version: "HTTP/1.1".to_string(),
            status: "200 OK".to_string(),
            headers: vec![
                (
                    "Content-Type".to_string(),
                    "application/octet-stream".to_string(),
                ),
                ("Content-Length".to_string(), content_length.to_string()),
            ],
            body: Some(String::from_utf8_lossy(&bytes).into_owned()),
        };

        http_response
    } else {
        println!("[files_handler_get] file not found, returning 404");
        error_handler_get(request)
    }
}

fn file_handler_post(request: &HttpRequest) -> HttpResponse {
    println!(
        "[file_handler_post] request target: {:?}",
        request._target_path
    );
    // get path arguments
    let (_, path_args) = match request._target_path.as_ref() {
        Some(path) => match path_spilter(path) {
            Ok(result) => result,
            Err(_) => {
                println!("[file_handler_post] failed to parse path, returning 404");
                return error_handler_get(request);
            }
        },
        None => {
            println!("[file_handler_post] no path specified, returning 404");
            return error_handler_get(request);
        }
    };

    // check if dir path is set otherwise fail
    let dir_path = match DIR_PATH.get() {
        Some(result) => result,
        None => {
            println!("[file_handler_post] DIR_PATH not set, returning 404");
            return error_handler_get(request);
        }
    };

    // get the file name
    let file_path = match path_args.first() {
        Some(result) => dir_path.join(result),
        None => {
            println!("[file_handler_post] no filename in path, returning 404");
            return error_handler_get(request);
        }
    };

    // make sure request body is none empty
    let file_contents = match request._body.to_owned() {
        Some(result) => result,
        None => {
            println!("[file_handler_post] request body is empty, returning 404");
            return error_handler_get(request);
        }
    };

    println!(
        "[file_handler_post] writing {} bytes to {:?}.",
        file_contents.len(),
        &file_path
    );

    match fs::write(file_path, file_contents) {
        Ok(_) => {
            let http_response = http_response::HttpResponse {
                http_version: "HTTP/1.1".to_string(),
                status: "201 Created".to_string(),
                headers: vec![],
                body: None,
            };

            http_response
        }
        Err(e) => {
            println!(
                "[file_handler_post] failed to write file: {}, returning 404",
                e
            );
            error_handler_get(request)
        }
    }
}
