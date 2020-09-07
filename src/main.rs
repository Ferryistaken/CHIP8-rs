use std::path::PathBuf;

mod chip8;
use chip8::Chip8;


fn main() {
    let mut chip8: Chip8 = Chip8::new();

    chip8.load_rom(PathBuf::from("/home/ferry/Documents/projects/rust/chip8-rs/roms/Chip8-Picture.ch8"));

    let rom: Vec<u8> = chip8.dump_rom(0x0, 30);

}