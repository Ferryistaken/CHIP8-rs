use std::path::PathBuf;

mod chip8;
use chip8::Chip8;
use std::{thread, time};

use std::time::Duration;


fn main() {
    let mut chip8: Chip8 = Chip8::new();
    //chip8.debug();

    // TODO: make it so that the file is taken as a positional argument
    chip8.load_rom(PathBuf::from(
        "roms/test_opcode.ch8",
    ));


    loop {
        chip8.Cycle();
        chip8.pretty_print_video();
        thread::sleep(time::Duration::from_millis(100));
    }

}

