use std::io::{stdout, Write};

const TERM_WIDTH: usize = 128;
const TERM_HEIGHT: i32 = 24;
const GRAPH_HEIGHT: i32 = TERM_HEIGHT - 2;

const F1_8: f32 = 0.125;
const F2_8: f32 = 0.25;
const F3_8: f32 = 0.375;
const F4_8: f32 = 0.5;
const F5_8: f32 = 0.625;
const F6_8: f32 = 0.75;
const F7_8: f32 = 0.875;

const BLOCKS: [char; 9] = [' ', '▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];

pub fn print_graph(data: Vec<f32>) {
    let data_cut = if data.len() > TERM_WIDTH {
        data[..TERM_WIDTH].to_vec()
    } else {
        data
    };
    // print!("\x1b[H\x1b[2J");

    let mut out = stdout().lock();

    write!(out, "\x1b[H\x1b[J").unwrap();
    writeln!(out, "{}", "⠉".repeat(TERM_WIDTH)).unwrap();
    for r in 0..GRAPH_HEIGHT {
        for h in &data_cut {
            let pivot = GRAPH_HEIGHT as f32 - h; // Amount of white, as h is amount of black

            let block: char;

            if r > pivot.ceil() as i32 {
                block = BLOCKS[8];
            } else if r as f32 >= pivot {
                let little_p = pivot.fract();
                let black_little_p = 1f32 - little_p;

                if black_little_p >= F7_8 {
                    block = BLOCKS[7];
                } else if black_little_p >= F6_8 {
                    block = BLOCKS[6];
                } else if black_little_p >= F5_8 {
                    block = BLOCKS[5];
                } else if black_little_p >= F4_8 {
                    block = BLOCKS[4];
                } else if black_little_p >= F3_8 {
                    block = BLOCKS[3];
                } else if black_little_p >= F2_8 {
                    block = BLOCKS[2];
                } else if black_little_p >= F1_8 {
                    block = BLOCKS[1];
                } else {
                    block = BLOCKS[0];
                }
            } else {
                block = BLOCKS[0];
            }

            write!(out, "{}", block).unwrap();
        }
        writeln!(out).unwrap();

        out.flush().unwrap();
    }
}
