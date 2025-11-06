use std::cell::{RefCell, Cell};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{window, HtmlCanvasElement, CanvasRenderingContext2d};

use crate::chip8::Chip8;

thread_local! {
    static EMU: RefCell<Emu> = RefCell::new(Emu::new_uninit());
    // Keep the JS callback alive for setTimeout
    static TICK_CB: RefCell<Option<Closure<dyn FnMut()>>> = RefCell::new(None);
    static KEYS_DOWN_MASK: Cell<u32> = Cell::new(0); // bit i set => key i down
}

struct Emu {
    chip8: Chip8,
    keys_down: [bool; 16],
    canvas: HtmlCanvasElement,
    ctx: CanvasRenderingContext2d,
    loaded: bool,
    running: bool,
    delay_ms: u32, // delay between *instruction* ticks
}

impl Emu {
    fn new_uninit() -> Self {
        let doc = window().unwrap().document().unwrap();
        let canvas = doc.create_element("canvas").unwrap().dyn_into::<HtmlCanvasElement>().unwrap();
        let ctx = canvas.get_context("2d").unwrap().unwrap().dyn_into::<CanvasRenderingContext2d>().unwrap();
        Self {
            chip8: Chip8::new(),
            keys_down: [false;16],
            canvas,
            ctx,
            loaded: false,
            running: false,   // start paused (manual stepping)
            delay_ms: 12,      // ~250 Hz instruction rate; tweak as you like
        }
    }

    fn apply_keymask_edges(&mut self, mask: u32) {
        for i in 0..16 {
            let now = (mask >> i) & 1 != 0;
            let was = self.keys_down[i];
            if now && !was { self.chip8.key_down(i as u8); }
            if !now && was { self.chip8.key_up(i as u8); }
            self.keys_down[i] = now;
        }
    }

    /// Execute exactly ONE CHIP-8 instruction, then present.
    fn tick_once(&mut self) {
        // If your core has Result, handle/log it instead of panicking.
        self.chip8.Cycle();   // or self.chip8.cycle().ok();
        self.present();
    }

    fn present(&self) {
        let w = self.canvas.width() as f64;
        let h = self.canvas.height() as f64;
        let cols = 64f64;
        let rows = 32f64;
        let pw = (w / cols).floor();
        let ph = (h / rows).floor();

        // Clear
        self.ctx.set_fill_style(&"#000".into());
        self.ctx.fill_rect(0.0, 0.0, w, h);
        self.ctx.set_image_smoothing_enabled(false);
        self.ctx.set_fill_style(&"#2182ff".into());

        let fb = &self.chip8.video; // works for [u8] / [u32] as 0/!0

        for y in 0..32usize {
            for x in 0..64usize {
                if fb[y * 64 + x] != 0 {
                    let xf = (x as f64) * pw;
                    let yf = (y as f64) * ph;
                    self.ctx.fill_rect(xf, yf, pw, ph);
                }
            }
        }
    }
}

fn schedule_next_timeout(delay_ms: i32) {
    TICK_CB.with(|slot| {
        if let Some(cb) = slot.borrow().as_ref() {
            let _ = window().unwrap()
                .set_timeout_with_callback_and_timeout_and_arguments_0(
                    cb.as_ref().unchecked_ref(), delay_ms
                );
        }
    });
}

#[wasm_bindgen]
pub fn init(canvas_id: &str) -> Result<(), JsValue> {
    // Better panic messages in console (enable via your "web" feature):
    #[cfg(feature = "web")]
    console_error_panic_hook::set_once();

    let doc = window().ok_or("no window")?.document().ok_or("no document")?;
    let canvas: HtmlCanvasElement = doc.get_element_by_id(canvas_id).ok_or("canvas not found")?.dyn_into()?;
    let ctx: CanvasRenderingContext2d = canvas.get_context("2d")?.unwrap().dyn_into()?;

    EMU.with(|cell| {
        let mut emu = cell.borrow_mut();
        emu.canvas = canvas;
        emu.ctx = ctx;
        // draw once so you see a blank screen
        emu.present();
    });

    // Build the tick closure (one instruction per timeout)
    TICK_CB.with(|slot| {
        let cb = Closure::wrap(Box::new(move || {
            // read input mask (no borrow)
            let mask = KEYS_DOWN_MASK.with(|c| c.get());
            EMU.with(|cell| {
                let mut emu = cell.borrow_mut();

                // Edge-detect keypad regardless of paused/running
                emu.apply_keymask_edges(mask);

                if emu.loaded && emu.running {
                    emu.tick_once(); // ONE instruction, ONE present
                } else {
                    // Even if paused, repaint (e.g., after reset/load)
                    emu.present();
                }

                // Log a cheap “is it alive” counter (optional)
                // let fb = &emu.chip8.video;
                // let lit: u32 = fb.iter().map(|&p| (p != 0) as u32).sum();
                // web_sys::console::log_1(&format!("lit={}", lit).into());

                // schedule next instruction tick
                let delay = emu.delay_ms as i32;
                drop(emu); // explicit drop before scheduling
                schedule_next_timeout(delay);
            });
        }) as Box<dyn FnMut()>);
        *slot.borrow_mut() = Some(cb);
    });

    // Kick the first timeout so the loop starts (it will do nothing when paused)
    schedule_next_timeout(16); // first repaint in ~1 frame
    Ok(())
}

// ---------- Controls exported to JS ----------

#[wasm_bindgen]
pub fn step(n: u32) {
    // Run exactly n instructions, painting after each.
    EMU.with(|cell| {
        let mut emu = cell.borrow_mut();
        for _ in 0..n {
            if !emu.loaded { break; }
            emu.tick_once();
        }
    });
}

#[wasm_bindgen]
pub fn set_running(run: bool) {
    EMU.with(|cell| cell.borrow_mut().running = run);
}

#[wasm_bindgen]
pub fn set_delay_ms(ms: u32) {
    EMU.with(|cell| cell.borrow_mut().delay_ms = ms.max(1));
}

#[wasm_bindgen]
pub fn load_rom(bytes: &[u8]) -> Result<(), JsValue> {
    EMU.with(|cell| {
        let mut emu = cell.borrow_mut();
        emu.chip8.reset_and_load_bytes(bytes)
            .map_err(|e| JsValue::from_str(&format!("{e:?}")))?;
        emu.loaded = true;
        // after loading, present once; stay paused by default
        emu.present();
        Ok(())
    })
}

#[wasm_bindgen]
pub fn reset() {
    EMU.with(|cell| {
        let mut emu = cell.borrow_mut();
        emu.chip8 = Chip8::new();
        emu.keys_down = [false;16];
        emu.loaded = false;
        KEYS_DOWN_MASK.with(|c| c.set(0));
        emu.present();
    });
}

#[wasm_bindgen]
pub fn set_key(idx: u8, down: bool) {
    if idx >= 16 { return; }
    KEYS_DOWN_MASK.with(|mask_cell| {
        let mut m = mask_cell.get();
        if down { m |= 1 << idx; } else { m &= !(1 << idx); }
        mask_cell.set(m);
    });
}

