use serde::{Deserialize, Serialize};
use std::{
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};
use xpress::{Xpress, XpressError};

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

fn main() -> Result<(), XpressError> {
    let mut app = Xpress::new(&format!("127.0.0.1:{}", PORT));
    let app_state = AppState {
        users: Arc::new(Mutex::new(Vec::new())),
    };

    app.get("/", |_req, res| match res.html("examples/hello.html") {
        Ok(_) => Ok(()),
        Err(err) => {
            res.status(500);
            res.send(format!("Error rendering HTML: {}", err))?;
            Ok(())
        }
    });

    app.get("/test/:id", |req, res| {
        let id = req.params.get("id").unwrap();
        let foo = req.query.get("foo").unwrap();
        res.send(format!("{} {}", id.clone(), foo.clone()))?;
        Ok(())
    });

    app.get("/delay", |_req, res| {
        thread::sleep(Duration::from_secs(10));
        res.send("Am I Late?")?;
        Ok(())
    });

    let users_get = Arc::clone(&app_state.users);
    app.get("/users", move |_req, res| match users_get.lock() {
        Ok(users) => {
            res.json(&*users)?;
            Ok(())
        }
        Err(_) => {
            res.status(500);
            res.send("Internal Server Error!")?;
            Ok(())
        }
    });

    let users_post = Arc::clone(&app_state.users);

    app.post("/users", move |req, res| {
        match req.from_json::<User>() {
            Ok(user) => {
                let mut users = users_post
                    .lock()
                    .map_err(|_| XpressError::MutexError("Failed to acquire lock".to_string()))?;
                users.push(user.clone());
                let response = UserRes {
                    message: "User created".to_string(),
                    user,
                };
                res.json(&response)?;
            }

            Err(_) => {
                res.status(400);
                res.send("Invalid user data")?;
            }
        }

        Ok(())
    });

    println!("Server running on port {}", PORT);
    app.listen()?;
    Ok(())
}
