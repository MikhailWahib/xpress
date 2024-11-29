use serde::{Deserialize, Serialize};
use serde_json::ser;
use std::sync::{Arc, Mutex};
use xpress::Xpress;

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

    let users = Arc::new(Mutex::new(Vec::new()));

    let users_get = Arc::clone(&users);
    app.get("/users", move |_req, res| {
        let users = users_get.lock().unwrap();
        let users_res = serde_json::to_string(&*users);
        res.json(users_res.unwrap()).unwrap();
    });

    let users_post = Arc::clone(&users);
    app.post("/users", move |req, res| {
        let user: User = serde_json::from_str(&req.body).unwrap();

        let res_msg = UserRes {
            message: "User created".to_string(),
            user: user.clone(),
        };

        users_post.lock().unwrap().push(user);

        res.json(ser::to_string(&res_msg).unwrap()).unwrap();
    });

    app.listen();
}
