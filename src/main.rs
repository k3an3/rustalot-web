use std::collections::HashMap;
use crate::server::{HTTPRequest, start_server, Handler, HttpStatus, HTTP_200};

mod server;

fn index(_request: &HTTPRequest) -> (String, HttpStatus){
    ("ITWORKS!!!!".to_string(), HTTP_200)
}

fn main() {
    let mut router: HashMap<String, Handler> = HashMap::new();
    router.insert("/".to_string(), index);
    println!("Starting server...");
    start_server("127.0.0.1", 8080, router).expect("Something bad happened.");
    println!("Shutting down...");
}
