# Xpress: A Lightweight HTTP Library for Rust.

**Xpress** is a minimalistic HTTP framework built from scratch in Rust. It aims to demonstrate the inner workings of web frameworks by implementing core features like routing, request parsing, and response handling in a simple yet extensible way.

> ⚠️ **Note**: This project is not intended for production use. It is designed as an educational example of how an HTTP library can be built from scratch in Rust.

---

## 🚀 Features

* **Routing**: Support for `GET`, `POST`, `PUT`, and `DELETE` methods.
* **Dynamic Route Params**: Define routes with `/:id`-style parameters and extract them easily.
* **Query Params**: Parse query strings and access key-value pairs directly.
* **Request Handling**: Parse HTTP requests and extract information seamlessly.
* **Response Management**: Send JSON, HTML, or plain text responses with ease.
* **Thread Pooling**: Handle multiple client connections concurrently using a thread pool.
* **Error Handling**: Custom error handling for robust and maintainable code.
* **Educational Design**: Clear and commented implementation for learning purposes.

---

## 📦 Installation

Add `Xpress` to your project by including it in your `Cargo.toml`:

```toml
[dependencies]
xpress = "0.2.0"
```

---

## 🛠️ Usage

Here’s a quick example with **static routes, dynamic params, and query params**:

```rust
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use xpress::Xpress;

#[derive(Serialize, Deserialize, Clone)]
struct User {
    name: String,
    age: u8,
    email: String,
}

fn main() {
    let mut app = Xpress::new("127.0.0.1:8080");

    let state = Arc::new(Mutex::new(Vec::new()));

    // Static route
    app.get("/", |_req, res| res.html("hello.html").unwrap());

    // Route returning all users
    let users_state = Arc::clone(&state);
    app.get("/users", move |_req, res| {
        let users = users_state.lock().unwrap();
        res.json(&*users).unwrap()
    });

    // Route with dynamic param
    app.get("/users/:id", |req, res| {
        let id = req.params.get("id").unwrap(); // e.g. /users/42 -> "42"
        res.text(format!("User ID: {}", id)).unwrap()
    });

    // Route with query params
    app.get("/search", |req, res| {
        let q = req.query.get("q").unwrap_or(&"".to_string()); // e.g. /search?q=rust
        res.text(format!("Search query: {}", q)).unwrap()
    });

    app.listen();
}
```

---

## 🧑‍💻 Contributing

Contributions are welcome! Feel free to submit issues or pull requests to improve functionality, fix bugs, or add examples.

---

## 📜 License

This project is licensed under the [MIT License](LICENSE).
