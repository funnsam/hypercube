#![feature(int_roundings)]

pub const FOV: f32 = 70.0;

mod renderer;

/*
use std::time::SystemTime;
use std::io::*;

pub const STDOUT_BUF_SIZE: usize = 128*KB;
pub const COMPRESSION_DIFF: u8 = 16;
pub const MAX_FPS: f64 = 60.0;
const KB: usize = 1024;

mod terminal;

fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let mut so = BufWriter::with_capacity(STDOUT_BUF_SIZE, stdout());
    let mut size = terminal::init(&mut so)? as usize;

    let mut fps = 0.0;
    let mut out_size = 0;

    let mut state = renderer::State::default();
    state.f = renderer::fov_to_fl(FOV);
    
    loop {
        let s = SystemTime::now();

        let rr = renderer::render(&mut state, size);

        out_size = terminal::push_image(rr, &format!("FPS {fps:.1} total / {:.1} render\r\nBuffer size {:.1}KB / {:.1}KB", 1000.0 / (s.elapsed()?.as_nanos() as f64 / 1e+6), out_size as f32 / KB as f32, STDOUT_BUF_SIZE as f32 / KB as f32))?;
        size = terminal::handle_input(s.elapsed()?, &mut state)?.unwrap_or(size);

        let total = s.elapsed()?.as_nanos() as f64 / 1e+6;
        fps = (fps + 1000.0 / total) / 2.0;

        if state.toggle_rotate {
            state.r4 += (std::f32::consts::TAU * 0.125) * (total as f32 / 1000.0);
            state.r3 += (std::f32::consts::TAU * 0.125) * (total as f32 / 1000.0) * (1.0 / 4.0);
        }
    }
}
*/

use std::{fs::File, io::Write};
use gif::*;
const SIZE: [usize; 2] = [1080, 1920];
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut palette = Vec::with_capacity(256*3);
    for i in 0..=255 {
        palette.push(i >> 1);
        palette.push(i);
        palette.push(i >> 1);
    }

    let mut file = File::create("hypercube.gif")?;
    let mut encoder = Encoder::new(&mut file, SIZE[0] as u16, SIZE[1] as u16, &palette)?;
    encoder.set_repeat(Repeat::Infinite)?;

    let mut state = renderer::State::default();
    for i in 0..20*32 {
        let mut frame = Frame::default();
        frame.width  = SIZE[0] as u16;
        frame.height = SIZE[1] as u16;
        frame.delay  = 100 / 20;
        let f = renderer::render(&mut state, SIZE[0], SIZE[1]).into_iter().flatten().collect::<Vec<u8>>();
        frame.buffer = std::borrow::Cow::Borrowed(&f);
        encoder.write_frame(&frame)?;

        state.r4 += (std::f32::consts::TAU * 0.125) * (1.0 / 20.0);
        state.r3 += (std::f32::consts::TAU * 0.125) * (1.0 / 20.0) * (1.0 / 4.0);

        print!("\x1b[0GFrame {i} done");
        std::io::stdout().lock().flush()?;
    }

    println!();

    Ok(())
}
