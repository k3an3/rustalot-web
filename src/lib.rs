use boolinator::Boolinator;
use regex::Regex;
use lazy_static::lazy_static;

use std::net::{TcpStream, TcpListener};
use std::io::{Read, Write};
use std::net::Shutdown::Both;
use std::collections::HashMap;
use std::error::Error;

pub use crate::util::{fmt_http_error, load_static_file, gen_http_error, walk_params, split_string, get_client_addr, log};

mod util;


pub const HTTP_404: HTTPStatus = (404, "File Not Found");
pub const HTTP_405: HTTPStatus = (405, "Method Not Supported");
pub const HTTP_400: HTTPStatus = (400, "Bad Request");
pub const HTTP_200: HTTPStatus = (200, "OK");
pub const HTTP_500: HTTPStatus = (500, "Internal Server Error");

pub type HTTPResult = Result<HTTPResponse, Box<dyn Error>>;
pub type Handler = fn(&HTTPRequest, HTTPResponse) -> HTTPResult;
pub type HTTPStatus = (u16, &'static str);


pub struct HTTPRequest {
    pub method: String,
    pub path: String,
    pub proto: String,
    pub headers: HashMap<String, String>,
    pub params: HashMap<String, String>,
    pub data: HashMap<String, String>,
    pub valid: bool,
}

pub struct HTTPResponse {
    pub status: HTTPStatus,
    pub headers: HashMap<String, String>,
    pub body: String,
}

pub struct HTTPServer {
    pub bind_addr: String,
    pub bind_port: u16,
    pub routes: Vec<(Regex, Handler)>
}

impl HTTPServer {
    pub fn new(bind_addr: String, bind_port: u16) -> HTTPServer {
        let mut h = HTTPServer{bind_addr, bind_port, routes: vec![]};
        h.add_route("/static/.*".to_string(), serve_static);
        h
    }

    pub fn add_route(&mut self, path: String, func: Handler) {
        self.routes.push((Regex::new(&format!("^{}$", path)).expect(&format!("Invalid regular expression: {}", path)), func));
    }

    pub fn start_server(&mut self) -> std::io::Result<()> {
        let bind = format!("{}:{}", self.bind_addr, self.bind_port);
        let listener = TcpListener::bind(bind).expect("Failed to bind!");

        let pool = threadpool::Builder::new()
            .thread_name("http_worker".into())
            .num_threads(5)
            .build();

        for stream in listener.incoming() {
            let routes_copy = self.routes.clone();
            pool.execute(move|| {
                handle_request(stream.unwrap(), routes_copy);
            });
        }
        Ok(())
    }
}

pub fn serve_static(request: &HTTPRequest, mut response: HTTPResponse) -> HTTPResult {
    // Builtin request handler that will handle any requests for static files.
    response.body = load_static_file(&request.path[1..]).unwrap_or_else(|_| {
        response.status = HTTP_404;
        return fmt_http_error(HTTP_404);
    });
    Ok(response)
}


pub fn router(request: &HTTPRequest, routes: Vec<(Regex, Handler)>) -> HTTPResponse {
    let mut response = HTTPResponse::new();

    return routes.iter().find_map(|x| x.0.is_match(&request.path).as_some(x.1)).unwrap_or_else(|| {
        router_404 as fn(&HTTPRequest, HTTPResponse) -> HTTPResult
    })(request, HTTPResponse::new()).unwrap_or_else(|_| {
        gen_http_error(HTTP_500, &mut response);
        response
    })
}


impl HTTPRequest {
    pub fn new(method: String, path: String, proto: String) -> HTTPRequest {
        HTTPRequest {
            method,
            path,
            proto,
            headers: HashMap::new(),
            params: HashMap::new(),
            data: HashMap::new(),
            valid: true,
        }
    }
}

impl HTTPResponse {
    pub fn new() -> HTTPResponse {
        HTTPResponse{status: HTTP_200, headers: HashMap::new(), body: String::new()}
    }

    fn send(&mut self, mut stream: &TcpStream) -> Result<(), ()> {
        self.headers.insert("Server".to_string(), "Rustalot/0.1.0".to_string());
        self.headers.insert("Content-Length".to_string(), self.body.len().to_string());
        let mut data = format!("HTTP/1.1 {} {}\r\n", self.status.0, self.status.1);
        for (key, value) in &self.headers {
            data.push_str(&format!{"{}: {}\r\n", key, value});
        }
        data.push_str("\r\n");
        data.push_str(&self.body);
        stream.write(data.as_bytes()).expect("Failed to write HTTP response.");
        stream.flush().unwrap();
        Ok(())
    }
}

lazy_static! {
    static ref HTTP_VERSION_REGEX: Regex = Regex::new(r"^HTTP/[\d]\.[\d]$").unwrap();
}

pub fn router_404(_request: &HTTPRequest, mut response: HTTPResponse) -> HTTPResult {
    gen_http_error(HTTP_404, &mut response);
    Ok(response)
}

pub fn handle_request(mut stream: TcpStream, routes: Vec<(Regex, Handler)>) {
    let mut buf= [0u8; 4096];
    stream.read(&mut buf).unwrap();
    let request_buf = String::from_utf8_lossy(&buf);
    let first_line = request_buf.lines().next().unwrap();
    let mut code = HTTP_200;

    let mut request = validate_request(first_line).unwrap_or_else(|_| {
        handle_error(&stream, HTTP_400);
        code = HTTP_400;
        let mut r = HTTPRequest::new("".to_string(), "".to_string(), "".to_string());
        r.valid = false;
        r
    });
    let body_offset = parse_headers(request_buf.to_string(), &mut request);
    let request_body = request_buf.lines().collect::<Vec<&str>>()[body_offset..].join("");
    parse_request(&mut request,&request_body);

    let mut response = router(&request, routes);
    log(get_client_addr(&stream), first_line, response.status, &request.headers.get("user-agent").unwrap().to_string());
    if request.valid {
        response.send(&stream).expect("Failed to send response body.");
    }
    stream.shutdown(Both).expect("Failed to close socket.");
}


pub fn validate_request(line: &str) -> Result<HTTPRequest, String> {
    let v: Vec<&str> = line.split_whitespace().collect();
    let request = HTTPRequest::new(v[0].to_string(), v[1].to_string(), v[2].to_string());
    if !vec!["GET", "POST"].iter().any(|x| x == &request.method) {
        return Err(format!("Invalid request method `{}'", &request.method));
    }
    if !HTTP_VERSION_REGEX.is_match(&request.proto) {
        return Err(format!("Invalid HTTP version `{}'", &request.proto));
    }
    if !&request.path.starts_with("/") {
        return Err(format!("Invalid request path `{}'", &request.path));
    }
    Ok(request)
}


fn handle_error(stream: &TcpStream, err: HTTPStatus) {
    let mut response = HTTPResponse::new();
    gen_http_error(err, &mut response);
    response.send(&stream).expect("Failed to send error response.");
}

pub fn parse_headers(request_buf: String, request: &mut HTTPRequest) -> usize {
    for (i, line) in request_buf.lines().enumerate() {
        if i == 0 {
            continue;
        } else if line.len() == 0 {
            return i+1
        }
        let s: Vec<&str> = line.split(": ").collect();
        request.headers.insert(s[0].to_lowercase(), s[1].to_string());
    }
    0
}

fn parse_request(request: &mut HTTPRequest, body: &str) {
    let mut path = request.path.clone();
    let mut params = "";
    if request.path.find("?").is_some() {
        path = split_string(&request.path, "?", 0).to_string();
        params = split_string(&request.path, "?", 1);
    }
    if params.find("#").is_some() {
        params = split_string(params, "#", 0) ;
    }
    walk_params(params, &mut request.params);
    walk_params(body, &mut request.data);
    request.path = path
}


