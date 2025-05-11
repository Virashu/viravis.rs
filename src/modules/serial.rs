use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

const LEDS: usize = 60;

pub struct Serial {
    data_mutex: Arc<Mutex<Vec<f32>>>,
    port: String,
}

impl Serial {
    pub fn new(data_mutex: Arc<Mutex<Vec<f32>>>, port: String) -> Self {
        Self { data_mutex, port }
    }

    fn get_data(&self) -> String {
        let mut data = { self.data_mutex.lock().unwrap().clone() };

        if data.len() > LEDS {
            data = Vec::from(&data[0..LEDS]);
        }

        let nums: Vec<String> = data
            .iter()
            .map(|n| n.round().clamp(0.0, 255.0) as u8)
            .map(|n| format!("{:02x?}", n))
            .collect();

        let mut nums_mirrored = nums.clone();
        nums_mirrored.reverse();

        format!("[{}]", [nums_mirrored, nums].concat().join(","))
    }

    pub fn run(&self) {
        let mut port = serialport::new(self.port.clone(), 115200)
            .open()
            .expect("Failed to open serial");

        loop {
            // [1,2,3]
            let data = self.get_data();
            port.write_all(data.as_bytes()).unwrap();
            thread::sleep(Duration::from_millis(20));
        }
    }
}
