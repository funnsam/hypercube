use crossterm::{*, style::*, event::*};
use std::io::*;
use std::time::*;
use std::ptr::null_mut;

pub static mut LOGS: Vec<(SystemTime, String)> = Vec::new();

pub static mut STDOUT_BUF: *mut BufWriter<Stdout> = null_mut();

pub fn push_log(s: &str) {
    unsafe {
        let t = SystemTime::now();
        LOGS.push((t, s.to_string()));
    }
}

pub fn init(so: &mut BufWriter<Stdout>) -> std::result::Result<u16, Box<dyn std::error::Error>> {
    unsafe { STDOUT_BUF = so as *mut BufWriter<Stdout> };

    terminal::enable_raw_mode()?;
    execute!(stdout(),
        terminal::EnterAlternateScreen,
        terminal::Clear(terminal::ClearType::All),
        cursor::Hide
    )?;

    let (c, r) = terminal::size()?;

    Ok(c.min((r-2) << 1))
}

pub fn prep_exit() -> core::result::Result<(), Box<dyn std::error::Error>> {
    execute!(stdout(),
        terminal::LeaveAlternateScreen,
        cursor::Show
    )?;
    terminal::disable_raw_mode()?;

    Ok(())
}

pub fn push_image(image: Vec<Vec<u8>>, msg: &str) -> core::result::Result<usize, Box<dyn std::error::Error>> {
    let so = unsafe { &mut *STDOUT_BUF };
    queue!(so, cursor::MoveTo(0, 0))?;
    for y in image.chunks(2) {
        let rle = rle_row(y);
        for i in rle.into_iter() {
            queue!(so,
                style::PrintStyledContent(
                    "\u{2580}".repeat(i.n)
                        .with(Color::Rgb { r: i.d.0, g: i.d.0, b: i.d.0 })
                        .on  (Color::Rgb { r: i.d.1, g: i.d.1, b: i.d.1 })
                )
            )?;
        }

        queue!(so, cursor::MoveToNextLine(1))?;
    }

    queue!(so,
        terminal::Clear(terminal::ClearType::FromCursorDown),
        style::PrintStyledContent(
            msg .with(Color::White)
                .on  (Color::Black)
        )
    )?;

    let total_size = so.buffer().len();

    so.flush()?;

    Ok(total_size)
}

struct RLEChunk {
    n: usize,
    d: (u8, u8)
}
fn rle_row(src: &[Vec<u8>]) -> Vec<RLEChunk> {
    let mut ic_at = 0;
    let mut ichunks = Vec::with_capacity(src[0].len());
    let mut fchunks = Vec::new();
    for i in 0..src[0].len() {
        if src.len() == 2 {
            ichunks.push(RLEChunk {
                n: 1,
                d: (src[0][i], src[1][i])
            })
        } else {
            ichunks.push(RLEChunk {
                n: 1,
                d: (src[0][i], 0)
            })
        }
    }
    let mut n = 0;
    let mut d = ichunks[0].d;
    let mut acc_d = (0, 0);
    while let Some(this) = next(&ichunks, &mut ic_at) {
        let diff_0 = this.d.0.abs_diff(d.0);
        let diff_1 = this.d.1.abs_diff(d.1);
        if diff_0 <= crate::COMPRESSION_DIFF &&
           diff_1 <= crate::COMPRESSION_DIFF {
            n += 1;
            acc_d.0 += this.d.0 as usize;
            acc_d.1 += this.d.1 as usize;
        } else {
            fchunks.push(RLEChunk {
                n,
                d: (
                    (acc_d.0 / n) as u8,
                    (acc_d.1 / n) as u8,
                )
            });
            n = 1;
            d = this.d;
            acc_d = (
                this.d.0 as usize,
                this.d.1 as usize,
            );
        }
    }
    fchunks.push(RLEChunk {
        n,
        d: (
            (acc_d.0 / n) as u8,
            (acc_d.1 / n) as u8,
            )
    });
    fchunks
}

fn next<'a, A>(a: &'a Vec<A>, i: &'a mut usize) -> Option<&'a A> {
    let b = a.get(*i);
    *i += 1;
    b
}

pub fn show(s: &str) -> core::result::Result<(), Box<dyn std::error::Error>> {
    let so = unsafe { &mut *STDOUT_BUF };
    execute!(so,
        cursor::MoveTo(0, 0),
        style::PrintStyledContent(s
            .with(Color::White)
            .on  (Color::DarkGrey)
        )
    )?;
    Ok(())
}

pub fn handle_input(el: Duration, state: &mut crate::renderer::State) -> std::result::Result<Option<usize>, Box<dyn std::error::Error>> {
    let pr = poll(
        Duration::from_millis(
            ((1000.0 / crate::MAX_FPS) as u128).checked_sub(el.as_millis()).unwrap_or(0) as u64
        )
    )?;
    if pr {
        match read()? {
            Event::Resize(c, r) => {
                let so = unsafe { &mut *STDOUT_BUF };
                execute!(so, terminal::Clear(terminal::ClearType::All))?;
                return Ok(Some(c.min((r-2) << 1) as usize));
            },
            Event::Key(KeyEvent { code: KeyCode::Esc, .. }) => {
                unsafe {
                    let mut s = String::new();

                    for i in &LOGS {
                        s += &format!("{:.02}s ago: {}\n", i.0.elapsed()?.as_secs_f64(), i.1);
                    }

                    std::fs::write("logs.txt", s)?;
                }

                prep_exit()?;
                std::process::exit(0);
            },
            Event::Key(KeyEvent { code: KeyCode::Char('r'), kind: KeyEventKind::Press, .. }) => {
                state.toggle_rotate ^= true;
            },
            Event::Key(KeyEvent { code: KeyCode::Char('w'), kind: KeyEventKind::Press, .. }) => state.p.z += 0.25,
            Event::Key(KeyEvent { code: KeyCode::Char('s'), kind: KeyEventKind::Press, .. }) => state.p.z -= 0.25,
            Event::Key(KeyEvent { code: KeyCode::Char('d'), kind: KeyEventKind::Press, .. }) => state.p.x += 0.25,
            Event::Key(KeyEvent { code: KeyCode::Char('a'), kind: KeyEventKind::Press, .. }) => state.p.x -= 0.25,
            Event::Key(KeyEvent { code: KeyCode::Char('q'), kind: KeyEventKind::Press, .. }) => state.p.y += 0.25,
            Event::Key(KeyEvent { code: KeyCode::Char('e'), kind: KeyEventKind::Press, .. }) => state.p.y -= 0.25,
            Event::Key(KeyEvent { code: KeyCode::Char('p'), kind: KeyEventKind::Press, .. }) => state.p.w += 0.25,
            Event::Key(KeyEvent { code: KeyCode::Char('l'), kind: KeyEventKind::Press, .. }) => state.p.w -= 0.25,
            _ => ()
        }
    }

    Ok(None)
}
