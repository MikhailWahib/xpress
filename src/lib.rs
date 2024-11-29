pub mod request;
pub mod response;
mod thread_pool;
use num_cpus;
use request::Request;
use response::Response;
use std::{
    collections::HashMap,
    io::{BufReader, Write},
    net::{TcpListener, TcpStream},
    sync::{Arc, Mutex},
};
use thread_pool::ThreadPool;

type HandlerFn = Arc<dyn Fn(&Request, &mut Response) + Send + Sync + 'static>;

pub struct Xpress {
    address: String,
    routes: Arc<Mutex<HashMap<(String, String), HandlerFn>>>,
}

impl Xpress {
    pub fn new(address: &str) -> Self {
        Self {
            address: address.to_string(),
            routes: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn listen(&self) {
        let cpu_num = num_cpus::get();
        let pool = ThreadPool::new(cpu_num * 2);
        let listener = TcpListener::bind(&self.address).unwrap();

        for stream in listener.incoming() {
            let stream = stream.unwrap();
            let routes = Arc::clone(&self.routes);
            pool.execute(move || {
                handle_connection(stream, routes);
            });
        }
    }

    pub fn get<F>(&mut self, path: &str, handler: F)
    where
        F: Fn(&Request, &mut Response) + Send + Sync + 'static,
    {
        self.routes
            .lock()
            .unwrap()
            .insert(("GET".to_string(), path.to_string()), Arc::new(handler));
    }

    pub fn post<F>(&mut self, path: &str, handler: F)
    where
        F: Fn(&Request, &mut Response) + Send + Sync + 'static,
    {
        self.routes
            .lock()
            .unwrap()
            .insert(("POST".to_string(), path.to_string()), Arc::new(handler));
    }

    pub fn put<F>(&mut self, path: &str, handler: F)
    where
        F: Fn(&Request, &mut Response) + Send + Sync + 'static,
    {
        self.routes
            .lock()
            .unwrap()
            .insert(("PUT".to_string(), path.to_string()), Arc::new(handler));
    }

    pub fn delete<F>(&mut self, path: &str, handler: F)
    where
        F: Fn(&Request, &mut Response) + Send + Sync + 'static,
    {
        self.routes
            .lock()
            .unwrap()
            .insert(("DELETE".to_string(), path.to_string()), Arc::new(handler));
    }
}

fn handle_connection(
    mut stream: TcpStream,
    routes: Arc<Mutex<HashMap<(String, String), HandlerFn>>>,
) {
    let mut buf_reader = BufReader::new(&stream);
    let request = Request::from(&mut buf_reader);
    let mut response = Response::new();

    let handler = {
        let routes = routes.lock().unwrap();
        routes
            .get(&(request.method.to_string(), request.path.to_string()))
            .cloned()
    };

    if let Some(handler) = handler {
        handler(&request, &mut response);
        send_response(response, &mut stream);
    }
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
