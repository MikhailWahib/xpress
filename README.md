# Xpress: A Lightweight HTTP Framework for Rust.

**Xpress** is a minimalistic, HTTP framework built from scratch in Rust. It aims to demonstrate the inner workings of web frameworks by implementing core features like routing, request parsing, and response handling in a simple yet extensible way.


_Note: This project is not intended for production use. It is designed to be a simple example of how an HTTP framework could be built from scratch in Rust, and should not be used in production environments._

---

## 🚀 Features
- **Routing**: Support for `GET`, `POST`, `PUT`, and `DELETE` methods.
- **Request Handling**: Parse HTTP requests and extract information seamlessly.
- **Response Management**: Send JSON, HTML, or plain text responses with ease.
- **Async Runtime**: Built on Tokio for efficient asynchronous request handling.
- **Error Handling**: Custom error handling for robust and maintainable code.
- **Educational Design**: Clear and commented implementation for learning purposes.
- **Comprehensive Tests**: Unit and integration tests ensuring reliability.

---

## 📦 Installation
Add `Xpress` to your project by including it in your `Cargo.toml`:
```toml
[dependencies]
xpress = "1.0.0"
```

---

## 🛠️ Usage

Here's a quick example to get started with `Xpress`:

```rust
use xpress::{Xpress, XpressError};

#[tokio::main]
async fn main() -> Result<(), XpressError> {
    // Initialize server
    let mut app = Xpress::new("127.0.0.1:8080");

    // GET /
    app.get("/", |_req, mut res| async move {
        match res.send("Hello World!").await {
            Ok(_) => Ok(res),
            Err(_) => {
                res.status(500);
                res.send("Internal Server Error").await?;
                Ok(res)
            }
        }
    })
    .await;

    println!("Server starting on http://127.0.0.1:8080");
    app.listen().await?;
    Ok(())
}

```

---

## 🧑‍💻 Contributing

Contributions are welcome! Feel free to submit issues or pull requests to improve functionality, fix bugs, or add examples.

---

## Local Development

To run the project locally, clone the repository and run the following commands:

```bash
# Clone the repository
git clone https://github.com/MikhailWahib/xpress.git

# Change the working directory
cd xpress

# Run the project
cargo run
```

### Running Tests

To run the tests, use the following command:

```bash

# Run the tests
cargo test
```

---

## 📜 License

This project is licensed under the [MIT License](LICENSE).

---