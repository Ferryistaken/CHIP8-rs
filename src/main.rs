use std::path::PathBuf;
use structopt::StructOpt;

mod chip8;
use chip8::Chip8;
use std::{io, thread, time};

mod platform;

use ratatui::{
    backend::CrosstermBackend,
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    widgets::Widget,
    Terminal,
};

use ratatui::crossterm::{
    execute,
    event::{self, Event, KeyCode, KeyEvent, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

fn map_pc_to_chip8(code: KeyCode) -> Option<u8> {
    use KeyCode::*;
    match code {
        // 1 2 3 C  => 1 2 3 4
        Char('1') => Some(0x1),
        Char('2') => Some(0x2),
        Char('3') => Some(0x3),
        Char('4') => Some(0xC),

        // 4 5 6 D  => Q W E R
        Char('q') | Char('Q') => Some(0x4),
        Char('w') | Char('W') => Some(0x5),
        Char('e') | Char('E') => Some(0x6),
        Char('r') | Char('R') => Some(0xD),

        // 7 8 9 E  => A S D F
        Char('a') | Char('A') => Some(0x7),
        Char('s') | Char('S') => Some(0x8),
        Char('d') | Char('D') => Some(0x9),
        Char('f') | Char('F') => Some(0xE),

        // A 0 B F  => Z X C V
        Char('z') | Char('Z') => Some(0xA),
        Char('x') | Char('X') => Some(0x0),
        Char('c') | Char('C') => Some(0xB),
        Char('v') | Char('V') => Some(0xF),

        _ => None,
    }
}

fn pump_input(
    chip8: &mut Chip8,
    frame_keys: &mut [u8; 16],
) -> Result<bool, Box<dyn std::error::Error>> {
    while event::poll(std::time::Duration::from_millis(0))? {
        match event::read()? {
            Event::Key(KeyEvent { code, kind, .. }) => {
                // exit keys
                if code == KeyCode::Esc || code == KeyCode::Char('Q') {
                    return Ok(true);
                }
                if let Some(k) = map_pc_to_chip8(code) {
                    match kind {
                        KeyEventKind::Press | KeyEventKind::Repeat => {
                            frame_keys[k as usize] = 1;

                            if chip8.keypad[k as usize] == 0 {
                                chip8.key_down(k);
                            }
                        }
                        KeyEventKind::Release => {
                            // ignore: we reset keypad each frame anyway
                        }
                        _ => {}
                    }
                }
            }
            Event::Resize(_, _) | Event::Mouse(_) => {}
            _ => {}
        }
    }
    Ok(false)
}

struct Chip8Screen<'a> {
    video: &'a [u32; 64 * 32], // nonzero = ON
    on: Color,
    off: Color,
}

impl<'a> Widget for Chip8Screen<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.width == 0 || area.height == 0 { return; }
        for ty in 0..area.height {
            let sy = (ty as u32 * 32) / area.height as u32; // 0..31
            for tx in 0..area.width {
                let sx = (tx as u32 * 64) / area.width as u32; // 0..63
                let idx = (sy * 64 + sx) as usize;
                let on = self.video[idx] != 0;

                let cell = buf.get_mut(area.x + tx, area.y + ty);
                cell.set_symbol(" ");
                cell.set_style(Style::default().bg(if on { self.on } else { self.off }));
            }
        }
    }
}

#[derive(Debug, StructOpt)]
#[structopt(name = "Example", about = "CHIP8-rs options")]
struct Opt {
    /// Activate debug mode
    // short and long flags (-d, --debug) will be deduced from the field's name
    #[structopt(short, long)]
    debug: bool,

    /// Set speed
    // we don't want to name it "speed", need to look smart
    #[structopt(short = "c", long = "clock", default_value = "10")]
    speed: u64,

    /// Input file
    #[structopt(short = "r", long = "rom", parse(from_os_str))]
    rom: PathBuf,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let opt = Opt::from_args();

    let mut chip8: Chip8 = Chip8::new();
    //chip8.debug();

    chip8.load_rom(opt.rom);

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    struct Cleanup;
    impl Drop for Cleanup {
        fn drop(&mut self) {
            let _ = disable_raw_mode();
            let _ = execute!(io::stdout(), LeaveAlternateScreen);
        }
    }
    let _cleanup = Cleanup;

    loop {
        chip8.Cycle();

        let mut frame_keys = [0u8; 16];

        if pump_input(&mut chip8, &mut frame_keys)? {
            return Ok(());
        }

        chip8.keypad = frame_keys;

        let video = chip8.export_video();
        terminal.draw(|f| {
            f.render_widget(
                Chip8Screen {
                    video,
                    on: Color::White,
                    off: Color::Black,
                },
                f.area(),
            );
        })?;
        thread::sleep(time::Duration::from_millis(opt.speed));
    }

}

