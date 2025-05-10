use std::thread;
use std::{
    error::Error,
    sync::{Arc, Mutex},
};

use clap::Parser;
use tracing::{error, info};

use viravis::{AnalyzerMode, Viravis};

mod analyzers;
mod graph;
mod modules;
mod viravis;

const SIZE: usize = 128;

#[derive(Parser)]
struct Args {
    /// Visualization mode
    #[arg(
        short,
        long,
        value_enum,
        default_value_t = AnalyzerMode::Rolling,
    )]
    mode: AnalyzerMode,

    /// Serial port with Viravis Arduino (optional)
    #[arg(short, long)]
    port: Option<String>,

    /// Turn on visualization in cli
    #[arg(long, action)]
    graph: bool,

    /// Audio sample rate
    #[arg(long)]
    sample_rate: Option<u32>,
}

fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    let args = Args::parse();

    let data_mutex = Arc::new(Mutex::new(vec![0.0; SIZE]));

    // Catch data
    let mutex_ref = data_mutex.clone();
    let cb = move |d: Vec<f32>| {
        for x in d.iter() {
            if x.is_nan() {
                tracing::error!("Encountered a `NaN` value in analyzer data!\n{:?}", d);
                return;
            } else if x.is_infinite() {
                tracing::error!("Encountered an `inf` value in analyzer data!\n{:?}", d);
                return;
            }
        }

        let mut lock = mutex_ref.lock().unwrap();
        *lock = d;
    };

    let mut v = Viravis::new(SIZE, args.mode, args.sample_rate)?;
    v.add_callback(cb);

    // Send data to Server module
    let mutex_ref = data_mutex.clone();
    thread::spawn(|| {
        let s = modules::HttpServer::new(mutex_ref);
        info!("Starting HTTP server");
        s.run();
    });

    // Send data to Ws Server module
    let mutex_ref = data_mutex.clone();
    thread::spawn(|| {
        let s = modules::WebSocketServer::new(mutex_ref);
        info!("Starting websocket server");
        s.run();
    });

    if let Some(port) = args.port {
        // Send data to Serial module
        let mutex_ref = data_mutex.clone();
        thread::spawn(|| {
            let s = modules::Serial::new(mutex_ref, port);
            tracing::info!("Opening serial port");
            s.run();
        });
    }

    if args.graph {
        v.add_callback(graph::print_graph);
        graph::init();
    }

    let orig_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        error!("{}", panic_info);
        orig_hook(panic_info);
        std::process::exit(1);
    }));

    ctrlc::set_handler(move || {
        if args.graph {
            graph::clear();
        }
        info!("Exiting Viravis, Goodbye!");
        std::process::exit(0);
    })
    .expect("Failed to set ctrlc handler");

    info!("Starting Viravis");
    v.run()?;

    Ok(())
}
