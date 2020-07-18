use std::collections::HashMap;
use rustalot::{HTTPRequest, start_server, Handler, HTTP_200, RouteResult};


fn index(_request: &HTTPRequest) -> RouteResult {
    Ok(("<html><h1>IT WORKS!!!</h1></html>".to_string(), HTTP_200))
}

fn params(request: &HTTPRequest) -> RouteResult {
    for (key, value) in &request.params {
        println!("{}: {}", key, value);
    }
    let opt = request.params.get("test");
    if opt.is_none() {
        return Err("")?;
    }
    println!("Made it here");
    Ok((format!("<html><p><b>Test param:</b>{}</p></html>", opt.unwrap().to_string()), HTTP_200))
}

fn error(_request: &HTTPRequest) -> RouteResult {
    Err("")?
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
