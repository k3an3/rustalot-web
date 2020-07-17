use std::net::{TcpStream, TcpListener};
use std::io::{Read, Write};
use regex::Regex;
use lazy_static::lazy_static;
use std::fs;
use std::net::Shutdown::Both;
use std::collections::HashMap;

struct HTTPRequest {
    method: String,
    path: String,
    proto: String,
    headers: HashMap<String, String>
}

lazy_static! {
    static ref HTTP_VERSION_REGEX: Regex = Regex::new(r"^HTTP/[\d]\.[\d]$").unwrap();
}

fn load_html(name: &str) {
    fs::read_to_string(name).expect("HTML file not found!!!");
}

fn router(request: &HTTPRequest) {
    if request.path == "/" {

    }
}

fn validate_request(line: &str) -> Result<HTTPRequest, String> {
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

fn http_response(mut stream: &TcpStream, code: u16, status: &str, resp_data: &str) -> Result<(), ()> {
    let mut data = format!("HTTP/1.1 {} {}", code, status);
    data.push_str("\r\nServer: Rustalot/0.1.0\r\n\r\n");
    data.push_str(resp_data);
    stream.write(data.as_bytes()).expect("Failed to write HTTP response.");
    stream.flush().unwrap();
    Ok(())
}

fn get_client_addr(stream: &TcpStream) -> String {
    stream.peer_addr().unwrap().to_string()
}

fn log(addr: String, line: &str, code: u16, user_agent: String) {
    println!("{} - \"{}\" {} \"{}\"", addr, line, code, user_agent);
}

fn handle_400(stream: &TcpStream) {
    http_response(&stream, 400, "Bad Request", "<html><h1>Bad Request</h1></html>").unwrap();
}

fn parse_headers(request_buf: String, request: &mut HTTPRequest) {
    for (i, line) in request_buf.lines().enumerate() {
        if i == 0 {
            continue;
        } else if line.len() == 0 {
            break
        }
        let s: Vec<&str> = line.split(": ").collect();
        request.headers.insert(s[0].to_string(), s[1].to_string());
    }
}

fn handle_client(mut stream: TcpStream) {
    let mut buf= [0u8; 4096];
    stream.read(&mut buf).unwrap();
    let request_buf = String::from_utf8_lossy(&buf);
    let first_line = request_buf.lines().next().unwrap();
    let mut code: u16 = 0;

    let mut request = validate_request(first_line).unwrap_or_else(|_| {
        handle_400(&stream);
        code = 400;
        return HTTPRequest {
            method: "".to_string(),
            path: "".to_string(),
            proto: "".to_string(),
            headers: HashMap::new(),
        };
    });
    parse_headers(request_buf.to_string(), &mut request);
    log(get_client_addr(&stream), first_line, code, request.headers.get("User-Agent").unwrap().to_string());
    router(&request);
    stream.shutdown(Both).unwrap();
    println!("Connection with {} closed.", get_client_addr(&stream));
}

fn start_server(port: u16) -> std::io::Result<()> {
    let mut bind = String::from("127.0.0.1:");
    bind.push_str(&port.to_string());
    let listener = TcpListener::bind(bind).expect("Failed to bind!");

    let pool = threadpool::Builder::new()
    .thread_name("http_worker".into())
    .num_threads(5)
    .build();

    for stream in listener.incoming() {
        let stream = stream.unwrap();
        println!("Got connection from {}.", get_client_addr(&stream));
        pool.execute(|| {
            handle_client(stream);
        });
    }
    Ok(())
}

fn main() {
    println!("Starting server...");
    start_server(8080).expect("Something bad happened.");
    println!("Shutting down...");
}
