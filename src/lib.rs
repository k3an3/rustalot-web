use std::net::{TcpStream, TcpListener};
use std::io::{Read, Write};
use regex::Regex;
use lazy_static::lazy_static;
use std::net::Shutdown::Both;
use std::collections::HashMap;
use std::error::Error;
pub use crate::util::{gen_http_error, walk_params, split_string, get_client_addr, log, load_html};

mod util;


pub type HttpStatus = (u16, &'static str);

pub const HTTP_404: HttpStatus = (404, "File Not Found");
pub const HTTP_405: HttpStatus = (405, "Method Not Supported");
pub const HTTP_400: HttpStatus = (400, "Bad Request");
pub const HTTP_200: HttpStatus = (200, "OK");
pub const HTTP_500: HttpStatus = (500, "Internal Server Error");

pub type RouteResult = Result<(String, HttpStatus), Box<dyn Error>>;
pub type Handler = fn(&HTTPRequest) -> RouteResult;

pub struct HTTPRequest {
    pub method: String,
    pub path: String,
    pub proto: String,
    pub headers: HashMap<String, String>,
    pub params: HashMap<String, String>,
    pub data: HashMap<String, String>,
}

lazy_static! {
    static ref HTTP_VERSION_REGEX: Regex = Regex::new(r"^HTTP/[\d]\.[\d]$").unwrap();
}

fn router_404(_request: &HTTPRequest) -> RouteResult {
    Ok((gen_http_error(HTTP_404), HTTP_404))
}

pub fn router(routes: &HashMap<String, Handler>, request: &HTTPRequest) -> (String, HttpStatus) {
    return routes.get(&request.path).unwrap_or_else(|| {
        &(router_404 as fn(&HTTPRequest) -> RouteResult)
    })(request).unwrap_or_else(|_| {
        (gen_http_error(HTTP_500), HTTP_500)
    });
}

pub fn validate_request(line: &str) -> Result<HTTPRequest, String> {
    let v: Vec<&str> = line.split_whitespace().collect();
    let request = HTTPRequest{method: v[0].to_string(), path: v[1].to_string(),
        proto: v[2].to_string(), headers: HashMap::new(), params: HashMap::new(),
        data: HashMap::new()};
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

pub fn http_response(mut stream: &TcpStream, status: HttpStatus, resp_data: &str) -> Result<(), ()> {
    let mut data = format!("HTTP/1.1 {} {}", status.0, status.1);
    data.push_str("\r\nServer: Rustalot/0.1.0\r\n\r\n");
    data.push_str(resp_data);
    stream.write(data.as_bytes()).expect("Failed to write HTTP response.");
    stream.flush().unwrap();
    Ok(())
}

fn handle_error(stream: &TcpStream, err: HttpStatus) {
    http_response(&stream, err,&gen_http_error(err)).unwrap();
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

pub fn handle_request(mut stream: TcpStream, routes: HashMap<String, Handler>) {
    let mut buf= [0u8; 4096];
    stream.read(&mut buf).unwrap();
    let request_buf = String::from_utf8_lossy(&buf);
    let first_line = request_buf.lines().next().unwrap();
    let mut code: HttpStatus = HTTP_200;

    let mut request = validate_request(first_line).unwrap_or_else(|_| {
        handle_error(&stream, HTTP_400);
        code = HTTP_400;
        return HTTPRequest {
            method: "".to_string(),
            path: "".to_string(),
            proto: "".to_string(),
            headers: HashMap::new(),
            params: HashMap::new(),
            data: HashMap::new(),
        };
    });
    let body_offset = parse_headers(request_buf.to_string(), &mut request);
    let request_body = request_buf.lines().collect::<Vec<&str>>()[body_offset..].join("");
    parse_request(&mut request,&request_body);

    let (resp_body, code) = router(&routes, &request);
    log(get_client_addr(&stream), first_line, code, &request.headers.get("user-agent").unwrap().to_string());
    http_response(&stream, code, &resp_body).unwrap();

    stream.shutdown(Both).unwrap();
}

pub fn start_server(bind_addr: &str, port: u16, routes: HashMap<String, Handler>) -> std::io::Result<()> {
    let mut bind = bind_addr.to_string();
    bind.push_str(":");
    bind.push_str(&port.to_string());
    let listener = TcpListener::bind(bind).expect("Failed to bind!");

    let pool = threadpool::Builder::new()
    .thread_name("http_worker".into())
    .num_threads(5)
    .build();

    for stream in listener.incoming() {
        let stream = stream.unwrap();
        let routes_copy = routes.clone();
        pool.execute(move|| {
            handle_request(stream, routes_copy);
        });
    }
    Ok(())
}