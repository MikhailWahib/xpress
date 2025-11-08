# Xpress: A Lightweight HTTP Library for Rust

**Xpress** is a minimalistic HTTP library built from scratch in Rust. It demonstrates the inner workings of web frameworks by implementing core features like routing, request parsing, and response handling in a simple, extensible way.

> ⚠️ **Note**: This project is not intended for production use. It is an educational example showing how an HTTP library can be built from scratch in Rust.

---

## 🚀 Features

* **Trie-Based Routing**: Efficiently match static and dynamic routes using a trie structure for high-performance route lookups.
* **Dynamic Route Params**: Easily define routes with `/:id`-style parameters and extract them.
* **Query Params**: Parse query strings and access key-value pairs directly.
* **Request Handling**: Seamlessly parse incoming HTTP requests.
* **Response Management**: Send JSON, HTML, or plain text responses easily.
* **Thread Pooling**: Handle multiple client connections concurrently.
* **Error Handling**: Robust and customizable error handling.
---

## 📦 Installation

Add `Xpress` to your project in `Cargo.toml`:

```toml
[dependencies]
xpress = "0.2.0"
```

---

## 🛠️ Usage

Here’s a quick example with **static routes, dynamic params, and query params**:

```rust
fn main() -> Result<(), XpressError> {
    const PORT: u16 = 8080;
    let mut app = Xpress::new(&format!("127.0.0.1:{}", PORT));

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

    println!("Server running on port {}", PORT);
    app.listen()?;
    Ok(())
}
```

---

## 🧑‍💻 Contributing

Contributions are welcome! Submit issues or pull requests to improve functionality, fix bugs, or add examples.

---

## 📜 License

This project is licensed under the [MIT License](LICENSE).
