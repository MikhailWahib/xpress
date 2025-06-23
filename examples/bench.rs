use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};
use xpress::{Xpress, XpressError};

struct AppState {
    users: Arc<RwLock<Vec<User>>>,
    // this is used for benchmark so i don't serialize users on each GET /users
    serialized_users_cache: RwLock<Vec<u8>>,
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
        users: Arc::new(RwLock::new(Vec::new())),
        serialized_users_cache: RwLock::new(Vec::new()),
    };

    // Populate users vec with initial entries
    {
        let mut users = app_state.users.write().unwrap();
        for i in 0..1000 {
            users.push(User {
                name: format!("User{}", i),
                age: 25,
                email: format!("user{}@example.com", i),
            });
        }

        *app_state.serialized_users_cache.write().unwrap() = serde_json::to_vec(&*users).unwrap();
    }

    app.get("/", |_req, res| match res.html("examples/hello.html") {
        Ok(_) => Ok(()),
        Err(err) => {
            res.status(500);
            res.send(format!("Error rendering HTML: {}", err))?;
            Ok(())
        }
    })?;

    let users_get = Arc::clone(&app_state.users);

    app.get("/users", move |_req, res| {
        // Check cache first
        match app_state.serialized_users_cache.read() {
            Ok(cache) if !cache.is_empty() => {
                res.headers
                    .insert("Content-Type".to_string(), "application/json".to_string());
                res.send(cache.to_vec())?;
            }
            _ => {
                // Fallback to live serialization
                match users_get.read() {
                    Ok(users) => res.json(&*users)?,
                    Err(_) => {
                        res.status(500);
                        res.send("Internal Server Error!")?;
                    }
                }
            }
        }
        Ok(())
    })?;

    let users_post = Arc::clone(&app_state.users);
    app.post("/users", move |req, res| {
        match req.from_json::<User>() {
            Ok(user) => {
                let mut users = users_post
                    .write()
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
    })?;

    println!("Server running on port {}", PORT);
    app.listen()?;
    Ok(())
}
