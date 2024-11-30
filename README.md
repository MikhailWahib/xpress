# Xpress: A Lightweight HTTP Framework for Rust.

**Xpress** is a minimalistic, HTTP framework built from scratch in Rust. It aims to demonstrate the inner workings of web frameworks by implementing core features like routing, request parsing, and response handling in a simple yet extensible way.


_Note: This project is not intended for production use. It is designed to be a simple example of how an HTTP framework could be built from scratch in Rust, and should not be used in production environments._

---

## 🚀 Features
- **Routing**: Support for `GET`, `POST`, `PUT`, and `DELETE` methods.
- **Request Handling**: Parse HTTP requests and extract information seamlessly.
- **Response Management**: Send JSON, HTML, or plain text responses with ease.
- **Thread Pooling**: Handle multiple client connections concurrently using a thread pool.
- **Error Handling**: Custom error handling for robust and maintainable code.
- **Educational Design**: Clear and commented implementation for learning purposes.

---

## 📦 Installation
Add `Xpress` to your project by including it in your `Cargo.toml`:
```toml
[dependencies]
xpress = "0.1.3"
```

---

## 🛠️ Usage

Here's a quick example to get started with `Xpress`:

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

    // Define routes
    app.get("/", |_req, res| res.html("hello.html").unwrap());

    let users_state = Arc::clone(&state);
    app.get("/users", move |_req, res| {
        let users = users_state.lock().unwrap();
        res.json(&*users).unwrap();
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

---