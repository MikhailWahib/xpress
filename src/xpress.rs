use crate::request::Request;
use crate::response::Response;
use crate::thread_pool::ThreadPool;
use crate::{error::XpressError, router::Router};
use std::sync::Arc;
use std::{
    io::{BufReader, Write},
    net::{TcpListener, TcpStream},
};

pub struct Xpress {
    address: String,
    router: Router,
}

impl Xpress {
    pub fn new(address: &str) -> Self {
        Self {
            address: address.to_string(),
            router: Router::new(),
        }
    }

    pub fn listen(self) -> Result<(), XpressError> {
        let listener = TcpListener::bind(&self.address)?;
        let pool = ThreadPool::new(num_cpus::get());
        let router = Arc::new(self.router);

        for stream in listener.incoming() {
            let stream = stream?;
            let router = Arc::clone(&router);

            pool.execute(move || {
                if let Err(err) = Self::handle_connection(stream, &router) {
                    eprintln!("Connection error: {}", err);
                }
            });
        }
        Ok(())
    }

    fn handle_connection(mut stream: TcpStream, router: &Router) -> Result<(), XpressError> {
        let mut buf_reader = BufReader::new(&stream);
        let mut request = Request::try_from(&mut buf_reader)?;
        let mut response = Response::new();

        let result = (|| {
            let Some((handler, params)) =
                router.resolve(request.method.clone(), request.path.clone())
            else {
                return Err(XpressError::NotFound(format!(
                    "{} {}",
                    request.method, request.path
                )));
            };

            request.params = params;
            handler(&request, &mut response)?;
            Ok(response)
        })();

        match result {
            Ok(resp) => Self::send_response(resp, &mut stream)?,
            Err(err) => {
                // build error response
                let mut resp = Response::new();
                resp.status = err.status_code();
                resp.body = format!("Error: {}", err).into();
                Self::send_response(resp, &mut stream)?;
            }
        }

        Ok(())
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

    pub fn get<F>(&mut self, path: &str, handler: F)
    where
        F: Fn(&Request, &mut Response) -> Result<(), XpressError> + Send + Sync + 'static,
    {
        self.router
            .register_route(format!("GET {path}"), Box::new(handler))
            .unwrap();
    }

    pub fn post<F>(&mut self, path: &str, handler: F)
    where
        F: Fn(&Request, &mut Response) -> Result<(), XpressError> + Send + Sync + 'static,
    {
        self.router
            .register_route(format!("POST {path}"), Box::new(handler))
            .unwrap();
    }

    pub fn put<F>(&mut self, path: &str, handler: F)
    where
        F: Fn(&Request, &mut Response) -> Result<(), XpressError> + Send + Sync + 'static,
    {
        self.router
            .register_route(format!("PUT {path}"), Box::new(handler))
            .unwrap();
    }

    pub fn delete<F>(&mut self, path: &str, handler: F)
    where
        F: Fn(&Request, &mut Response) -> Result<(), XpressError> + Send + Sync + 'static,
    {
        self.router
            .register_route(format!("DELETE {path}"), Box::new(handler))
            .unwrap();
    }
}
