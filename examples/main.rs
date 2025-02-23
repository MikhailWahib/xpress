use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use xpress::{Xpress, XpressError};

struct AppState {
    users: Vec<User>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct User {
    name: String,
    age: u8,
    email: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct UserResponse {
    message: String,
    user: User,
}

#[derive(Serialize, Deserialize, Debug)]
struct ErrorResponse {
    message: String,
}

#[tokio::main]
async fn main() -> Result<(), XpressError> {
    // Initialize server
    let mut app = Xpress::new("127.0.0.1:8080");

    // Initialize state
    let state = Arc::new(RwLock::new(AppState { users: Vec::new() }));

    // GET /
    app.get("/", |_req, mut res| async move {
        match res.html("examples/hello.html").await {
            Ok(_) => Ok(res),
            Err(_) => {
                res.status(500);
                res.send("Internal Server Error").await?;
                Ok(res)
            }
        }
    })
    .await;

    // GET /users - List all users
    let state_get = Arc::clone(&state);
    app.get("/users", move |_req, mut res| {
        let state_clone = state_get.clone();
        async move {
            let state = state_clone.read().await;
            res.json(&state.users).await?;
            Ok(res)
        }
    })
    .await;

    // POST /users - Create new user
    let state_post = Arc::clone(&state);
    app.post("/users", move |req, mut res| {
        let state_clone = state_post.clone();
        async move {
            match serde_json::from_str::<User>(&req.body) {
                Ok(user) => {
                    // Validate user data
                    if user.name.is_empty() || user.email.is_empty() {
                        res.status(400);
                        res.json(&ErrorResponse {
                            message: "Name and email are required".to_string(),
                        })
                        .await?;
                        return Ok(res);
                    }

                    // Store user
                    let mut state = state_clone.write().await;
                    state.users.push(user.clone());

                    // Return success response
                    res.status(201);
                    res.json(&UserResponse {
                        message: "User created successfully".to_string(),
                        user,
                    })
                    .await?;
                }
                Err(e) => {
                    res.status(400);
                    res.json(&ErrorResponse {
                        message: format!("Invalid user data: {}", e),
                    })
                    .await?;
                }
            }
            Ok(res)
        }
    })
    .await;

    // PUT /users - Update user
    let state_put = Arc::clone(&state);
    app.put("/users", move |req, mut res| {
        let state_clone = state_put.clone();
        async move {
            match serde_json::from_str::<User>(&req.body) {
                Ok(updated_user) => {
                    let mut state = state_clone.write().await;
                    if let Some(user) = state
                        .users
                        .iter_mut()
                        .find(|u| u.email == updated_user.email)
                    {
                        *user = updated_user.clone();
                        res.json(&UserResponse {
                            message: "User updated successfully".to_string(),
                            user: updated_user,
                        })
                        .await?;
                    } else {
                        res.status(404);
                        res.json(&ErrorResponse {
                            message: "User not found".to_string(),
                        })
                        .await?;
                    }
                }
                Err(e) => {
                    res.status(400);
                    res.json(&ErrorResponse {
                        message: format!("Invalid user data: {}", e),
                    })
                    .await?;
                }
            }
            Ok(res)
        }
    })
    .await;

    // DELETE /users - Delete user
    let state_delete = Arc::clone(&state);
    app.delete("/users", move |req, mut res| {
        let state_clone = state_delete.clone();
        async move {
            #[derive(Deserialize)]
            struct DeleteRequest {
                email: String,
            }

            match serde_json::from_str::<DeleteRequest>(&req.body) {
                Ok(delete_req) => {
                    let mut state = state_clone.write().await;
                    let initial_len = state.users.len();
                    state.users.retain(|user| user.email != delete_req.email);

                    if state.users.len() < initial_len {
                        res.json(&ErrorResponse {
                            message: "User deleted successfully".to_string(),
                        })
                        .await?;
                    } else {
                        res.status(404);
                        res.json(&ErrorResponse {
                            message: "User not found".to_string(),
                        })
                        .await?;
                    }
                }
                Err(e) => {
                    res.status(400);
                    res.json(&ErrorResponse {
                        message: format!("Invalid request data: {}", e),
                    })
                    .await?;
                }
            }
            Ok(res)
        }
    })
    .await;

    println!("Server starting on http://127.0.0.1:8080");
    app.listen().await?;
    Ok(())
}
