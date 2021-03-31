use std::path::PathBuf;

mod chip8;
use chip8::Chip8;

mod platform;

fn main() {
    let mut chip8: Chip8 = Chip8::new();

    let mut chip9: Chip8 = Chip8::new();

    chip9.load_rom(PathBuf::from("roms/IBM Logo.ch8"));

    // TODO: make it so that the file is taken as a positional argument
    chip8.load_rom(PathBuf::from(
        "roms/Chip8-Picture.ch8",
    ));

    println!("Program loaded, dumping rom");
    let _rom8: Vec<u8> = chip8.dump_rom(0x50, 80);
    let _rom9: Vec<u8> = chip9.dump_rom(0x50, 80);

    eprintln!("{:?} Lenght is: {}", _rom8, _rom8.len());
    eprintln!("{:?} Lenght is: {}", _rom9, _rom9.len());
}

