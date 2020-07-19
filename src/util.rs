use std::fs;
use std::net::TcpStream;
use percent_encoding::percent_decode_str;
use std::collections::HashMap;
use crate::{HTTPStatus, HTTPResponse};
use std::path::Path;

pub fn gen_http_error(err: HTTPStatus, response: &mut HTTPResponse) {
    response.body = format!("<html><h1>{}</h1></html>", err.1);
    response.status = err;
}

pub fn walk_params(data: &str, map: &mut HashMap<String, String>) {
    for pair in data.split("&") {
        let s: Vec<&str> = pair.split("=").collect();
        if s.len() > 1 {
            map.insert(s[0].to_string(), percent_decode_str(s[1]).decode_utf8_lossy().to_string());
        }
    }
}

pub fn load_html(name: &str) -> String {
    fs::read_to_string(Path::new("templates").join(name)).expect("HTML file not found!!!")
}

pub fn load_static(name: &str) -> String {
    fs::read_to_string(Path::new("static").join(name)).expect("Static file not found!!!")
}

pub fn split_string<'a>(s: &'a str, split: &str, offset: usize) -> &'a str {
     s.split(split).collect::<Vec<&str>>()[offset]
}

pub fn get_client_addr(stream: &TcpStream) -> String {
    stream.peer_addr().unwrap().to_string()
}

pub fn log(addr: String, line: &str, err: HTTPStatus, user_agent: &String) {
    println!("{} - \"{}\" {} \"{}\"", addr, line, err.0, user_agent);
}

