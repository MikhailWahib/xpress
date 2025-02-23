mod error;
mod request;
mod response;

use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use tokio::io::{AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::RwLock;

pub use error::XpressError;
pub use request::Request;
pub use response::Response;

type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;
type AsyncHandlerFn = Arc<
    dyn Fn(Request, Response) -> BoxFuture<'static, Result<Response, XpressError>> + Send + Sync,
>;

pub struct Xpress {
    address: String,
    routes: Arc<RwLock<HashMap<(String, String), AsyncHandlerFn>>>,
}

impl Xpress {
    pub fn new(address: &str) -> Self {
        Self {
            address: address.to_string(),
            routes: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn listen(self) -> Result<(), XpressError> {
        let listener = TcpListener::bind(&self.address).await?;
        println!("Server listening on {}", self.address);

        loop {
            let (stream, _) = listener.accept().await?;
            let routes = Arc::clone(&self.routes);

            tokio::spawn(async move {
                if let Err(e) = handle_connection(stream, routes).await {
                    eprintln!("Connection error: {}", e);
                }
            });
        }
    }

    pub async fn get<F, Fut>(&mut self, path: &str, handler: F)
    where
        F: Fn(Request, Response) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<Response, XpressError>> + Send + 'static,
    {
        self.register_route("GET", path, handler).await;
    }

    pub async fn post<F, Fut>(&mut self, path: &str, handler: F)
    where
        F: Fn(Request, Response) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<Response, XpressError>> + Send + 'static,
    {
        self.register_route("POST", path, handler).await;
    }

    pub async fn put<F, Fut>(&mut self, path: &str, handler: F)
    where
        F: Fn(Request, Response) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<Response, XpressError>> + Send + 'static,
    {
        self.register_route("PUT", path, handler).await;
    }

    pub async fn delete<F, Fut>(&mut self, path: &str, handler: F)
    where
        F: Fn(Request, Response) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<Response, XpressError>> + Send + 'static,
    {
        self.register_route("DELETE", path, handler).await;
    }

    async fn register_route<F, Fut>(&mut self, method: &str, path: &str, handler: F)
    where
        F: Fn(Request, Response) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<Response, XpressError>> + Send + 'static,
    {
        let mut routes = self.routes.write().await;
        let handler = Arc::new(
            move |req, res| -> BoxFuture<'static, Result<Response, XpressError>> {
                Box::pin(handler(req, res))
            },
        );
        routes.insert((method.to_string(), path.to_string()), handler);
    }
}

async fn handle_connection(
    mut stream: TcpStream,
    routes: Arc<RwLock<HashMap<(String, String), AsyncHandlerFn>>>,
) -> Result<(), XpressError> {
    let mut buf_reader = BufReader::new(&mut stream);
    let request = Request::from_stream(&mut buf_reader).await?;
    let mut response = Response::new();

    let handler = {
        let routes = routes.read().await;
        routes
            .get(&(request.method.clone(), request.path.clone()))
            .cloned()
    };

    match handler {
        Some(handler) => {
            response = handler(request, response).await?;
        }
        None => {
            response.status(404);
            response.send("Not Found").await?;
        }
    }

    send_response(response, &mut stream).await?;
    Ok(())
}

async fn send_response(res: Response, stream: &mut TcpStream) -> Result<(), XpressError> {
    if res.sent {
        let status_text = match res.status {
            200 => "OK",
            404 => "Not Found",
            500 => "Internal Server Error",
            _ => "Unknown",
        };

        let response_string = format!(
            "HTTP/1.1 {} {}\r\n{}\r\nContent-Length: {}\r\n\r\n",
            res.status,
            status_text,
            res.headers
                .iter()
                .map(|(k, v)| format!("{}: {}", k, v))
                .collect::<Vec<_>>()
                .join("\r\n"),
            res.body.len(),
        );

        stream.write_all(response_string.as_bytes()).await?;
        stream.write_all(&res.body).await?;
        stream.flush().await?;
    }
    Ok(())
}
