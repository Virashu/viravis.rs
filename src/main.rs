use std::thread;
use std::{
    error::Error,
    sync::{Arc, Mutex},
};

use clap::builder::TypedValueParser;
use clap::Parser;

use viravis::modules::Serial;
use viravis::{modules::Server, AnalyzerMode, Viravis};

const SIZE: usize = 128;

#[derive(Parser)]
struct Args {
    #[arg(
        short,
        long,
        default_value_t = AnalyzerMode::Rolling,
        value_parser = clap::builder::PossibleValuesParser::new(["fft", "rolling"]).map(|s| s.parse::<AnalyzerMode>().unwrap()))]
    mode: AnalyzerMode,

    #[arg(short, long)]
    port: Option<String>,

    #[arg(long, action)]
    graph: bool,

    #[arg(long)]
    sample_rate: Option<u32>,
}

fn main() -> Result<(), Box<dyn Error>> {
    colog::default_builder()
        .filter(None, log::LevelFilter::Debug)
        .filter(Some("saaba"), log::LevelFilter::Warn)
        .init();

    let args = Args::parse();

    let data_mutex = Arc::new(Mutex::new(vec![0.0; SIZE]));

    // Catch data
    let mutex_ref = Arc::clone(&data_mutex);
    let cb = move |d| {
        let mut lock = mutex_ref.lock().unwrap();
        *lock = d;
    };

    // Send data to Server module
    let mutex_ref = Arc::clone(&data_mutex);
    thread::spawn(|| {
        let mut s = Server::new(mutex_ref);
        log::info!("Starting server");
        s.run();
    });

    if let Some(port) = args.port {
        // Send data to Serial module
        let mutex_ref = Arc::clone(&data_mutex);
        thread::spawn(|| {
            let s = Serial::new(mutex_ref, port);
            log::info!("Opening serial port");
            s.run();
        });
    }

    let mut v = Viravis::new(SIZE, args.mode, args.sample_rate)?;

    v.add_callback(cb);

    if args.graph {
        v.add_callback(viravis::graph::print_graph);

        print!("\x1b[?25l");
        ctrlc::set_handler(|| {
            print!("\x1b[H\x1b[J\x1b[?25h^C");
            std::process::exit(0);
        })?;
    }

    let orig_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        orig_hook(panic_info);
        log::error!("{}", panic_info);
        std::process::exit(1);
    }));

    log::info!("Starting Viravis!");
    v.run()?;

    Ok(())
}
