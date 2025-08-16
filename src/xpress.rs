use crate::error::XpressError;
use crate::request::Request;
use crate::response::Response;
use crate::thread_pool::ThreadPool;
use std::{
    collections::HashMap,
    io::{BufReader, Write},
    net::{TcpListener, TcpStream},
    sync::Arc,
};

type Handler = Box<dyn Fn(&Request, &mut Response) -> Result<(), XpressError> + Send + Sync>;
type Routes = HashMap<(String, String), Handler>;

pub struct Xpress {
    address: String,
    routes: Routes,
}

impl Xpress {
    pub fn new(address: &str) -> Self {
        Self {
            address: address.to_string(),
            routes: HashMap::new(),
        }
    }

    pub fn listen(self) -> Result<(), XpressError> {
        let listener = TcpListener::bind(&self.address)?;
        let routes = Arc::new(self.routes);
        let pool = ThreadPool::new(num_cpus::get());

        for stream in listener.incoming() {
            let stream = stream?;
            let routes = Arc::clone(&routes);

            pool.execute(move || {
                if let Err(err) = Self::handle_connection(stream, &routes) {
                    eprintln!("Connection error: {}", err);
                }
            });
        }
        Ok(())
    }

    pub fn get<F>(&mut self, path: &str, handler: F)
    where
        F: Fn(&Request, &mut Response) -> Result<(), XpressError> + Send + Sync + 'static,
    {
        self.routes
            .insert(("GET".to_string(), path.to_string()), Box::new(handler));
    }

    pub fn post<F>(&mut self, path: &str, handler: F)
    where
        F: Fn(&Request, &mut Response) -> Result<(), XpressError> + Send + Sync + 'static,
    {
        self.routes
            .insert(("POST".to_string(), path.to_string()), Box::new(handler));
    }

    pub fn put<F>(&mut self, path: &str, handler: F)
    where
        F: Fn(&Request, &mut Response) -> Result<(), XpressError> + Send + Sync + 'static,
    {
        self.routes
            .insert(("PUT".to_string(), path.to_string()), Box::new(handler));
    }

    pub fn delete<F>(&mut self, path: &str, handler: F)
    where
        F: Fn(&Request, &mut Response) -> Result<(), XpressError> + Send + Sync + 'static,
    {
        self.routes
            .insert(("DELETE".to_string(), path.to_string()), Box::new(handler));
    }

    fn handle_connection(mut stream: TcpStream, routes: &Routes) -> Result<(), XpressError> {
        let mut buf_reader = BufReader::new(&stream);
        let request = Request::from(&mut buf_reader);
        let mut response = Response::new();

        let key = (request.method.clone(), request.path.clone());

        if let Some(handler) = routes.get(&key) {
            handler(&request, &mut response)?;
        } else {
            response.status(404);
            response.send("Not Found")?;
        }

        Self::send_response(response, &mut stream)
    }

    fn send_response(response: Response, stream: &mut TcpStream) -> Result<(), XpressError> {
        let headers = format!(
            "HTTP/1.1 {}\r\n{}\r\nContent-Length: {}\r\n\r\n",
            response.status,
            response
                .headers
                .iter()
                .map(|(k, v)| format!("{}: {}", k, v))
                .collect::<Vec<_>>()
                .join("\r\n"),
            response.body.len()
        );

        stream.write_all(headers.as_bytes())?;
        stream.write_all(&response.body)?;
        stream.flush()?;

        Ok(())
    }
}
