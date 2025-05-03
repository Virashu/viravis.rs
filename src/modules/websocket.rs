use std::{
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};
use websocket::{sync::Server, Message};

pub struct WebSocketServer {
    data_mutex: Arc<Mutex<Vec<f32>>>,
}

impl WebSocketServer {
    pub fn new(data_mutex: Arc<Mutex<Vec<f32>>>) -> Self {
        Self { data_mutex }
    }

    pub fn run(&self) {
        let server = Server::bind("0.0.0.0:7778")
            .inspect_err(|_| tracing::error!("Failed to start websocket server"))
            .expect("Failed to start websocket server");

        for connection in server.filter_map(Result::ok) {
            let arc_ref = self.data_mutex.clone();

            thread::spawn(move || {
                let mut client = connection.accept().unwrap();

                loop {
                    let content;
                    {
                        let data = arc_ref.lock().unwrap();
                        content = json::stringify((*data).clone());
                    }

                    let message = Message::text(content);
                    match client.send_message(&message) {
                        Ok(_) => {}
                        Err(_) => {
                            break;
                        }
                    }

                    thread::sleep(Duration::from_millis(20));
                }
            });
        }
    }
}
