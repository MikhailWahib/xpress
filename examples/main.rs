use xpress::{error::XpressError, response::Response, Xpress};

#[tokio::main]
async fn main() -> Result<(), XpressError> {
    let mut app = Xpress::new();

    // Sync handlers
    app.get("/", |_req| Response::new().body("Hello, World!"))
        .await;

    app.post("/submit", |req| {
        let body = req.body_as_string().unwrap_or_default();
        Response::new().body(&format!("Received: {}", body))
    })
    .await;

    // Async handlers
    app.get_async("/delay", |_req| async {
        tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
        Response::new().body("Delayed response")
    })
    .await;

    // File serving with proper error handling
    app.get_async("/html", |_req| async {
        match tokio::fs::read_to_string("examples/hello.html").await {
            Ok(content) => Response::new().html(&content),
            Err(e) => Response::new().status(404).body(&format!("Error: {}", e)),
        }
    })
    .await;

    // JSON response example
    app.get_async("/json", |_req| async {
        let data = serde_json::json!({
            "message": "Hello from Xpress",
            "status": "success",
        });

        match Response::new().json(&data) {
            Ok(response) => response,
            Err(_) => Response::new()
                .status(500)
                .body("Error generating JSON response"),
        }
    })
    .await;

    println!("Server starting on http://127.0.0.1:8080");
    app.listen("127.0.0.1:8080").await
}
