use http_server::HttpServer;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
struct User {
    name: String,
    age: u8,
    email: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct UserRes {
    message: String,
    user: User,
}

fn main() {
    let mut app = HttpServer::new("127.0.0.1:8080");

    app.get("/hi", |_req, res| {
        res.html("<h1>Welcome to homepage!</h1>").unwrap();
    });

    app.get("/user", |_req, res| {
        let user = User {
            name: "John Doe".to_string(),
            age: 30,
            email: "HtXH8@example.com".to_string(),
        };

        let response = UserRes {
            message: "Hello, world!".to_string(),
            user,
        };
        res.json(serde_json::to_string(&response).unwrap()).unwrap();
    });

    app.listen();
}
