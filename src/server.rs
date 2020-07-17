use std::net::{TcpStream, TcpListener};
use std::io::{Read, Write};
use regex::Regex;
use lazy_static::lazy_static;
use std::fs;
use std::net::Shutdown::Both;
use std::collections::HashMap;

pub const ERR_404: (u16, &str) = (404, "File Not Found");
pub const ERR_400: (u16, &str) = (400, "Bad Request");

pub type Handler = fn(HTTPRequest, String);

pub struct HTTPRequest {
    method: String,
    path: String,
    proto: String,
    headers: HashMap<String, String>
}

lazy_static! {
    static ref HTTP_VERSION_REGEX: Regex = Regex::new(r"^HTTP/[\d]\.[\d]$").unwrap();
}

pub fn load_html(name: &str) {
    fs::read_to_string(name).expect("HTML file not found!!!");
}

pub fn router(routes: &HashMap<String, Handler>, request: &HTTPRequest, request_body: String) {
}

pub fn validate_request(line: &str) -> Result<HTTPRequest, String> {
    let v: Vec<&str> = line.split_whitespace().collect();
    let request = HTTPRequest{method: v[0].to_string(), path: v[1].to_string(), proto: v[2].to_string(), headers: HashMap::new()};
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

pub fn http_response(mut stream: &TcpStream, code: u16, status: &str, resp_data: &str) -> Result<(), ()> {
    let mut data = format!("HTTP/1.1 {} {}", code, status);
    data.push_str("\r\nServer: Rustalot/0.1.0\r\n\r\n");
    data.push_str(resp_data);
    stream.write(data.as_bytes()).expect("Failed to write HTTP response.");
    stream.flush().unwrap();
    Ok(())
}

pub fn get_client_addr(stream: &TcpStream) -> String {
    stream.peer_addr().unwrap().to_string()
}

fn log(addr: String, line: &str, code: u16, user_agent: String) {
    println!("{} - \"{}\" {} \"{}\"", addr, line, code, user_agent);
}

fn handle_error(stream: &TcpStream, err: (u16, &str)) {
    http_response(&stream, err.0, err.1, &format!("<html><h1>{}</h1></html>", err.1)).unwrap();
}

pub fn parse_headers(request_buf: String, request: &mut HTTPRequest) -> usize {
    for (i, line) in request_buf.lines().enumerate() {
        if i == 0 {
            continue;
        } else if line.len() == 0 {
            return i+1
        }
        let s: Vec<&str> = line.split(": ").collect();
        request.headers.insert(s[0].to_string(), s[1].to_string());
    }
    0
}

pub fn handle_request(mut stream: TcpStream, routes: &HashMap<String, Handler>) {
    let mut buf= [0u8; 4096];
    stream.read(&mut buf).unwrap();
    let request_buf = String::from_utf8_lossy(&buf);
    let first_line = request_buf.lines().next().unwrap();
    let mut code: u16 = 0;

    let mut request = validate_request(first_line).unwrap_or_else(|_| {
        handle_error(&stream, ERR_400);
        code = 400;
        return HTTPRequest {
            method: "".to_string(),
            path: "".to_string(),
            proto: "".to_string(),
            headers: HashMap::new(),
        };
    });
    let body_offset = parse_headers(request_buf.to_string(), &mut request);
    let request_body = request_buf.lines().collect::<Vec<&str>>()[body_offset..].join("");
    log(get_client_addr(&stream), first_line, code, request.headers.get("User-Agent").unwrap().to_string());
    router(routes, &request, request_body);
    stream.shutdown(Both).unwrap();
    println!("Connection with {} closed.", get_client_addr(&stream));
}

pub fn start_server(bind_addr: &str, port: u16, routes: &HashMap<String, Handler>) -> std::io::Result<()> {
    let mut bind = bind_addr.to_string();
    bind.push_str(&port.to_string());
    let listener = TcpListener::bind(bind).expect("Failed to bind!");

    let pool = threadpool::Builder::new()
    .thread_name("http_worker".into())
    .num_threads(5)
    .build();

    for stream in listener.incoming() {
        let stream = stream.unwrap();
        println!("Got connection from {}.", get_client_addr(&stream));
        pool.execute(move|| {
            handle_request(stream, &routes);
        });
    }
    Ok(())
}

