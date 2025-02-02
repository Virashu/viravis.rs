use std::sync::{Arc, Mutex};

use saaba::Response;

pub struct Server {
    app: saaba::App,
    data_mutex: Arc<Mutex<Vec<f32>>>,
}

impl Server {
    pub fn new(data_mutex: Arc<Mutex<Vec<f32>>>) -> Self {
        let app = saaba::App::new();

        Self { app, data_mutex }
    }

    pub fn run(&mut self) {
        let arc_ref = self.data_mutex.clone();
        self.app.get("/", move |_| {
            let data = arc_ref.lock().unwrap();
            let content = json::stringify((*data).clone());

            Response::from_content_string(content).with_header("Access-Control-Allow-Origin", "*")
        });

        self.app
            .run("0.0.0.0", 7777)
            .expect("Server failed to start");
    }
}
