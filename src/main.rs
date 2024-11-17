use http_server::HttpServer;

fn main() {
    let server = HttpServer::new("127.0.0.1:8080", 4);
    server.run().unwrap();
}
