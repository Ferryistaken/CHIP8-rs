#![cfg(all(feature = "cli", not(target_arch = "wasm32")))]
use std::path::PathBuf;
use structopt::StructOpt;

use chip8_rs::chip8;
use chip8::Chip8;
use std::{io, thread, time};

use chip8_rs::platform::{
    Chip8Screen,
    pump_input,
    fit_chip8_top_left,
    LogBuf
};

use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Terminal,
};

use ratatui::crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

#[derive(Debug, StructOpt)]
#[structopt(name = "Example", about = "CHIP8-rs options")]
struct Opt {
    /// Activate debug mode
    #[structopt(short, long)]
    debug: bool,

    /// Set speed (ms per cycle)
    #[structopt(short = "c", long = "clock", default_value = "10")]
    speed: u64,

    /// Input file
    #[structopt(short = "r", long = "rom", parse(from_os_str))]
    rom: PathBuf,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let opt = Opt::from_args();

    let mut chip8: Chip8 = Chip8::new();
    chip8.load_rom(opt.rom.clone());

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

    let mut logs = LogBuf::new(200);

    let hz = 1000 / opt.speed;

    loop {
        chip8.Cycle();

        let mut frame_keys = [0u8; 16];
        if pump_input(&mut chip8, &mut frame_keys, &mut logs)? {
            return Ok(());
        }
        chip8.keypad = frame_keys;

        let video = chip8.video;

        let last_op: u16 = chip8.last_opcode(); // implement this in your Chip8 as needed

        terminal.draw(|f| {
            let area = f.area();

            if !opt.debug {
                // Fullscreen simple mode
                f.render_widget(
                    Chip8Screen { video: &video, on: Color::White, off: Color::Black },
                    area,
                );
                return;
            }

            // Debug layout:
            // [ top: main row => (left chip8 screen, right logs) ]
            // [ bottom: status bar with last opcode ]
            let v = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Min(3),
                    Constraint::Length(3),
                ])
                .split(area);

            let main = v[0];
            let status = v[1];

            let h = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Min(10),
                    Constraint::Length(32),
                ])
                .split(main);

            let screen_host = h[0];
            let log_host = h[1];

            let screen_rect = fit_chip8_top_left(screen_host);
            f.render_widget(
                Chip8Screen { video: &video, on: Color::White, off: Color::Black },
                screen_rect,
            );

            let log_para = logs.to_paragraph("logs");
            f.render_widget(log_para, log_host);

            let status_line = Paragraph::new(Line::from(vec![
                Span::raw("opcode: "),
                Span::styled(format!("{:04X}", last_op), Style::default().fg(Color::Yellow)),
                Span::raw("\tquit:\t Esc or Q"),
                Span::raw("\t Speed: "),
                Span::styled(format!("{:X}", hz), Style::default().fg(Color::Red)),
                Span::raw(" Hz"),
            ]))
            .block(Block::default().borders(Borders::ALL).title("status"));
            f.render_widget(status_line, status);
        })?;

        thread::sleep(time::Duration::from_millis(opt.speed));
    }
}

