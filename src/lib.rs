pub mod error;
pub mod request;
pub mod response;
mod thread_pool;

use error::XpressError;
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

type HandlerFn =
    Arc<dyn Fn(&Request, &mut Response) -> Result<(), XpressError> + Send + Sync + 'static>;

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

    pub fn listen(&self) -> Result<(), XpressError> {
        let listener = TcpListener::bind(&self.address).map_err(|e| XpressError::IoError(e))?;
        let pool = ThreadPool::new(num_cpus::get());

        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    let routes = Arc::clone(&self.routes);
                    pool.execute(move || {
                        if let Err(err) = handle_connection(stream, routes) {
                            eprintln!("Connection error: {}", err);
                        }
                    });
                }
                Err(err) => {
                    return Err(XpressError::ConnectionError(err.to_string()));
                }
            }
        }
        Ok(())
    }

    fn register_route<F>(&mut self, method: &str, path: &str, handler: F) -> Result<(), XpressError>
    where
        F: Fn(&Request, &mut Response) -> Result<(), XpressError> + Send + Sync + 'static,
    {
        let mut routes = self
            .routes
            .lock()
            .map_err(|_| XpressError::MutexError("Failed to acquire routes lock".to_string()))?;

        routes.insert((method.to_string(), path.to_string()), Arc::new(handler));
        Ok(())
    }

    pub fn get<F>(&mut self, path: &str, handler: F) -> Result<(), XpressError>
    where
        F: Fn(&Request, &mut Response) -> Result<(), XpressError> + Send + Sync + 'static,
    {
        self.register_route("GET", path, handler)?;
        Ok(())
    }

    pub fn post<F>(&mut self, path: &str, handler: F) -> Result<(), XpressError>
    where
        F: Fn(&Request, &mut Response) -> Result<(), XpressError> + Send + Sync + 'static,
    {
        self.register_route("POST", path, handler)?;
        Ok(())
    }

    pub fn put<F>(&mut self, path: &str, handler: F) -> Result<(), XpressError>
    where
        F: Fn(&Request, &mut Response) -> Result<(), XpressError> + Send + Sync + 'static,
    {
        self.register_route("PUT", path, handler)?;
        Ok(())
    }

    pub fn delete<F>(&mut self, path: &str, handler: F) -> Result<(), XpressError>
    where
        F: Fn(&Request, &mut Response) -> Result<(), XpressError> + Send + Sync + 'static,
    {
        self.register_route("DELETE", path, handler)?;
        Ok(())
    }
}

fn handle_connection(
    mut stream: TcpStream,
    routes: Arc<Mutex<HashMap<(String, String), HandlerFn>>>,
) -> Result<(), XpressError> {
    let mut buf_reader = BufReader::new(&stream);
    let request = Request::from(&mut buf_reader);
    let mut response = Response::new();

    let handler = {
        let routes = routes
            .lock()
            .map_err(|_| XpressError::MutexError("Failed to acquire routes lock".to_string()))?;
        routes
            .get(&(request.method.to_string(), request.path.to_string()))
            .cloned()
    };

    if let Some(handler) = handler {
        handler(&request, &mut response)?;
        send_response(response, &mut stream)?;
    } else {
        response.status(404);
        response.send("Not Found")?;
        send_response(response, &mut stream)?;
    }
    Ok(())
}

fn send_response(res: Response, stream: &mut TcpStream) -> Result<(), XpressError> {
    if res.sent {
        let response_string = format!(
            "HTTP/1.1 {}\r\n{}\r\nContent-Length: {}\r\n\r\n{}\r\n",
            res.status,
            res.headers
                .iter()
                .map(|(k, v)| format!("{}: {}", k, v))
                .collect::<Vec<_>>()
                .join("\r\n"),
            res.body.len(),
            String::from_utf8_lossy(&res.body)
        );
        stream.write_all(response_string.as_bytes())?;
    }
    Ok(())
}
