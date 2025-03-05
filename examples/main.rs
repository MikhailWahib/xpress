use xpress::{error::XpressError, response::Response, Xpress};

#[tokio::main]
async fn main() -> Result<(), XpressError> {
    let app = Xpress::new();

    app.get("/", |_req| Response::new().body("Hello, World!"))
        .await;

    app.post("/submit", |req| {
        Response::new().body(&format!("Received: {}", req.body))
    })
    .await;

    // app.get("/html", async |_req| {
    //     let file = tokio::fs::read_to_string("examples/index.html").await?;
    //     Ok(Response::new().html(file.as_str()))
    // })
    // .await;

    app.listen("127.0.0.1:8080").await
}
