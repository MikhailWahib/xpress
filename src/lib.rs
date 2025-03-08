pub mod error;
pub mod request;
pub mod response;

use error::XpressError;
use request::Request;
use response::Response;
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use tokio::io::{AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::RwLock;

type SyncHandler = Box<dyn Fn(Request) -> Response + Send + Sync>;
type AsyncHandler =
    Box<dyn Fn(Request) -> Pin<Box<dyn Future<Output = Response> + Send>> + Send + Sync>;

enum Handler {
    Sync(SyncHandler),
    Async(AsyncHandler),
}

struct RouteInfo {
    method: String,
    handler: Handler,
}

pub struct Xpress {
    routes: Arc<RwLock<HashMap<String, RouteInfo>>>,
}

impl Xpress {
    pub fn new() -> Self {
        Self {
            routes: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn listen(&self, addr: &str) -> Result<(), XpressError> {
        let listener = TcpListener::bind(addr).await?;
        println!("Server listening on {}", addr);

        loop {
            let (socket, addr) = listener.accept().await?;
            println!("Connection from: {}", addr);

            let routes = Arc::clone(&self.routes);

            tokio::spawn(async move {
                if let Err(e) = Self::handle_connection(socket, routes).await {
                    eprintln!("Error handling connection: {}", e);
                }
            });
        }
    }

    async fn handle_connection(
        mut socket: TcpStream,
        routes: Arc<RwLock<HashMap<String, RouteInfo>>>,
    ) -> Result<(), XpressError> {
        let mut buffer = BufReader::new(&mut socket);

        // Parse request
        let request = match Request::from(&mut buffer).await {
            Ok(req) => req,
            Err(e) => {
                // Return a 400 Bad Request response
                let response = Response::new()
                    .status(400)
                    .body(&format!("Bad Request: {}", e));

                let response_bytes = response.to_bytes();
                socket.write_all(&response_bytes).await?;
                socket.flush().await?;
                return Err(e);
            }
        };

        let response = match Self::route_request(&request, &routes).await {
            Some(res) => res,
            None => Response::new().status(404).body("Not Found"),
        };

        let response_bytes = response.to_bytes();
        socket.write_all(&response_bytes).await?;
        socket.flush().await?;

        Ok(())
    }

    async fn route_request(
        request: &Request,
        routes: &Arc<RwLock<HashMap<String, RouteInfo>>>,
    ) -> Option<Response> {
        let routes = routes.read().await;

        if let Some(route_info) = routes.get(&request.path) {
            if route_info.method == request.method {
                match &route_info.handler {
                    Handler::Sync(handler) => Some(handler(request.clone())),
                    Handler::Async(handler) => Some(handler(request.clone()).await),
                }
            } else {
                Some(Response::new().status(405).body("Method Not Allowed"))
            }
        } else {
            None
        }
    }

    pub async fn get<F>(&mut self, path: &str, handler: F)
    where
        F: Fn(Request) -> Response + Send + Sync + 'static,
    {
        let mut routes = self.routes.write().await;
        routes.insert(
            path.to_string(),
            RouteInfo {
                method: "GET".to_string(),
                handler: Handler::Sync(Box::new(move |req: Request| handler(req))),
            },
        );
    }

    // Register POST handler (sync)
    pub async fn post<F>(&mut self, path: &str, handler: F)
    where
        F: Fn(Request) -> Response + Send + Sync + 'static,
    {
        let mut routes = self.routes.write().await;
        routes.insert(
            path.to_string(),
            RouteInfo {
                method: "POST".to_string(),
                handler: Handler::Sync(Box::new(move |req: Request| handler(req))),
            },
        );
    }

    // Register GET handler (async)
    pub async fn get_async<F, Fut>(&mut self, path: &str, handler: F)
    where
        F: Fn(Request) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Response> + Send + 'static,
    {
        let mut routes = self.routes.write().await;
        routes.insert(
            path.to_string(),
            RouteInfo {
                method: "GET".to_string(),
                handler: Handler::Async(Box::new(move |req| {
                    let future = handler(req);
                    Box::pin(future)
                })),
            },
        );
    }

    pub async fn post_async<F, Fut>(&mut self, path: &str, handler: F)
    where
        F: Fn(Request) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Response> + Send + 'static,
    {
        let mut routes = self.routes.write().await;
        routes.insert(
            path.to_string(),
            RouteInfo {
                method: "POST".to_string(),
                handler: Handler::Async(Box::new(move |req| {
                    let future = handler(req);
                    Box::pin(future)
                })),
            },
        );
    }
}
