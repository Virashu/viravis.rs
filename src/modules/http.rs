use std::io::Write;
use std::net::TcpListener;
use std::sync::{Arc, Mutex};

const CORS: &str = "Access-Control-Allow-Origin: *";

pub struct HttpServer {
    data_mutex: Arc<Mutex<Vec<f32>>>,
}

impl HttpServer {
    pub fn new(data_mutex: Arc<Mutex<Vec<f32>>>) -> Self {
        Self { data_mutex }
    }

    pub fn run(&self) {
        let listener = TcpListener::bind("0.0.0.0:7777")
            .inspect_err(|_| tracing::error!("Failed to start HTTP server"))
            .expect("Failed to start HTTP server");

        let arc_ref = self.data_mutex.clone();
        for mut stream in listener.incoming().filter_map(Result::ok) {
            let content;

            {
                let data = arc_ref.lock().unwrap();
                content = json::stringify((*data).clone());
            }

            let content_length = format!("Content-Length: {}", content.len());
            let response =
                format!("HTTP/1.1 200 OK\r\n{content_length}\r\n{CORS}\r\n\r\n{content}");

            stream.write_all(response.as_bytes()).unwrap();
        }
    }
}
