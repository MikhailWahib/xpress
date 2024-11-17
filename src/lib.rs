mod thread_pool;

use std::{
    fs,
    io::{BufRead, BufReader, Error, ErrorKind, Result, Write},
    net::{TcpListener, TcpStream},
    thread,
    time::Duration,
};

use thread_pool::ThreadPool;

pub struct HttpServer {
    address: String,
    thread_count: usize,
}

impl HttpServer {
    pub fn new(address: impl Into<String>, thread_count: usize) -> Self {
        Self {
            address: address.into(),
            thread_count,
        }
    }

    pub fn run(&self) -> Result<()> {
        let pool = ThreadPool::new(self.thread_count);
        let listener = TcpListener::bind(&self.address)?;

        for stream in listener.incoming() {
            let stream = stream?;
            pool.execute(|| {
                if let Err(e) = Self::handle_connection(stream) {
                    eprintln!("Error handling connection: {}", e);
                }
            });
        }

        Ok(())
    }

    fn handle_connection(mut stream: TcpStream) -> Result<()> {
        let buf_reader = BufReader::new(&stream);
        let req = buf_reader
            .lines()
            .next()
            .ok_or_else(|| Error::new(ErrorKind::InvalidData, "Empty request"))??;

        let (status_line, filename) = match &req[..] {
            "GET / HTTP/1.1" => ("HTTP/1.1 200 OK", "hello.html"),
            "GET /sleep HTTP/1.1" => {
                thread::sleep(Duration::from_secs(4));
                ("HTTP/1.1 200 OK", "hello.html")
            }
            _ => ("HTTP/1.1 404 NOT FOUND", "404.html"),
        };

        let contents = fs::read_to_string(filename)?;
        let length = contents.len();
        let response = format!("{status_line}\r\nContent-Length: {length}\r\n\r\n{contents}");

        stream.write_all(response.as_bytes())?;
        Ok(())
    }
}
