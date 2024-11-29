use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use xpress::Xpress;

struct AppState {
    users: Arc<Mutex<Vec<User>>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
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
    let app_state = AppState {
        users: Arc::new(Mutex::new(Vec::new())),
    };

    app.get("/", |_req, res| res.html("hello.html").unwrap());

    let users_get = Arc::clone(&app_state.users);
    app.get("/users", move |_req, res| match users_get.lock() {
        Ok(users) => {
            res.json(&*users).unwrap();
        }
        Err(_) => {
            res.status(500);
            res.send("Internal Server Error!").unwrap();
        }
    });

    let users_post = Arc::clone(&app_state.users);
    app.post("/users", move |req, res| {
        match serde_json::from_str::<User>(&req.body) {
            Ok(user) => {
                let response = UserRes {
                    message: "User created".to_string(),
                    user: user.clone(),
                };

                match users_post.lock() {
                    Ok(mut users) => {
                        users.push(user);
                        res.json(&response).unwrap();
                    }
                    Err(_) => {
                        res.status(500);
                        res.send("Internal Server Error!").unwrap();
                    }
                }
            }
            Err(_) => {
                res.status(400);
                res.send("Invalid user data").unwrap();
            }
        }
    });

    app.listen();
}
