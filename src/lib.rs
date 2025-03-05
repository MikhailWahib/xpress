pub mod error;
mod request;
pub mod response;

use error::XpressError;
use request::Request;
use response::Response;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::io::{AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;

type Handler = Box<dyn Fn(Request) -> Response + Send + Sync>;

pub struct Xpress {
    routes: Arc<Mutex<HashMap<String, (String, Handler)>>>,
}

impl Xpress {
    pub fn new() -> Self {
        Self {
            routes: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn listen(&self, addr: &str) -> Result<(), XpressError> {
        let listener = TcpListener::bind(addr).await?;
        println!("Server listening on {}", addr);

        loop {
            let (socket, _) = listener.accept().await?;
            let routes = Arc::clone(&self.routes);

            tokio::spawn(async move {
                if let Err(e) = Self::handle_connection(socket, routes).await {
                    eprintln!("Error handling connection: {}", e);
                }
            });
        }
    }

    pub async fn get<F>(&self, path: &str, handler: F)
    where
        F: Fn(Request) -> Response + Send + Sync + 'static,
    {
        let mut routes = self.routes.lock().await;
        routes.insert(path.to_string(), (String::from("GET"), Box::new(handler)));
    }

    pub async fn post<F>(&self, path: &str, handler: F)
    where
        F: Fn(Request) -> Response + Send + Sync + 'static,
    {
        let mut routes = self.routes.lock().await;
        routes.insert(path.to_string(), (String::from("POST"), Box::new(handler)));
    }

    pub async fn put<F>(&self, path: &str, handler: F)
    where
        F: Fn(Request) -> Response + Send + Sync + 'static,
    {
        let mut routes = self.routes.lock().await;
        routes.insert(path.to_string(), (String::from("PUT"), Box::new(handler)));
    }

    pub async fn delete<F>(&self, path: &str, handler: F)
    where
        F: Fn(Request) -> Response + Send + Sync + 'static,
    {
        let mut routes = self.routes.lock().await;
        routes.insert(
            path.to_string(),
            (String::from("DELETE"), Box::new(handler)),
        );
    }

    async fn handle_connection(
        socket: TcpStream,
        routes: Arc<Mutex<HashMap<String, (String, Handler)>>>,
    ) -> Result<(), XpressError> {
        let mut buffer = BufReader::new(socket);
        let request = Request::from(&mut buffer).await?;
        let response = Self::route_request(&request, &routes).await;
        let response_str = Self::format_response(&response.unwrap());

        buffer.get_mut().write_all(response_str.as_bytes()).await?;
        buffer.get_mut().flush().await?;

        Ok(())
    }

    async fn route_request(
        request: &Request,
        routes: &Arc<Mutex<HashMap<String, (String, Handler)>>>,
    ) -> Option<Response> {
        let routes = routes.lock().await;
        routes.get(&request.path).and_then(|(method, handler)| {
            if *method == request.method {
                Some(handler(request.clone()))
            } else {
                None
            }
        })
    }

    fn format_response(response: &Response) -> String {
        let status_text = match response.status {
            200 => "OK",
            404 => "Not Found",
            _ => "Unknown",
        };

        let mut headers_str = String::new();
        for (key, value) in &response.headers {
            headers_str.push_str(&format!("{}: {}\r\n", key, value));
        }

        format!(
            "HTTP/1.1 {} {}\r\n\
            Content-Length: {}\r\n\
            {}\r\n\
            {}",
            response.status,
            status_text,
            response.body.len(),
            headers_str,
            String::from_utf8_lossy(&response.body)
        )
    }
}
