use std::path::PathBuf;
use std::fs::File;
use std::io::prelude::*;

/// general chip 8 struct
pub struct Chip8 {
    register: [u8; 16],
    memory: [u8; 4096],
    index: u16,
    program_counter: u16,
    stack: [u16; 16],
    stack_pointer: u8,
    delay_timer: u8,
    sound_timer: u8,
    keypad: [u8; 16],
    video: [[u32; 64]; 32],
    op_code: u16
}

impl Chip8 {
    /// Create a new chip8 instance with everything empty
    pub fn new() -> Self {
        Chip8 {
            register: [0; 16],
            memory: [0; 4096],
            index: 0,
            program_counter: 0,
            stack: [0; 16],
            stack_pointer: 0,
            delay_timer: 0,
            sound_timer: 0,
            keypad: [0; 16],
            video: [[0; 64]; 32],
            op_code: 0
        }
    }
    
    pub fn load_rom(&mut self, path: PathBuf) {
        // memory addres before this are reserved
        let start_address = 0x200; 

        // open rom file
        let file = File::open("roms/Chip8-Picture.ch8").expect("Error opening rom file");

        // buffer to store bytes in
        let mut buf: Vec<u8> = Vec::new();

        // read the bytes from file
        for byte in file.bytes() {
            let byte = match byte {
                Ok(byte) => byte,
                Err(error) => panic!("Provided rom is not a valid binary. {:?}", error),
            };
            buf.push(byte);
        }
        // println!("{:x?}", buf);

        // load the buffer into the chip 8 memory
        for i in 0..buf.len() - 1 {
            self.memory[start_address + i] = buf[i];
        }
    }

    pub fn dump_rom(&self, pointer: usize, byte_number: usize) -> Vec<u8> {
        let mut rom: Vec<u8> = Vec::new();
        for i in pointer..self.memory.len() {
            rom.push(self.memory[i]);
        }

        return rom;
    }
}