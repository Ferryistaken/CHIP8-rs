use std::path::PathBuf;

mod chip8;
use chip8::Chip8;

fn main() {
    let mut chip8: Chip8 = Chip8::new();

    // TODO: make it so that the file is taken as a positional argument
    chip8.load_rom(PathBuf::from(
        "roms/Chip8-Picture.ch8",
    ));

    println!("Program loaded, dumping rom");
    let _rom: Vec<u8> = chip8.dump_rom(0x0, 30);
}

