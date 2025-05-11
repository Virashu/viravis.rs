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
        let listener = TcpListener::bind("0.0.0.0:7777").expect("Failed to start HTTP server");

        for mut stream in listener.incoming().filter_map(Result::ok) {
            let data = { self.data_mutex.lock().unwrap().clone() };
            let json_string = json::stringify(data);

            let content_length = format!("Content-Length: {}", json_string.len());
            let response =
                format!("HTTP/1.1 200 OK\r\n{content_length}\r\n{CORS}\r\n\r\n{json_string}");

            stream.write_all(response.as_bytes()).unwrap();
        }
    }
}
