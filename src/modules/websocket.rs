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
        let server = Server::bind("0.0.0.0:7778").expect("Failed to start websocket server");

        for connection in server.filter_map(Result::ok) {
            let arc_ref = self.data_mutex.clone();

            thread::spawn(move || {
                let mut client = connection.accept().unwrap();

                loop {
                    let data = { arc_ref.lock().unwrap().clone() };
                    let json_string = json::stringify(data);

                    match client.send_message(&Message::text(json_string)) {
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
