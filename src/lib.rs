pub mod request;
pub mod response;
use request::Request;
use response::Response;
use std::{
    collections::HashMap,
    io::{BufRead, BufReader, Write},
    net::{TcpListener, TcpStream},
    sync::{Arc, Mutex},
    thread,
};

pub struct HttpServer {
    address: String,
    routes: Arc<Mutex<HashMap<(String, String), HandlerFn>>>,
    // template_dir: String,
}

type HandlerFn = fn(&Request, &mut Response);

impl HttpServer {
    pub fn new(address: &str) -> Self {
        Self {
            address: address.to_string(),
            routes: Arc::new(Mutex::new(HashMap::new())),
            // template_dir: String::new(),
        }
    }

    pub fn listen(&self) {
        let listener = TcpListener::bind(&self.address).unwrap();

        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    let routes = Arc::clone(&self.routes);

                    thread::spawn(move || {
                        handle_connection(stream, routes);
                    });
                }
                Err(e) => println!("Error: {}", e),
            }
        }
    }

    pub fn get(&mut self, path: &str, handler: HandlerFn) {
        self.routes
            .lock()
            .unwrap()
            .insert(("GET".to_string(), path.to_string()), handler);
    }
}

fn handle_connection(
    mut stream: TcpStream,
    routes: Arc<Mutex<HashMap<(String, String), HandlerFn>>>,
) {
    let mut buf_reader = BufReader::new(&stream);
    let mut buffer = String::new();

    let bytes = buf_reader.read_line(&mut buffer).unwrap();
    if bytes == 0 {
        return;
    }

    let request = Request::from(&buffer);
    println!("{:?}", request);

    let mut response = Response::new();

    let routes = routes.lock().unwrap();
    let handler = routes
        .get(&(request.method.to_string(), request.path.to_string()))
        .unwrap();

    handler(&request, &mut response);

    response.send(response.body.clone()).unwrap();

    let response_string = format!(
        "HTTP/1.1 {}\r\n{}\r\nContent-Length: {}\r\n\r\n{}",
        response.status,
        response
            .headers
            .iter()
            .map(|(k, v)| format!("{}: {}", k, v))
            .collect::<Vec<String>>()
            .join("\r\n"),
        response.body.len(),
        String::from_utf8_lossy(&response.body)
    );

    if response.sent {
        stream.write_all(response_string.as_bytes()).unwrap();
    }
}
