pub mod request;
pub mod response;
use request::Request;
use response::Response;
use std::{
    collections::HashMap,
    io::{BufReader, Write},
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
            let stream = stream.unwrap();
            let routes = Arc::clone(&self.routes);
            thread::spawn(move || {
                handle_connection(stream, routes);
            });
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

    let request = Request::from(&mut buf_reader);

    let mut response = Response::new();
    let handler: HandlerFn;
    {
        let routes = routes.lock().unwrap();
        handler = *routes
            .get(&(request.method.to_string(), request.path.to_string()))
            .unwrap();
    }

    handler(&request, &mut response);

    send_response(response, &mut stream);
}

fn send_response(mut res: Response, stream: &mut TcpStream) {
    res.send(res.body.clone()).unwrap();

    let res_string = format!(
        "HTTP/1.1 {}\r\n{}\r\nContent-Length: {}\r\n\r\n{}\r\n",
        res.status,
        res.headers
            .iter()
            .map(|(k, v)| format!("{}: {}", k, v))
            .collect::<Vec<String>>()
            .join("\r\n"),
        res.body.len(),
        String::from_utf8_lossy(&res.body)
    );

    if res.sent {
        stream.write_all(res_string.as_bytes()).unwrap();
    }
}
