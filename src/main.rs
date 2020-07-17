use std::collections::HashMap;
use crate::server::{HTTPRequest, start_server, Handler};

mod server;

fn index(request: HTTPRequest, request_body: String) {

}

fn main() {
    let mut router: HashMap<String, Handler> = HashMap::new();
    router.insert("/".to_string(), index);
    println!("Starting server...");
    start_server("127.0.0.1", 8080, &router).expect("Something bad happened.");
    println!("Shutting down...");
}
