use std::collections::HashMap;
use rustalot::{HTTPRequest, start_server, Handler, HTTP_200, RouteResult, gen_http_error, HTTP_405};


fn index(_request: &HTTPRequest) -> RouteResult {
    Ok(("<html><h1>IT WORKS!!!</h1></html>".to_string(), HTTP_200))
}

fn get(request: &HTTPRequest) -> RouteResult {
    if request.method == "GET" {
        let opt = request.params.get("test");
        if opt.is_none() {
            return Err("")?;
        }
        return Ok((format!("<html><p><b>Test param:</b>{}</p></html>", opt.unwrap().to_string()), HTTP_200));
    }
    Ok((gen_http_error(HTTP_405), HTTP_405))
}

fn post(request: &HTTPRequest) -> RouteResult {
    if request.method == "POST" {
        let opt = request.data.get("test");
        if opt.is_none() {
            return Err("")?;
        }
        return Ok((format!("<html><p><b>Test param:</b>{}</p></html>", opt.unwrap().to_string()), HTTP_200));
    }
    return Ok((gen_http_error(HTTP_405), HTTP_405))
}

fn error(_request: &HTTPRequest) -> RouteResult {
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
