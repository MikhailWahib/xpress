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
        let request = {
            let mut buf_reader = BufReader::new(&mut stream);
            match Request::try_from(&mut buf_reader) {
                Ok(req) => Some(req),
                Err(e) => {
                    eprintln!("Request parsing error: {}", e);
                    None
                }
            }
        };

        if let Some(mut req) = request {
            let mut response = Response::new();
            let result = (|| {
                let Some((handler, params)) = router.resolve(req.method.clone(), req.path.clone())
                else {
                    return Err(XpressError::NotFound(format!(
                        "{} {}",
                        req.method, req.path
                    )));
                };
                req.params = params;
                handler(&req, &mut response)?;
                Ok(response)
            })();

            let mut resp = match result {
                Ok(resp) => resp,
                Err(err) => {
                    let mut resp = Response::new();
                    resp.status = err.status_code();
                    resp.body = format!("Error: {}", err).into();
                    resp
                }
            };

            resp.headers
                .insert("Connection".to_string(), "close".to_string());

            Self::send_response(resp, &mut stream)?;
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
