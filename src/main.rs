use std::path::PathBuf;

mod chip8;
use chip8::Chip8;

mod platform;

fn main() {
    let mut chip8: Chip8 = Chip8::new();

    // TODO: make it so that the file is taken as a positional argument
    chip8.load_rom(PathBuf::from(
        "roms/IBM-Logo.ch8",
    ));

    /*
    println!("Program loaded, dumping rom");
    let _rom8: Vec<u8> = chip8.dump_rom(0x50, 80);
    let _rom9: Vec<u8> = chip9.dump_rom(0x50, 80);
    */

    loop {
        chip8.Cycle();
        println!("{:?}", chip8.dump_video());
    }

}

