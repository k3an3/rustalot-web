use std::collections::HashMap;
use crate::server::{HTTPRequest, start_server, Handler, HttpStatus, HTTP_200, RouteResult};
use std::error::Error;
use core::fmt;
use std::io::ErrorKind;
use std::sync::mpsc::RecvError;

mod server;

fn index(request: &HTTPRequest) -> RouteResult {
    Ok(("<html><h1>IT WORKS!!!</h1></html>".to_string(), HTTP_200))
}

fn params(request: &HTTPRequest) -> RouteResult {
    Ok((format!("Test param is {}", request.params.get("test").ok_or(0).unwrap().to_string()), HTTP_200))
}

fn error(_request: &HTTPRequest) -> RouteResult {
    Err("Test error")?
}

fn main() {
    let mut router: HashMap<String, Handler> = HashMap::new();
    router.insert("/".to_string(), index);
    router.insert("/error".to_string(), error);
    router.insert("/params".to_string(), params);
    println!("Starting server...");
    start_server("127.0.0.1", 8080, router).expect("Something bad happened.");
    println!("Shutting down...");
}
