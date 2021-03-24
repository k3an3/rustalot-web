# rustalot-web
A standards-ignorant, simple, barebones HTTP server framework, written in Rust. Written while I was learning more about Rust.

Usage:
```rust

fn index(request: &HTTPRequest, mut response: HTTPResponse) -> HttpResult {
    /// Example route handler. 
    let ut = Command::new("/usr/bin/uptime").output()?;
    // Set the response body like this. A string is fine, or json can be used with 3rd party libraries.
    response.body = json!({
        "result": String::from_utf8(ut.stdout).unwrap()[1..].replace("\n", "")
    }).to_string();
    Ok(response)
}

fn do_something(request: &HTTPRequest, mut response: HTTPResponse) -> HTTPResult {
    if request.method == "POST" {
        // access request.params, request.headers, request.data, and others
        // ... your logic here
        return Ok(response);
    }
    gen_http_error(HTTP_405, &mut response);
    Ok(response)
}

fn serve_static_asset(request: &HTTPRequest, mut response: HTTPResponse) -> HTTPResult {
    /// Simple route handler to return static assets
    let p = Asset::get(&request.path[1..]);
    if !p.is_some() {
        return router_404(request, response);
    }
    response.body = from_utf8(p.unwrap().as_ref()).unwrap().to_string();
    Ok(response)
}


fn main() {
    let bind_addr = "127.0.0.1";
    let bind_port = 8000;

    // initialize the server with bind address/port
    let mut server = HTTPServer::new(bind_addr.to_string(), bind_port);

    // add routes to call your own handlers
    server.add_route("/".to_string(), index);
    server.add_route("/something".to_string(), do_something);
    // regexes allowed
    server.add_route("/.*\\.(html|js|css|py)".to_string(), serve_static_asset);
    println!("Starting server on {}:{}...", bind_addr, bind_port);
    server.start_server().expect("Something bad happened; bailing!!!");
    println!("Shutting down...");
}
```
