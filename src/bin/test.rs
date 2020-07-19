use rustalot::{HTTPRequest, HTTPResult, gen_http_error, HTTP_405, HTTPResponse, HTTPServer};


fn index(_request: &HTTPRequest, mut response: HTTPResponse) -> HTTPResult {
    response.body = "<html><h1>IT WORKS!!!</h1></html>".to_string();
    Ok(response)
}

fn get(request: &HTTPRequest, mut response: HTTPResponse) -> HTTPResult {
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

fn post(request: &HTTPRequest, mut response: HTTPResponse) -> HTTPResult {
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

fn error(_request: &HTTPRequest, mut _response: HTTPResponse) -> HTTPResult {
    Err("")?
}

fn regex_test(_request: &HTTPRequest, mut response: HTTPResponse) -> HTTPResult {
    response.body = "VALID".to_string();
    Ok(response)
}

fn main() {
    let mut server = HTTPServer::new("127.0.0.1".to_string(), 8080);
    server.add_route("/".to_string(), index);
    server.add_route("/regex.*".to_string(), regex_test);
    server.add_route("/error".to_string(), error);
    server.add_route("/params".to_string(), get);
    server.add_route("/post".to_string(), post);
    println!("Starting server...");
    server.start_server().expect("Something bad happened.");
    println!("Shutting down...");
}
