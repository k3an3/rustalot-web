use std::collections::HashMap;
use rustalot::{HTTPRequest, start_server, Handler, HTTPResult, gen_http_error, HTTP_405, HTTPResponse};


fn index(_request: &HTTPRequest) -> HTTPResult {
    let mut response = HTTPResponse::new();
    response.body = "<html><h1>IT WORKS!!!</h1></html>".to_string();
    Ok(response)
}

fn get(request: &HTTPRequest) -> HTTPResult {
    let mut response = HTTPResponse::new();
    if request.method == "GET" {
        let opt = request.params.get("test");
        if opt.is_none() {
            return Err("")?;
        }
        response.body = format!("<html><p><b>Test param:</b>{}</p></html>", opt.unwrap().to_string());
        return Ok(response);
    }
    gen_http_error(HTTP_405, &mut response);
    Ok(response)
}

fn post(request: &HTTPRequest) -> HTTPResult {
    let mut response = HTTPResponse::new();
    if request.method == "POST" {
        let opt = request.data.get("test");
        if opt.is_none() {
            return Err("")?;
        }
        response.body = format!("<html><p><b>Test param:</b>{}</p></html>", opt.unwrap().to_string());
        return Ok(response);
    }
    gen_http_error(HTTP_405, &mut response);
    Ok(response)
}

fn error(_request: &HTTPRequest) -> HTTPResult {
    Err("")?
}

fn main() {
    let mut router: HashMap<String, Handler> = HashMap::new();
    router.insert("/".to_string(), index);
    router.insert("/error".to_string(), error);
    router.insert("/params".to_string(), get);
    router.insert("/post".to_string(), post);
    println!("Starting server...");
    start_server("127.0.0.1", 8080, router).expect("Something bad happened.");
    println!("Shutting down...");
}
