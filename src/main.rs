use serde::{Deserialize, Serialize};
use xpress::Xpress;

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

const PORT: u16 = 8080;

fn main() {
    let mut app = Xpress::new(&format!("127.0.0.1:{}", PORT));

    app.get("/", |_req, res| {
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

    println!("Listening on port: {}", PORT);
    app.listen();
}
