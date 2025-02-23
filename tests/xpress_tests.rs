use reqwest;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use xpress::Xpress;

// App state for testing
#[derive(Clone)]
struct TestState {
    counter: u32,
}

// Test model
#[derive(Serialize, Deserialize, Debug, Clone)]
struct TestModel {
    id: u32,
    name: String,
}

#[tokio::test]
async fn test_get_route() {
    let mut app = Xpress::new("127.0.0.1:8081");

    // Define a GET route
    app.get("/test", |_req, mut res| async move {
        res.send("Hello, World!").await.unwrap();
        Ok(res)
    })
    .await;

    // Start the server in the background
    tokio::spawn(async move {
        app.listen().await.unwrap();
    });

    // Give the server a moment to start
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Make a GET request to the route
    let client = reqwest::Client::new();
    let response = client
        .get("http://127.0.0.1:8081/test")
        .send()
        .await
        .unwrap();

    // Assert the response
    assert_eq!(response.status(), 200);
    assert_eq!(response.text().await.unwrap(), "Hello, World!");
}

#[tokio::test]
async fn test_post_route() {
    let mut app = Xpress::new("127.0.0.1:8082");

    // Define a POST route
    app.post("/test", |req, mut res| async move {
        let body = req.body;
        res.send(body).await.unwrap();
        Ok(res)
    })
    .await;

    // Start the server in the background
    tokio::spawn(async move {
        app.listen().await.unwrap();
    });

    // Give the server a moment to start
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Make a POST request to the route
    let client = reqwest::Client::new();
    let response = client
        .post("http://127.0.0.1:8082/test")
        .body("Test Body")
        .send()
        .await
        .unwrap();

    // Assert the response
    assert_eq!(response.status(), 200);
    assert_eq!(response.text().await.unwrap(), "Test Body");
}

#[tokio::test]
async fn test_json_response() {
    let mut app = Xpress::new("127.0.0.1:8083");

    // Define a GET route that returns JSON
    app.get("/json", |_req, mut res| async move {
        let data = TestModel {
            id: 1,
            name: "Test".to_string(),
        };
        res.json(&data).await.unwrap();
        Ok(res)
    })
    .await;

    // Start the server in the background
    tokio::spawn(async move {
        app.listen().await.unwrap();
    });

    // Give the server a moment to start
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Make a GET request to the route
    let client = reqwest::Client::new();
    let response = client
        .get("http://127.0.0.1:8083/json")
        .send()
        .await
        .unwrap();

    // Assert the response
    assert_eq!(response.status(), 200);
    let json: TestModel = response.json().await.unwrap();
    assert_eq!(json.id, 1);
    assert_eq!(json.name, "Test");
}

#[tokio::test]
async fn test_state_management() {
    let mut app = Xpress::new("127.0.0.1:8084");
    let state = Arc::new(RwLock::new(TestState { counter: 0 }));

    // Define a route that increments a counter in the state
    let state_clone = Arc::clone(&state);
    app.get("/increment", move |_req, mut res| {
        let state_clone = state_clone.clone();
        async move {
            let mut state = state_clone.write().await;
            state.counter += 1;
            res.send(format!("Counter: {}", state.counter))
                .await
                .unwrap();
            Ok(res)
        }
    })
    .await;

    // Start the server in the background
    tokio::spawn(async move {
        app.listen().await.unwrap();
    });

    // Give the server a moment to start
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Make multiple GET requests to increment the counter
    let client = reqwest::Client::new();
    for i in 1..=3 {
        let response = client
            .get("http://127.0.0.1:8084/increment")
            .send()
            .await
            .unwrap();

        // Assert the response
        assert_eq!(response.status(), 200);
        assert_eq!(response.text().await.unwrap(), format!("Counter: {}", i));
    }
}

#[tokio::test]
async fn test_error_handling() {
    let mut app = Xpress::new("127.0.0.1:8085");

    // Define a route that returns a 404 error
    app.get("/not_found", |_req, mut res| async move {
        res.status(404);
        res.send("Not Found").await.unwrap();
        Ok(res)
    })
    .await;

    // Start the server in the background
    tokio::spawn(async move {
        app.listen().await.unwrap();
    });

    // Give the server a moment to start
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Make a GET request to the route
    let client = reqwest::Client::new();
    let response = client
        .get("http://127.0.0.1:8085/not_found")
        .send()
        .await
        .unwrap();

    // Assert the response
    assert_eq!(response.status(), 404);
    assert_eq!(response.text().await.unwrap(), "Not Found");
}
