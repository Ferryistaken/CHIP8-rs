use std::collections::VecDeque;
use std::path::PathBuf;
use structopt::StructOpt;

use crate::chip8::Chip8;
use std::{io, thread, time};

use ratatui::{
    backend::CrosstermBackend,
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Widget, Wrap},
    Terminal,
};

use ratatui::crossterm::{
    execute,
    event::{self, Event, KeyCode, KeyEvent, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

// ---------- Key mapping ----------
pub fn map_pc_to_chip8(code: KeyCode) -> Option<u8> {
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

pub fn pump_input(
    chip8: &mut Chip8,
    frame_keys: &mut [u8; 16],
    logs: &mut LogBuf,
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
                            logs.push(format!("key {:X} down", k));
                        }
                        KeyEventKind::Release => {
                            // we clear keys each frame anyway
                        }
                        _ => {}
                    }
                }
            }
            Event::Resize(w, h) => {
                logs.push(format!("resize -> {}x{}", w, h));
            }
            Event::Mouse(_) => {}
            _ => {}
        }
    }
    Ok(false)
}

pub struct Chip8Screen<'a> {
    pub video: &'a [u32; 64 * 32], // nonzero = ON
    pub on: Color,
    pub off: Color,
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

pub struct LogBuf {
    lines: VecDeque<String>,
    cap: usize,
}

impl LogBuf {
    pub fn new(cap: usize) -> Self {
        Self { lines: VecDeque::new(), cap }
    }
    pub fn push<T: Into<String>>(&mut self, s: T) {
        if self.lines.len() == self.cap { self.lines.pop_front(); }
        self.lines.push_back(s.into());
    }

    // Use a 'static title to avoid lifetime entanglement with the Paragraph
    pub fn to_paragraph(&self, title: &'static str) -> Paragraph<'static> {
        // Own all line data so Paragraph doesn't borrow from self
        let lines: Vec<Line<'static>> = self.lines.iter().cloned().map(Line::from).collect();
        Paragraph::new(lines)
            .block(Block::default().borders(Borders::ALL).title(title))
            .wrap(Wrap { trim: false })
    }
}

// “replace println!” helper
macro_rules! dbg_log {
    ($logs:expr, $($arg:tt)*) => {{
        $logs.push(format!($($arg)*));
    }};
}

pub fn fit_chip8_top_left(area: Rect) -> Rect {
    let aspect_w = 64.0f32;
    let aspect_h = 32.0f32; // 2:1
    let want = aspect_w / aspect_h;

    let aw = area.width.max(1) as f32;
    let ah = area.height.max(1) as f32;

    // try max width, then adjust height; if too tall, clamp by height
    let mut w = aw;
    let mut h = (aw / want).floor();
    if h > ah {
        h = ah;
        w = (ah * want).floor();
    }
    Rect {
        x: area.x,
        y: area.y,
        width: w as u16,
        height: h as u16,
    }
}
