use std::path::PathBuf;
use chip8_rs::Chip8;

mod lib;
fn main() {
    let mut chip8: Chip8 = Chip8::new();

    chip8.load_rom(PathBuf::from("/home/ferry/Documents/projects/rust/chip8-rs/roms/Chip8-Picture.ch8"));

    println!("{:?}", chip8.dump_rom(0x200, 30));
}